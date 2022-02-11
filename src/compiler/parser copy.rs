// tokens -> ast-vector -> btree-graph
/**
 * TODO:
 * --	change AST to an array, and make nested references indexes into this array
 * 	this negates the need for boxed values, which are a mess
 *
 *
 *
 */
use super::tokenizer::{Kind, Name, Token};
use std::cell::RefCell;

pub type EID = usize;
#[derive(Debug, Clone)]
pub enum Ast {
	Nothing,
	Parent(EID),
	//
	Integer(String),
	Decimal(String),
	String(String),
	Clock(String, Name),
	Size(Name),

	Graph(EID, Vec<Ast>), // { .. }
	Space(Vec<Ast>),      // [ .. ]

	Apply(Box<Ast>, Box<Ast>), //	graph { .. }

	Key(String, Box<Ast>), // word: exp
	// Ret(Box<Ast>),                 // -> exp
	Ref(EID, String),              // word
	Op2(Name, Box<Ast>, Box<Ast>), // 1 + 2
	Op1(Name, Box<Ast>),           // - 10
	Op0(Name),                     // i32
}
pub struct Tokens<'a> {
	cursor: RefCell<usize>,
	tokens: &'a Vec<Token>,
}

type Rast = Result<Ast, String>;
type Rasts = Result<Vec<Ast>, String>;

pub fn parser(tokens: &Vec<Token>) -> Rast {
	let cursor = Tokens {
		cursor: RefCell::new(0),
		tokens,
	};

	cursor.program()
}

fn strip_keys(ast: &Ast) -> Ast {
	match ast {
		Ast::Key(_, point) => *point.clone(),
		x => x.clone(),
	}
}

fn flatten_key(top: bool, point: &Ast, flat_points: &mut Vec<Ast>) {
	match point {
		Ast::Key(label, point) => {
			flat_points
				.push(Ast::Key(label.clone(), Box::new(strip_keys(&point))));

			flatten_key(false, &point, flat_points);
		}
		Ast::Ref(_, label) => {
			if top {
				flat_points.push(Ast::Key(
					label.clone(),
					Box::new(Ast::Ref(0, label.clone())),
				));
			}
		}
		// Ast::Ret(point) => {
		// 	flat_points.push(Ast::Ret(Box::new(strip_keys(&point))));

		// 	flatten_key(false, &point, flat_points);
		// }
		_ => {
			if top {
				flat_points.push(point.clone())
			}
		}
	}
}
fn flatten_keys(points: Vec<Ast>, flat_points: &mut Vec<Ast>) {
	for point in points {
		flatten_key(true, &point, flat_points)
	}
}

impl Tokens<'_> {
	fn program(&self) -> Rast {
		Ok(Ast::Graph(0, self.points(false, &[])?))
	}

	fn points(&self, op2: bool, stops: &[Name]) -> Rasts {
		let mut points: Vec<Ast> = vec![];
		self.clear_stops();
		while self.until(0, stops) {
			if op2 {
				points.push(self.return_exp()?);
			} else {
				if self.any(0, &[Name::Key, Name::Las, Name::Arrow]) {
					points.push(self.point_exp()?);
				} else {
					points.push(self.pattern_exp()?);
				}
			}
			self.clear_stops();
		}

		let mut flat_points = vec![];
		flatten_keys(points, &mut flat_points);

		Ok(flat_points)
	}

	fn return_exp(&self) -> Rast {
		let mut left = self.apply()?;
		if self.is(0, Name::Arrow) {
			self.eat(Name::Arrow)?;
			left = Ast::Op2(
				Name::Arrow,
				Box::new(left),
				Box::new(self.sizer_exp()?),
			);
		}

		Ok(left)
	}

	fn point_exp(&self) -> Rast {
		if self.is(0, Name::Las) {
			let key = &self.eat(Name::Las)?.meta.text;
			let label = key[..key.len() - 1].to_string().clone();

			return Ok(Ast::Key(label, Box::new(Ast::Nothing)));
		}
		if self.is(0, Name::Arrow) {
			let _ = self.eat(Name::Arrow)?;
			// return Ok(Ast::Ret(Box::new(self.sizer_exp()?)));
			return Ok(Ast::Key(
				"<return>".to_string(),
				Box::new(self.sizer_exp()?),
			));
		}
		if self.is(0, Name::Key) {
			let key = &self.eat(Name::Key)?.meta.text;
			let label = key[..key.len() - 1].to_string().clone();

			return Ok(Ast::Key(label, Box::new(self.point_exp()?)));
		}
		self.sizer_exp()
	}

	fn pattern_exp(&self) -> Rast {
		let mut left = self.sizer_exp()?;
		while self.is(0, Name::Pattern) {
			self.eat(Name::Pattern)?;
			left = Ast::Op2(
				Name::Pattern,
				Box::new(left),
				Box::new(self.sizer_exp()?),
			);
		}

		Ok(left)
	}

	fn sizer_exp(&self) -> Rast {
		let mut left = self.signal_exp()?;
		if self.is(0, Name::Sizer) {
			self.eat(Name::Sizer)?;
			left = Ast::Op2(
				Name::Sizer,
				Box::new(left),
				Box::new(self.signal_exp()?),
			);
		}

		Ok(left)
	}

	fn signal_exp(&self) -> Rast {
		let mut left = self.join_exp()?;
		while self.is(0, Name::Signal) {
			self.eat(Name::Signal)?;
			left = Ast::Op2(
				Name::Signal,
				Box::new(left),
				Box::new(self.join_exp()?),
			);
		}

		Ok(left)
	}

	fn join_exp(&self) -> Rast {
		let mut left = self.shape_exp()?;
		while self.is(0, Name::Bleed) {
			self.eat(Name::Bleed)?;
			left = Ast::Op2(
				Name::Bleed,
				Box::new(left),
				Box::new(self.shape_exp()?),
			);
		}

		Ok(left)
	}

	fn shape_exp(&self) -> Rast {
		let mut left = self.or_exp()?;
		while self.is(0, Name::Shape) {
			self.eat(Name::Shape)?;
			left = Ast::Op2(
				Name::Shape,
				Box::new(left),
				Box::new(self.or_exp()?),
			);
		}

		Ok(left)
	}

	fn or_exp(&self) -> Rast {
		let mut left = self.and_exp()?;
		while self.is(0, Name::Or) {
			self.eat(Name::Or)?;
			left =
				Ast::Op2(Name::Or, Box::new(left), Box::new(self.and_exp()?));
		}

		Ok(left)
	}

	fn and_exp(&self) -> Rast {
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

	fn equality_exp(&self) -> Rast {
		let mut left = self.relation_exp()?;
		while self.any(0, &[Name::Eq, Name::Ne]) {
			let t = self.eat_of(Kind::Operator)?;
			left = Ast::Op2(
				t.of.name,
				Box::new(left),
				Box::new(self.relation_exp()?),
			);
		}

		Ok(left)
	}

	fn relation_exp(&self) -> Rast {
		let mut left = self.additive_exp()?;
		while self.any(0, &[Name::Gt, Name::Ge, Name::Lt, Name::Le]) {
			let t = self.eat_of(Kind::Operator)?;
			left = Ast::Op2(
				t.of.name,
				Box::new(left),
				Box::new(self.additive_exp()?),
			);
		}

		Ok(left)
	}

	fn additive_exp(&self) -> Rast {
		let mut left = self.multiplicative_exp()?;
		while self.any(0, &[Name::Add, Name::Sub]) {
			let t = self.eat_of(Kind::Operator)?;
			left = Ast::Op2(
				t.of.name,
				Box::new(left),
				Box::new(self.multiplicative_exp()?),
			);
		}

		Ok(left)
	}

	fn multiplicative_exp(&self) -> Rast {
		let mut left = self.exponential_exp()?;
		while self.any(0, &[Name::Mul, Name::Div]) {
			let t = self.eat_of(Kind::Operator).unwrap();
			left = Ast::Op2(
				t.of.name,
				Box::new(left),
				Box::new(self.exponential_exp()?),
			);
		}

		Ok(left)
	}

	fn exponential_exp(&self) -> Rast {
		let mut left = self.unary_exp()?;
		while self.is(0, Name::Exp) {
			self.eat(Name::Exp)?;
			left = Ast::Op2(
				Name::Exp,
				Box::new(left),
				Box::new(self.unary_exp()?),
			);
		}

		Ok(left)
	}

	fn unary_exp(&self) -> Rast {
		if self.of(0, Kind::Operator) {
			let operator = self.eat_of(Kind::Operator)?;
			Ok(Ast::Op1(operator.of.name, Box::new(self.unary_exp()?)))
		} else {
			self.select_exp()
		}
	}

	fn select_exp(&self) -> Rast {
		let mut left = self.apply()?;
		while self.any(0, &[Name::Select, Name::Index]) {
			let operator = self.eats(&[Name::Select, Name::Index])?;
			left = Ast::Op2(
				operator.of.name,
				Box::new(left),
				Box::new(self.apply()?),
			);
		}

		Ok(left)
	}

	fn apply(&self) -> Rast {
		let mut left = self.primary()?;
		if self.any(0, &[Name::BracketLF, Name::ParenLF]) {
			left = Ast::Apply(Box::new(left), Box::new(self.primary()?));
		}

		Ok(left)
	}
	fn primary(&self) -> Rast {
		if self.is(0, Name::Ref) {
			self.ref_()
		} else if self.of(0, Kind::Bracket) {
			self.graph_exp(false)
		} else if self.of(0, Kind::Paren) {
			self.paren_exp()
		} else if self.of(0, Kind::Squaren) {
			self.space_exp()
		} else {
			self.literal()
		}
	}

	fn graph_exp(&self, op2: bool) -> Rast {
		self.eat(Name::BracketLF)?;
		let points = self.points(op2, &[Name::BracketRT])?;
		self.eat(Name::BracketRT)?;
		Ok(Ast::Graph(0, points))
	}

	fn paren_exp(&self) -> Rast {
		self.eat(Name::ParenLF)?;
		let exp = self.or_exp();
		self.eat(Name::ParenRT)?;
		exp
	}

	fn space_exp(&self) -> Rast {
		self.eat(Name::SquarenLF)?;
		let points = self.points(false, &[Name::SquarenRT])?;
		self.eat(Name::SquarenRT)?;
		Ok(Ast::Space(points))
	}

	fn ref_(&self) -> Rast {
		let t = self.eat(Name::Ref)?;
		Ok(Ast::Ref(0, t.meta.text.clone()))
	}

	fn literal(&self) -> Rast {
		if self.of(0, Kind::String) {
			self.string()
		} else if self.of(0, Kind::Clock) {
			self.clock()
		} else if self.of(0, Kind::Size) {
			self.size()
		} else {
			self.number()
		}
	}

	fn number(&self) -> Rast {
		let t = self.eat_of(Kind::Number)?;
		let ast = match t.of.name {
			Name::Decimal => Ast::Decimal(t.meta.text.clone()),
			Name::Integer => Ast::Integer(t.meta.text.clone()),
			_ => panic!(),
		};
		Ok(ast)
	}

	fn clock(&self) -> Rast {
		let t = self.eat_of(Kind::Clock)?;
		Ok(Ast::Clock(t.meta.text.clone(), t.of.name))
	}

	fn size(&self) -> Rast {
		let t = self.eat_of(Kind::Size)?;
		Ok(Ast::Size(t.of.name))
	}

	fn string(&self) -> Rast {
		let t = self.eat_of(Kind::String)?;
		Ok(Ast::String(t.meta.text.clone()))
	}
}

//

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
						"UnexpectedToken: {:?} of {:?}:{:?} on line {}\nExpected tokens of names: {:?}",
						t.meta.text, t.of.kind, t.of.name, t.meta.line, names
					))
				};
				*self.cursor.borrow_mut() += 1; // must occur after self.any
				ret
			}
			None => Err("UnexpectedEndOfInput".to_string()),
		}
	}

	fn _eats_of(&self, kinds: &[Kind]) -> Result<&Token, String> {
		match self.get(0) {
			Some(t) => {
				let ret = if self._any_of(0, kinds) {
					Ok(t)
				} else {
					Err(format!(
						"UnexpectedToken: {:?} of {:?}:{:?} on line {}\nExpected tokens of kinds: {:?}",
						t.meta.text, t.of.kind, t.of.name, t.meta.line, kinds
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
	fn _any_of(&self, offset: usize, kinds: &[Kind]) -> bool {
		for kind in kinds {
			if self.of(offset, *kind) {
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

	fn _until_of(&self, offset: usize, stops: &[Kind]) -> bool {
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
