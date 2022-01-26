use super::tokenizer::{Kind, Name, Token};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// pub struct Map<'a> {
// 	pub parent: Option<&'a HashMap<String, (&'a Typ, &'a Ast)>>,
// 	pub this: HashMap<String, (&'a Typ, &'a Ast)>,
// }

pub struct Map<'a> {
	pub parent: Option<&'a HashMap<String, Rc<Box<Ast>>>>,
	pub this: HashMap<String, Rc<Box<Ast>>>,
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

#[derive(Debug, Clone)]
pub enum Typ {
	Number(Format),
	String(Format),

	Thing,           // placeholder
	Graph(Vec<Typ>), // a graph that is packed into a single array space, a struct
	Array(Box<Typ>), // packed array of things
}

#[derive(Debug, Clone)]
pub enum Ast {
	Nothing,
	//
	Integer(String),
	Decimal(String),
	String(String),

	Graph(Vec<Ast>),
	Point(String, Typ, u16, bool, Box<Ast>), // Type, Label, Index, isReturn, Value

	Apply(Box<Ast>, Box<Ast>),
	Op2(Name, Box<Ast>, Box<Ast>),
	Op1(Name, Box<Ast>),

	Ref(String),
}

pub struct Parse<'a> {
	pub ast: Ast,
	pub map: Map<'a>,
	pub typ: Typ,
}

pub struct Tokens<'a> {
	cursor: RefCell<usize>,
	tokens: &'a Vec<Token>,
}

pub fn parser(tokens: &Vec<Token>) -> Result<Parse, String> {
	let cursor = Tokens {
		cursor: RefCell::new(0),
		tokens,
	};

	let mut map = Map {
		parent: None,
		this: HashMap::new(),
	};

	let ast = cursor.program()?;
	// let typ = walk(ast, &mut map);
	let typ = Typ::Thing;

	Ok(Parse { ast, map, typ })
	// Err("".to_string())
}

impl Tokens<'_> {
	fn program(&self) -> Result<Ast, String> {
		let ast = self.point_list(&[])?;

		Ok(Ast::Point(
			"Program".to_string(),
			Typ::Thing,
			0,
			false,
			Box::new(ast),
		))
	}

	fn point_list(&self, stops: &[Name]) -> Result<Ast, String> {
		let mut points: Vec<Ast> = vec![];
		self.clear_stops();
		let mut index = 0;
		while self.until(0, stops) {
			points.push(self.point(&mut index)?);
			// if !self.any(0, stops) {
			// self.eat_of(Kind::Stop)?;
			// }
			self.clear_stops();
		}

		Ok(Ast::Graph(points))
	}

	fn point(&self, shared_index: &mut u16) -> Result<Ast, String> {
		let point: Ast;
		let space: Typ;
		let label: String;
		let index = *shared_index;
		let graph: Ast;

		*shared_index += 1;
		if *shared_index > 10 {
			panic!()
		}

		if self.is(0, Name::Label)
			&& self.any(1, &[Name::SquarenLF, Name::Colon, Name::Semicolon])
		{
			label = self.eat(Name::Label)?.meta.text.clone();
			if self.is(0, Name::SquarenLF) {
				space = self.space()?;
				graph = if self.is(0, Name::Colon) {
					self.eat(Name::Colon)?;
					self.or_exp()?
				} else {
					Ast::Nothing
				}
			} else if self.is(0, Name::Semicolon) {
				// label;
				self.eat(Name::Semicolon)?;
				space = Typ::Thing;
				graph = Ast::Nothing;
			} else {
				// label : value
				space = Typ::Thing;
				self.eat(Name::Colon)?;
				graph = if self.is(0, Name::Label)
					&& self.any(1, &[Name::Colon, Name::Semicolon])
				{
					self.point(shared_index)?
				} else {
					self.or_exp()?
				}
			}
		} else {
			space = Typ::Thing;
			label = String::new();
			graph = self.or_exp()?;
		}

		point = Ast::Point(label, space, index, false, Box::new(graph));

		Ok(point)
	}
	fn space(&self) -> Result<Typ, String> {
		dbg!();
		self.eat(Name::SquarenLF)?;
		Ok(Typ::Thing)
	}

	// fn space_list(&self) -> Result<

	fn or_exp(&self) -> Result<Ast, String> {
		let mut left = self.and_exp()?;
		while self.is(0, Name::Or) {
			self.eat(Name::Or)?;
			left =
				Ast::Op2(Name::Or, Box::new(left), Box::new(self.and_exp()?));
		}

		Ok(left)
	}

	fn and_exp(&self) -> Result<Ast, String> {
		let mut left = self.equality_exp()?;
		while self.is(0, Name::And) {
			self.eat(Name::And)?;
			left = Ast::Op2(
				Name::And,
				Box::new(left),
				Box::new(self.equality_exp()?),
			);
		}

		Ok(left)
	}

	fn equality_exp(&self) -> Result<Ast, String> {
		let mut left = self.relation_exp()?;
		while self.any(0, &[Name::Eq, Name::Ne]) {
			let t = self.eat_of(Kind::Binary)?;
			left = Ast::Op2(
				t.of.name,
				Box::new(left),
				Box::new(self.relation_exp()?),
			);
		}

		Ok(left)
	}

	fn relation_exp(&self) -> Result<Ast, String> {
		let mut left = self.additive_exp()?;
		while self.any(0, &[Name::Gt, Name::Ge, Name::Lt, Name::Le]) {
			let t = self.eat_of(Kind::Binary)?;
			left = Ast::Op2(
				t.of.name,
				Box::new(left),
				Box::new(self.additive_exp()?),
			);
		}

		Ok(left)
	}

	fn additive_exp(&self) -> Result<Ast, String> {
		let mut left = self.multiplicative_exp()?;
		while self.any(0, &[Name::Add, Name::Sub]) {
			let t = self.eat_of(Kind::Binary)?;
			left = Ast::Op2(
				t.of.name,
				Box::new(left),
				Box::new(self.multiplicative_exp()?),
			);
		}

		Ok(left)
	}

	fn multiplicative_exp(&self) -> Result<Ast, String> {
		let mut left = self.exponential_exp()?;
		while self.any(0, &[Name::Mul, Name::Div]) {
			let t = self.eat_of(Kind::Binary).unwrap();
			left = Ast::Op2(
				t.of.name,
				Box::new(left),
				Box::new(self.exponential_exp()?),
			);
		}

		Ok(left)
	}

	fn exponential_exp(&self) -> Result<Ast, String> {
		let mut left = self.range_exp()?;
		while self.is(0, Name::Exp) {
			self.eat(Name::Exp)?;
			left = Ast::Op2(
				Name::Exp,
				Box::new(left),
				Box::new(self.range_exp()?),
			);
		}

		Ok(left)
	}

	fn range_exp(&self) -> Result<Ast, String> {
		let mut left = self.unary_exp()?;
		while self.is(0, Name::Range) {
			self.eat(Name::Range)?;
			left = Ast::Op2(
				Name::Range,
				Box::new(left),
				Box::new(self.unary_exp()?),
			);
		}

		Ok(left)
	}

	fn unary_exp(&self) -> Result<Ast, String> {
		if self.any(
			0,
			&[
				Name::Add,
				Name::Sub,
				Name::Not,
				Name::Range,
				Name::Colon,
				Name::Gt,
				Name::Lt,
				Name::Length,
			],
		) {
			let operator = self.eats(&[
				Name::Add,
				Name::Sub,
				Name::Not,
				Name::Range,
				Name::Colon,
				Name::Gt,
				Name::Lt,
				Name::Length,
			])?;
			Ok(Ast::Op1(operator.of.name, Box::new(self.unary_exp()?)))
		} else {
			self.select_exp()
			// Ok(AST::Nothing)
			// Ok(self.literal()?)
		}
	}

	fn select_exp(&self) -> Result<Ast, String> {
		let mut left = self.apply()?;
		while self.is(0, Name::Select) {
			self.eat(Name::Select)?;
			left = Ast::Op2(
				Name::Select,
				Box::new(left),
				Box::new(self.apply()?),
			);
		}

		Ok(left)
	}

	fn apply(&self) -> Result<Ast, String> {
		let mut left = self.primary()?;
		// while self.until_of(0, &[Kind::Stop]) {
		while self.any(0, &[Name::BracketLF, Name::ParenLF]) {
			left = Ast::Apply(Box::new(left), Box::new(self.primary()?));
		}

		Ok(left)
	}

	fn primary(&self) -> Result<Ast, String> {
		if self.is(0, Name::Label) {
			self.reference()
		} else if self.of(0, Kind::Bracket) {
			self.graph_exp()
		} else if self.of(0, Kind::Paren) {
			self.paren_exp()
		} else {
			self.literal()
		}
	}
	fn graph_exp(&self) -> Result<Ast, String> {
		self.eat(Name::BracketLF)?;
		let exp = self.point_list(&[Name::BracketRT]);
		self.eat(Name::BracketRT)?;
		exp
	}
	fn paren_exp(&self) -> Result<Ast, String> {
		self.eat(Name::ParenLF)?;
		let exp = self.or_exp();
		self.eat(Name::ParenRT)?;
		exp
	}
	fn reference(&self) -> Result<Ast, String> {
		let t = self.eat(Name::Label)?;
		Ok(Ast::Ref(t.meta.text.clone()))
	}

	fn literal(&self) -> Result<Ast, String> {
		if self.of(0, Kind::String) {
			self.string()
		} else {
			self.number()
		}
	}

	fn number(&self) -> Result<Ast, String> {
		let t = self.eat_of(Kind::Number)?;
		let ast = match t.of.name {
			Name::Decimal => Ast::Decimal(t.meta.text.clone()),
			Name::Integer => Ast::Integer(t.meta.text.clone()),
			_ => panic!(),
		};
		Ok(ast)
	}

	fn string(&self) -> Result<Ast, String> {
		let t = self.eat_of(Kind::String)?;
		Ok(Ast::String(t.meta.text.clone()))
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
						"UnexpectedToken: {:?} of {:?}:{:?} on line {}\nExpected token of name: {:?}",
						t.meta.text, t.of.kind, t.of.name, t.meta.line, name
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
						"UnexpectedToken: {:?} of {:?}:{:?} on line {}\nExpected token of kind: {:?}",
						t.meta.text, t.of.kind, t.of.name, t.meta.line, kind
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

	fn until_of(&self, offset: usize, stops: &[Kind]) -> bool {
		match self.get(offset) {
			Some(t) => {
				for stop in stops {
					if t.of.kind == *stop {
						return false;
					}
				}
				true
			}
			None => false,
		}
	}
}
