/*
reduce turn ast to sequences

*/

use super::expander::Parse;
use super::parser::Ast;
use std::collections::BTreeMap;

pub fn reducer(
	ast: &Ast,
	envs: &Vec<BTreeMap<String, (usize, Ast, Vec<String>)>>,
) -> Result<Ast, String> {
	let mut parse = Parse { envs: envs.clone() };
	parse.reduce(ast, 0)
}
// #[derive(Debug, Clone)]
// pub struct Parse {
// 	envs: Vec<BTreeMap<String, (usize, Ast, Vec<String>)>>,
// }

impl Parse {
	fn reduce(&mut self, ast: &Ast, parent: usize) -> Result<Ast, String> {
		match ast {
			// Ast::Graph(points) => {
			// 	let mut reduced_points = Vec::new();
			// 	for point in points {
			// 		reduced_points.push(self.reduce(point, parent + 1)?);
			// 	}

			// 	Ok(Ast::Graph(reduced_points))
			// }
			// Ast::Key(label, point) => {
			// 	let point = self.reduce(point, parent)?;
			// 	self.envs[parent]
			// 		.insert(label.clone(), (parent, point.clone(), vec![]));
			// 	Ok(Ast::Key(label.clone(), Box::new(point)))
			// }

			// Ast::Ref(label) => {
			// 	let source = self.lookup(label, parent)?;
			// 	Ok(source.1.clone())
			// }
			// Ast::Op2(name, left, right) => {
			// 	let left = self.reduce(left, parent)?;
			// 	let right = self.reduce(right, parent)?;
			// 	Ok(Ast::Op2(*name, Box::new(left), Box::new(right)))
			// }
			// Ast::Op1(name, operand) => {
			// 	let operand = self.reduce(operand, parent)?;
			// 	Ok(Ast::Op1(*name, Box::new(operand)))
			// }
			// Ast::Op0(name) => Ok(Ast::Op0(*name)),
			// Ast::Space(sizes) => {
			// 	let mut ss = vec![];
			// 	for size in sizes {
			// 		ss.push(self.reduce(size, parent)?)
			// 	}
			// 	Ok(Ast::Space(ss))
			// }

			// Ast::Decimal(i) => Ok(Ast::Decimal(i.clone())),
			// Ast::Integer(i) => Ok(Ast::Integer(i.clone())),
			// Ast::String(i) => Ok(Ast::String(i.clone())),
			// Ast::Clock(t, i) => Ok(Ast::Clock(t.clone(), i.clone())),
			x => return Err(format!("No reduce match defined for {:?}", x)),
		}
	}
}
