/*
references are expanded
arrays of dependants established

map element maps string name of point to point's ast and a vector of the string name of points depending on that point

first expandation expands ast and create deps lists
then every operation is type/space annotated and the deps list swaps string names for hard indexes

datatree is just the ast with space for storing computed results



map references
perform substitutions
determine size/type




*/

/**
 * TODO
 * 	- flatten select operators   graph.point => point
 * 	- add types, for non-base types, second operand must be graph or ref
 */
use super::parser::Ast;
use super::tokenizer::Name;
use std::collections::BTreeMap;

pub type Maps = Vec<BTreeMap<String, (usize, Ast, Vec<String>)>>;

#[derive(Debug, Clone)]
pub enum tAst {
	Nothing,
	//
	Integer(String),
	Decimal(String),
	String(String),
	Clock(String, Name),

	Graph(Vec<Ast>), // { .. }
	Space(Vec<Ast>), // [ .. ]

	Apply(Box<Ast>, Box<Ast>), //	graph { .. }

	//  space, label,   point
	Key(Box<Ast>, String, Box<Ast>), // word: exp
	// Ret(Box<Ast>),                 // -> exp
	Ref(String),                             // word
	Op2(Box<Ast>, Name, Box<Ast>, Box<Ast>), // 1 + 2
	Op1(Box<Ast>, Name, Box<Ast>),           // - 10
}

#[derive(Debug, Clone)]
pub struct Parse {
	pub envs: Maps,
}

pub fn expander(ast: &Ast) -> Result<(Ast, Maps), String> {
	let mut parse = Parse {
		envs: vec![BTreeMap::new()], // 0th un-used, parent 0 used to mean "no-parent"
	};
	Ok((parse.map(ast, 0)?, parse.envs))
}

fn key_match(point_a: &Ast, point_b: &Ast) -> bool {
	match point_a {
		Ast::Key(label_a, _) => {
			match point_b {
				Ast::Key(label_b, _) => {
					// note, rust auto-derefs references during comparisons
					label_a == label_b
				}
				_ => false,
			}
		}
		_ => false,
	}
}

fn has_key(points: &Vec<Ast>, point_a: &Ast) -> bool {
	for point in points {
		if key_match(point, point_a) {
			return true;
		}
	}
	false
}

fn get_key(points: &Vec<Ast>, point_a: &Ast) -> Option<Ast> {
	for point in points {
		if key_match(point, point_a) {
			return Some(point.clone());
		}
	}
	None
}

impl Parse {
	fn map(&mut self, ast: &Ast, parent: usize) -> Result<Ast, String> {
		match ast {
			Ast::Graph(points) => {
				let index = self.envs.len();
				let mut env = BTreeMap::new();
				env.insert(
					"<parent>".to_string(),
					(parent, Ast::Nothing, vec![]),
				);

				self.envs.push(env);

				let mut expanded_points = Vec::new();
				for point in points {
					expanded_points.push(self.map(point, index)?);
				}

				Ok(Ast::Graph(expanded_points))
			}

			Ast::Apply(source, modifier) => {
				let index = self.envs.len();
				let mut env = BTreeMap::new();
				env.insert(
					"<parent>".to_string(),
					(parent, Ast::Nothing, Vec::new()),
				);
				self.envs.push(env);
				// removed the concept of ordinal args, because I could not create an intuative set of replacement rules, especially with intermediary points
				// removed the concept of the intermediary points, because it also creates confusion
				let source = self.deref(source, parent)?;

				let s_points = match &source.1 {
					Ast::Graph(points) => (*points).clone(),
					_ => Vec::new(),
				};

				let m_points = match (**modifier).clone() {
					Ast::Graph(points) => points,
					_ => Vec::new(),
				};

				// throw error for modifier points that don't exist in source
				for m_point in &m_points {
					if !has_key(&s_points, &m_point) {
						return Err(format!(
							"{:?} does not exist in {:?}",
							&m_point, &s_points
						));
					}
				}

				let mut points = Vec::new();
				for s_point in s_points {
					points.push(match get_key(&m_points, &s_point) {
						Some(point) => point,
						None => s_point.clone(),
					});
				}

				let mut expanded_points = Vec::new();
				for point in points {
					expanded_points.push(self.map(&point, index)?);
				}

				Ok(Ast::Graph(expanded_points))
			}

			Ast::Key(label, point) => {
				let point = self.map(point, parent)?;
				self.envs[parent]
					.insert(label.clone(), (parent, point.clone(), vec![]));
				Ok(Ast::Key(label.clone(), Box::new(point)))
			}

			Ast::Ref(label) => {
				// dbg!(self.lookup(label, parent));
				Ok(Ast::Ref(label.clone()))
			}
			Ast::Op2(name, left, right) => match name {
				Name::Select => {
					let p = self.deref(left, parent)?;
					let k = self.deref(right, p.0);
					dbg!(p);
					dbg!(k);
					dbg!(&self.envs);
					Ok(Ast::Nothing)
				}
				_ => {
					let left = self.map(left, parent)?;
					let right = self.map(right, parent)?;
					Ok(Ast::Op2(*name, Box::new(left), Box::new(right)))
				}
			},
			Ast::Op1(name, operand) => {
				let operand = self.map(operand, parent)?;
				Ok(Ast::Op1(*name, Box::new(operand)))
			}
			Ast::Op0(name) => Ok(Ast::Op0(*name)),
			Ast::Space(sizes) => {
				let mut ss = vec![];
				for size in sizes {
					ss.push(self.map(size, parent)?)
				}
				Ok(Ast::Space(ss))
			}

			Ast::Decimal(i) => Ok(Ast::Decimal(i.clone())),
			Ast::Integer(i) => Ok(Ast::Integer(i.clone())),
			Ast::String(i) => Ok(Ast::String(i.clone())),
			Ast::Clock(t, i) => Ok(Ast::Clock(t.clone(), i.clone())),
			Ast::Size(i) => Ok(Ast::Size(i.clone())),
			Ast::Nothing => Ok(Ast::Nothing),
			// x => return Err(format!("No map match defined for {:?}", x)),
		}
	}

	fn deref(
		&self,
		ref_: &Ast,
		env: usize,
	) -> Result<&(usize, Ast, Vec<String>), String> {
		match (*ref_).clone() {
			Ast::Ref(label) => self.lookup(&label, env),
			ast => Err(format!("{:?} is not a Reference", ast)),
		}
	}
	fn lookup(
		&self,
		label: &String,
		env: usize,
	) -> Result<&(usize, Ast, Vec<String>), String> {
		if env == 0 {
			return Err(format!("Point {} is undefined", label));
		}
		match self.envs[env].get(label) {
			Some(result) => Ok(result),
			None => self.lookup(
				label,
				self.envs[env].get(&"<parent>".to_string()).unwrap().0,
			),
		}
	}
}
