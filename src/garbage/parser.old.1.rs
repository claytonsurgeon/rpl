use super::tokenizer::{Kind, Name, Token};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Tokens<'a> {
	cursor: RefCell<usize>,
	tokens: &'a Vec<Token>,
	namespace: &'a Namespace<'a>,
}

struct Namespace<'a> {
	parent: &'a HashMap<String, (&'a AST<'a>, &'a Namespace<'a>)>,
	item: &'a HashMap<String, (&'a AST<'a>, &'a Namespace<'a>)>,
}

pub fn parser<'a>(
	tokens: &Vec<Token>,
) -> Result<(Type<'a>, AST<'a>), String> {
	let cursor = Tokens {
		cursor: RefCell::new(0),
		tokens,
		namespace: &Namespace {
			parent: &HashMap::new(),
			item: &HashMap::new(),
		},
	};
	Ok(cursor.program()?)
}
#[derive(Debug, Clone)]
pub enum Format {
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

fn name_to_number(name: Name) -> Format {
	match name {
		Name::Decimal => Format::Decimal,
		Name::Integer => Format::Integer,
		Name::Boolean => Format::Boolean,
		_ => panic!("name_to_number should be infallible"),
	}
}

#[derive(Debug, Clone)]
pub enum Type<'a> {
	Number(Format),
	String(Format),

	Thing,                // placeholder
	Graph(Vec<Type<'a>>), // a graph that is packed into a single array space, a struct
	Array(Box<Type<'a>>), // packed array of things
}

#[derive(Debug, Clone)]
pub enum AST<'a> {
	Nothing,
	//
	Program(&'a Type<'a>, Box<AST<'a>>), // maybe only program needs Type
	Number(&'a Type<'a>, String),
	String(&'a Type<'a>, String),

	Graph(&'a Type<'a>, Vec<AST<'a>>),
	Point(&'a Type<'a>, String, u16, bool, Box<AST<'a>>), // Type, Label, Index, isReturn, Value

	Op2(&'a Type<'a>, Name, Box<AST<'a>>, Box<AST<'a>>),
	Op1(&'a Type<'a>, Name, Box<AST<'a>>),
	// Ref(Type, String),
	// Arg(Box<AST>),
	// Rep(Box<AST>, Box<AST>),
	//
}

impl Tokens<'_> {
	fn program<'a>(&self) -> Result<(Type<'a>, AST), String> {
		let namespace = &Namespace {
			parent: self.namespace.item,
			item: &HashMap::new(),
		};
		let (tys, points) = self.point_list(namespace, &[])?;
		let point = AST::Point(
			&tys,
			String::from("Program"), // should be file name
			0,
			false,
			Box::new(AST::Graph(&tys, points)),
		);

		self
			.namespace
			.item
			.insert(String::from("Program"), (&point, namespace));
		Ok((tys, point))
	}

	fn point_list<'a>(
		&self,
		namespace: &Namespace,
		stops: &[Name],
	) -> Result<(Type, Vec<AST>), String> {
		let mut points: Vec<AST> = vec![];
		let mut tys: Vec<Type> = vec![];

		self.clear_stops();
		let mut index = 0;
		while self.until(0, stops) {
			// index += 1;
			points.push(self.point(namespace, &mut index)?);
			self.clear_stops();
		}

		// Ok(points)
		Ok((Type::Graph(tys), points))
	}

	fn point(
		&self,
		namespace: &Namespace,
		index: &mut u16,
	) -> Result<AST, String> {
		Ok(self.graph(namespace, index)?)
	}

	fn graph(
		&self,
		namespace: &Namespace,
		index: &mut u16,
	) -> Result<AST, String> {
		let point: AST;
		let this_index = *index;

		*index += 1;

		if self.is(0, Name::Label)
			&& self.any(1, &[Name::SquarenLF, Name::Colon, Name::Semicolon])
		{
			let label = self.eat(Name::Label)?;
			if self.is(0, Name::SquarenLF) {
				let space = self.space()?;
				if self.is(0, Name::Colon) {
					self.eat(Name::Colon)?;
					let or_graph = self.or_graph()?;
				}
			} else if self.is(0, Name::Semicolon) { // label;
			} else {
				// label : value
				self.eat(Name::Colon)?;
				let graph = self.graph(namespace, index)?;
			}
		} else {
			let (t, or_graph) = self.or_graph()?;
			point = AST::Point(t)
		}

		Ok(AST::Nothing)
	}

	fn space(&self) -> Result<AST, String> {
		self.eat(Name::SquarenLF)?;
		Ok(AST::Nothing)
	}

	fn or_graph(&self) -> Result<(Type, AST), String> {
		Ok((Type::Thing, AST::Nothing))
	}
}

impl Tokens<'_> {
	fn eat(&self, name: Name) -> Result<&Token, String> {
		match self.get(0) {
			Some(t) => {
				*self.cursor.borrow_mut() += 1;
				if t.of.name == name {
					Ok(t)
				} else {
					Err(format!(
						"UnexpectedToken: {:?} on line {}\nExpected token of name: {:?}",
						t.meta.text, t.meta.line, t.of.name
					))
				}
			}
			None => Err("UnexpectedEndOfInput".to_string()),
		}
	}
	fn eat_of(&self, kind: Kind) -> Result<&Token, String> {
		match self.get(0) {
			Some(t) => {
				*self.cursor.borrow_mut() += 1;
				if t.of.kind == kind {
					Ok(t)
				} else {
					Err(format!(
						"UnexpectedToken: {:?} on line {}\nExpected token of kind: {:?}",
						t.meta.text, t.meta.line, t.of.kind
					))
				}
			}
			None => Err("UnexpectedEndOfInput".to_string()),
		}
	}

	fn eats(&self, names: &[Name]) -> Result<&Token, String> {
		match self.get(0) {
			Some(t) => {
				let ret = if self.any(0, names) {
					Ok(t)
				} else {
					Err(format!(
						"UnexpectedToken: {:?} on line {}\nExpected token of name: {:?}",
						t.meta.text, t.meta.line, t.of.name
					))
				};
				*self.cursor.borrow_mut() += 1; // must occur after self.any
				ret
			}
			None => Err("UnexpectedEndOfInput".to_string()),
		}
	}

	fn clear_stops(&self) {
		while self.of(0, Kind::Stop) {
			*self.cursor.borrow_mut() += 1;
		}
	}

	fn get(&self, offset: usize) -> Option<&Token> {
		if *self.cursor.borrow() + offset < self.tokens.len() {
			Some(&self.tokens[*self.cursor.borrow() + offset])
		} else {
			None
		}
	}

	fn is(&self, offset: usize, stop: Name) -> bool {
		match self.get(offset) {
			Some(t) => t.of.name == stop,
			None => false,
		}
	}

	fn of(&self, offset: usize, stop: Kind) -> bool {
		match self.get(offset) {
			Some(t) => t.of.kind == stop,
			None => false,
		}
	}
	fn any(&self, offset: usize, names: &[Name]) -> bool {
		for name in names {
			if self.is(offset, *name) {
				return true;
			}
		}
		false
	}

	fn until(&self, offset: usize, stops: &[Name]) -> bool {
		match self.get(offset) {
			Some(t) => {
				for stop in stops {
					if t.of.name == *stop {
						return false;
					}
				}
				true
			}
			None => false,
		}
	}
}
