/*
references are normalized
arrays of dependants established

map element maps string name of point to point's ast and a vector of the string name of points depending on that point

first normalization expands ast and create deps lists
then every operation is type/space annotated and the deps list swaps string names for hard indexes

datatree is just the ast with space for storing computed results








*/

use super::parser::Ast;
use super::tokenizer::Name;
use std::collections::BTreeMap;

// pub struct Map<'a> {
// 	pub parent: Option<&'a BTreeMap<String, (Ast, Vec<String>)>>,
// 	pub this: BTreeMap<String, (Ast, Vec<String>)>,
// }

// pub struct Parse<'a> {
pub struct Parse {
	// ast: &'a Ast,
	//								parent, ast, dependants
	// envs: &'a mut Vec<BTreeMap<String, (usize, Ast, Vec<String>)>>,
	envs: Vec<BTreeMap<String, (usize, Ast, Vec<String>)>>,
}

pub struct Graph {}

pub fn normalize(ast: &Ast) -> Result<Ast, String> {
	let mut parse = Parse {
		envs: vec![BTreeMap::new()], // 0th un-used, parent 0 used to mean "no-parent"
	};
	parse.program(ast)
}

// impl Parse<'_> {
impl Parse {
	fn walk(&mut self, ast: &Ast, parent: usize) -> Result<Ast, String> {
		match ast {
			Ast::Graph(points) => {
				let index = self.envs.len();
				let mut env = BTreeMap::new();
				env.insert(
					"<parent>".to_string(),
					(parent, Ast::Nothing, vec![]),
				);

				self.envs.push(env);

				let mut normal_points = vec![];
				for point in points {
					normal_points.push(self.walk(point, index)?)
				}

				Ok(Ast::Graph(normal_points))
			}

			Ast::Op2(name, left, right) => {
				// let left = *left;
				match name {
					Name::Label => {
						let label = match *left.clone() {
							Ast::Word(l) => l,
							x => return Err(format!("Invalid label ${:?}", x)),
						};
						let right = self.walk(right, parent)?;
						self.envs[parent].insert(
							label.clone(),
							(parent, right.clone(), vec![]),
						);
						dbg!(self.envs[parent].get(&label));
						Ok(Ast::Op2(*name, left.clone(), Box::new(right)))
					}
					_ => {
						let left = self.walk(left, parent)?;
						let right = self.walk(right, parent)?;
						Ok(Ast::Op2(*name, Box::new(left), Box::new(right)))
					}
				}
				// Err(format!("No match defined for Op2 {:?}", name))
			}

			Ast::Op1(name, operand) => {
				Err(format!("No match defined for Op1 {:?}", name))
			}

			Ast::Space(sizes) => {
				Err(format!("No match defined for Space {:?}", sizes))
			}

			Ast::Integer(i) => Ok(Ast::Integer(i.clone())),

			x => Err(format!("No match defined for {:?}", x)),
		}
	}

	// fn op2(&mut self, ast: &Ast) -> Result<Ast, String> {

	// }

	fn program(&mut self, ast: &Ast) -> Result<Ast, String> {
		self.walk(ast, 0)
	}

	fn lookup(
		&self,
		label: String,
		env: usize,
	) -> Result<&(usize, Ast, Vec<String>), String> {
		if env == 0 {
			return Err(format!("Point {} is undefined", &label));
		}
		match self.envs[env].get(&label) {
			Some(result) => Ok(result),
			None => self.lookup(
				label,
				self.envs[env].get(&"<parent>".to_string()).unwrap().0,
			),
		}
	}
}
