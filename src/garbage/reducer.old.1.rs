/*
memory:
data memory is static in size

the size of a point is the sum of its graph. A graph is a series of connected points

execution memory space is implemetation defined, this may be done via a stack or register machine

a: 10

graph(number(i32), [
		Op2(number(Integer), Label
			Word(thing, "a"),
			Integer(10),
		)
])


*/

use super::parser::Ast;
// use super::tokenizer::Name;
use std::collections::BTreeMap;

pub struct Map<'a> {
	pub parent: Option<&'a BTreeMap<String, (u64, Ast)>>,
	pub this: BTreeMap<String, (u64, Ast)>,
}

pub struct Parse<'a> {
	ast: &'a Ast,
	map: Map<'a>,
}

#[derive(Debug, Clone)]
pub enum Number {
	// ie how are the bytes formatted, what standard
	Integer,
	Decimal,
	Boolean,

	// ints
	I8,
	I16,
	I32,
	I64,
	I128,

	// uints
	U8,
	U16,
	U32,
	U64,
	U128,

	// floats
	F32,
	F64,
	F128,
}
#[derive(Debug, Clone)]
pub enum Char {
	C8, // UTF-8, but only chars of 8 bits like ascii
	C16,
	C32, // Full UTF-8
}

#[derive(Debug, Clone)]
pub enum Typ {
	Nothing,
	Number(Number),
	String(Char),

	Graph(Vec<Typ>),
}

pub struct Graph {}

pub fn reducer(ast: &Ast) -> Result<Graph, String> {
	let parse = Parse {
		ast,
		map: Map {
			parent: None,
			this: BTreeMap::new(),
		},
	};
	parse.program()
}

impl Parse<'_> {
	fn program(&self) -> Result<Graph, String> {
		match self.ast {
			Ast::Graph(g) => Err("hello there from reducer".to_string()),
			_ => Err("hello there from reducer".to_string()),
		}
	}
}
