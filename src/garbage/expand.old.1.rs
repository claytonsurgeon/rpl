/*
references are normalized
arrays of dependants established

map element maps string name of point to point's ast and a vector of the string name of points depending on that point

first normalization expands ast and create deps lists
then every operation is type/space annotated and the deps list swaps string names for hard indexes

datatree is just the ast with space for storing computed results



map references
perform substitutions
determine size/type




*/

use super::parser::Ast;
use super::tokenizer::Name;
use std::collections::BTreeMap;

// pub struct Map<'a> {
// 	pub parent: Option<&'a BTreeMap<String, (Ast, Vec<String>)>>,
// 	pub this: BTreeMap<String, (Ast, Vec<String>)>,
// }

// pub struct Parse<'a> {
#[derive(Debug, Clone)]
pub struct Parse {
	// ast: &'a Ast,
	//								parent, ast, dependants
	// envs: &'a mut Vec<BTreeMap<String, (usize, Ast, Vec<String>)>>,
	envs: Vec<BTreeMap<String, (usize, Ast, Vec<String>)>>,
}

pub fn normalize(ast: &Ast) -> Result<Ast, String> {
	let mut parse = Parse {
		envs: vec![BTreeMap::new()], // 0th un-used, parent 0 used to mean "no-parent"
	};
	let ast = parse.map(ast, 0);
	// get dependancies
	//  dependancy = (parent-index, btree-index)
	// let ast = parse.apply
	// dbg!(&parse);
	ast
}

fn strip_keys(ast: &Ast) -> Ast {
	match ast {
		Ast::Key(_, point) => *point.clone(),
		x => x.clone(),
	}
}

fn flatten_key(top: bool, point: &Ast, normal_points: &mut Vec<Ast>) {
	match point {
		Ast::Key(label, point) => {
			normal_points
				.push(Ast::Key(label.clone(), Box::new(strip_keys(&point))));

			flatten_key(false, &point, normal_points);
		}
		Ast::Ret(point) => {
			normal_points.push(Ast::Ret(Box::new(strip_keys(&point))));

			flatten_key(false, &point, normal_points);
		}
		_ => {
			if top {
				normal_points.push(point.clone())
			}
		}
	}
}
fn flatten_keys(points: Vec<Ast>, normal_points: &mut Vec<Ast>) {
	for point in points {
		flatten_key(true, &point, normal_points)
	}
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

				let mut pts = vec![];
				for point in points {
					pts.push(self.map(point, index)?)
				}

				let mut normal_points = vec![];
				flatten_keys(pts, &mut normal_points);

				Ok(Ast::Graph(normal_points))
			}

			Ast::Ref(label) => {
				// let item = self.lookup(label, parent)?;
				Ok(Ast::Ref(label.clone()))
			}

			Ast::Ret(point) => {
				let p = self.map(point, parent)?;
				self.envs[parent].insert(
					"<return>".to_string(),
					(parent, strip_keys(&p), vec![]),
				);
				Ok(Ast::Ret(Box::new(p)))
			}

			Ast::Key(label, point) => {
				let p = self.map(point, parent)?;
				self.envs[parent]
					.insert(label.clone(), (parent, strip_keys(&p), vec![]));
				Ok(Ast::Key(label.clone(), Box::new(p)))
			}

			Ast::Op2(name, left, right) => {
				let left = self.map(left, parent)?;
				let right = self.map(right, parent)?;
				Ok(Ast::Op2(*name, Box::new(left), Box::new(right)))
			}

			Ast::Op1(name, operand) => {
				let operand = self.map(operand, parent)?;
				Ok(Ast::Op1(*name, Box::new(operand)))
			}

			Ast::Apply(name, operand) => {
				let operand = self.map(operand, parent)?;
				Ok(Ast::Apply((*name).clone(), Box::new(operand)))
			}

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
			Ast::Nothing => Ok(Ast::Nothing),

			x => Err(format!("No map match defined for {:?}", x)),
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
