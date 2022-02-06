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
// use super::tokenizer::Name;
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
	// ast
	Ok(Ast::Nothing)
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

fn is_label(ast: &Ast) -> bool {
	match ast {
		Ast::Key(_, _) => true,
		_ => false,
	}
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

impl Parse {
	fn map(&mut self, ast: &Ast, parent: usize) -> Result<String, String> {
		match ast {
			Ast::Graph(points) => {
				let index = self.envs.len();
				let mut env = BTreeMap::new();
				env.insert(
					"<parent>".to_string(),
					(parent, Ast::Nothing, vec![]),
				);

				self.envs.push(env);

				for point in points {
					self.map(point, index)?;
				}
			}

			Ast::Apply(source, modifier) => {
				let source = self.deref(source, parent)?;

				let s_points = match &source.1 {
					Ast::Graph(points) => (*points).clone(),
					_ => Vec::new(),
				};

				let m_points = match (**modifier).clone() {
					Ast::Graph(points) => points,
					_ => Vec::new(),
				};

				/*
				 * labeled points in the modifier replace labeled points of same name in source
				 *	unlabeled points in modifier replace points of same index in source
				 *	labeled points in modifier  do not replace points of samed index in source
				 *	modifier may contain intermediary points that have no corollary in source
				 *	intermediary points with no corollary should be placed in same index as modifier
				 * for sanity, ordinal modifiers should be required to precede labeled modifiers
				 * it is possible to have contention between ordinal and labeled modifiers, the last modifier wins
				 */
				// let mut points = vec![];
				// for (i, point) in s_points.iter().enumerate() {
				// 	// m_points[i] exists and m_points[i] is not labeled
				// 	if i < m_points.len() && !is_label(&m_points[i]) {
				// 		let p = match point {
				// 			Ast::Key(label, _) => Ast::Key(
				// 				label.clone(),
				// 				Box::new(m_points[i].clone()),
				// 			),
				// 			_ => m_points[i].clone(),
				// 		};
				// 		points.push(p);
				// 	} else if i < m_points.len()
				// 		&& !has_key(&s_points, &m_points[i])
				// 	{
				// 	}
				// }

				let mut final_points: Vec<Ast> = vec![];
				let mut shared_points: Vec<Ast> = vec![];
				let mut i = 0;
				for m_point in m_points {
					if !is_label(&m_point) {
						// ordinal
						if i < s_points.len() {
							let p = match &s_points[i] {
								Ast::Key(label, _) => Ast::Key(
									label.clone(),
									Box::new(m_point.clone()),
								),
								_ => m_point.clone(),
							};
							final_points.push(p);
						} else {
							final_points.push(m_point.clone())
						}
					} else if !has_key(&s_points, &m_point) {
						// intermediary
						final_points.push(m_point.clone())
					} else {
						shared_points.push(m_point.clone())
					}

					i += 1;
				}
			}

			Ast::Ret(point) => {}
			Ast::Key(label, point) => {
				self.map(point, parent)?;
				self.envs[parent]
					.insert(label.clone(), (parent, (**point).clone(), vec![]));
			}

			Ast::Ref(label) => {}
			Ast::Op2(name, left, right) => {}
			Ast::Op1(name, operand) => {}
			Ast::Space(sizes) => {}
			Ast::Decimal(i) => {}
			Ast::Integer(i) => {}
			Ast::String(i) => {}
			Ast::Nothing => {}

			x => return Err(format!("No map match defined for {:?}", x)),
		}
		return Ok("No error".to_string());
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
