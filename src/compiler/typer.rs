use super::expander::Maps;
use super::parser;
use super::tokenizer::Name;
// use std::collections::BTreeMap;

pub type Fast = Vec<Ast>;
pub type AstIndex = usize;

pub fn typer(
	ast: &parser::Ast,
	envs: Maps,
) -> Result<(Fast, Maps), String> {
	// let mut parse = Parse { envs: envs.clone() };
	// parse.reduce(ast, 0)
	Ok((vec![Ast::Nothing], envs))
}
