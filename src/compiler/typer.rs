use super::expander::Maps;
use super::parser;
use super::tokenizer::Name;
// use std::collections::BTreeMap;

pub type Fast = Vec<Ast>;
pub type AstIndex = usize;

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

	// char
	C8, // UTF-8, but only chars of 8 bits like ascii
	C16,
	C32, // Full UTF-8
}

#[derive(Debug, Clone)]
pub enum Ast {
	Nothing,

	Graph(Vec<AstIndex>), // { .. }
	Space(Vec<AstIndex>), // [ .. ]

	Key(String, AstIndex, AstIndex), // word: exp

	Ref(String),
	Op2(Name, AstIndex, AstIndex),
	Op1(Name, AstIndex),
	// Op0(Name),
	String(String),
	Clock(i64, Name),
	//
	Bool(bool),
	//
	I8(i8),
	I16(i16),
	I32(i32),
	I64(i64),
	I128(i128),

	// uints
	U8(u8),
	U16(u16),
	U32(u16),
	U64(u64),
	U128(u128),
	F32(f32),
	F64(f64),
	// F128(f128),
}

pub fn typer(
	ast: &parser::Ast,
	envs: Maps,
) -> Result<(Fast, Maps), String> {
	// let mut parse = Parse { envs: envs.clone() };
	// parse.reduce(ast, 0)
	Ok((vec![Ast::Nothing], envs))
}
