/*
reduce turn ast to sequences


a code point is either an fundemental operator (add, mul, etc) or a server
a server communicates with other programs in a system to perform work. RPL has no Foriegn function interface.


	graph is a function that allocates space, the pulls data corresponding to the points it contains
	it is just a series of references to its points


	therefore the code of a graph is all its references
	the the data of a graph is combined result, in the case of a function-like graph then only the return value is stored


! takes a graph and turns it into a packed array
array: [100 i32]! {1, 2, 3}


*/

use super::expander::Maps;
use super::parser::{Ast, IDX};
use super::tokenizer::Name;
use std::collections::BTreeMap;

pub struct Program {
	data: Vec<i32>,
	code: Vec<Point>,
	push: Vec<i32>,
}

#[derive(Debug, Clone)]
pub enum Point {
	Dummy,
	Error(String),
	Graph(Vec<IDX>),
	Integer(i64),
	Add(IDX, IDX, IDX),
}

pub fn reducer(ast: &Ast, envs: &Maps) -> Result<IDX, String> {
	let mut parse = Program {
		// don't actually need code AND data, because in this system they are the same thing
		data: Vec::new(),
		code: Vec::new(),
		push: Vec::new(),
	};
	let r = parse.reduce(ast, 0);

	dbg!(parse.code);

	return r;
}

impl Program {
	fn reduce(&mut self, ast: &Ast, parent: usize) -> Result<IDX, String> {
		match ast {
			Ast::Graph(eid, points) => {
				let mut idxs = Vec::new();
				// let idx = self.code.len();
				// self.code.push(Point::Dummy); // claim spot in array
				for point in points {
					idxs.push(self.reduce(point, *eid)?);
				}
				// self.code[idx] = Point::Graph(idxs); // replace dummy
				let idx = self.code.len();
				self.code.push(Point::Graph(idxs));
				Ok(idx)
			}

			Ast::Key(_, point) => self.reduce(point, parent),

			Ast::Integer(value) => {
				let v: i64 = value.parse().unwrap();
				let idx = self.code.len();
				self.code.push(Point::Integer(v));
				Ok(idx)
			}

			Ast::Op2(name, point_a, point_b) => {
				let a = self.reduce(point_a, parent)?;
				let b = self.reduce(point_b, parent)?;
				let c_point = execute2(*name, &self.code[a], &self.code[b]);
				let c = self.code.len();
				self.code.push(c_point);
				let idx = self.code.len();
				self.code.push(Point::Add(a, b, c));
				Ok(idx)
			}
			x => return Err(format!("No reduce match defined for {:?}", x)),
		}
	}
}

fn execute2(name: Name, a: &Point, b: &Point) -> Point {
	match name {
		Name::Add => add(a, b),
		_ => Point::Error(format!("fug")),
	}
}

fn add(a: &Point, b: &Point) -> Point {
	use Point::{Error, Integer};
	match (a, b) {
		(Integer(a), Integer(b)) => Integer(a + b),
		_ => Error(format!("fug")),
	}
}
