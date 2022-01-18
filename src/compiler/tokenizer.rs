use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
	Invalid,
	Skip,
	Stop,
	//
	// Colon,
	Binary,
	Unary,
	Select,
	Range,
	//
	Label,
	Reserved,
	//
	String,
	Number,
	//
	Paren,
	Squaren,
	Bracket,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Name {
	Invalid,
	Skip,
	Newline,
	Comma,
	//
	Pattern,
	//
	Or,
	And,
	Add,
	Sub,
	Mul,
	Div,
	Exp,
	Not,
	//
	Eq,
	Ne,
	Gt,
	Lt,
	Ge,
	Le,
	//
	Semicolon,
	Colon,
	Length,
	//
	Select,
	// Parent, // shouldn't be needed
	Range,
	//
	Key,
	Ref,
	Label,
	Arrow,
	Reserved,
	//
	String,
	Integer,
	Decimal,
	Boolean,
	//
	ParenLF,
	ParenRT,
	SquarenLF,
	SquarenRT,
	BracketLF,
	BracketRT,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Meta {
	pub line: u32,
	pub text: String,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Of {
	pub kind: Kind,
	pub name: Name,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
	pub of: Of,
	pub meta: Meta,
}

pub fn tokenizer(input: &String) -> Vec<Token> {
	lazy_static! {
		static ref SPEC: Vec<(Kind, Name, Regex)> =
			vec![
				(Kind::Stop, Name::Newline, Regex::new(r"^\n").unwrap()),
				(Kind::Skip, Name::Skip, Regex::new(r"^\r").unwrap()),
				(Kind::Stop, Name::Comma, Regex::new(r"^,").unwrap()),
				(Kind::Skip, Name::Skip, Regex::new(r"^[[:blank:]]+").unwrap()),

				// Comments
				(Kind::Skip, Name::Skip, Regex::new(r"^//.*").unwrap()),
				(Kind::Skip, Name::Skip, Regex::new(r"^/\*[\s\S]*?\*/").unwrap()),

				// Labels
				(Kind::Label, Name::Label, Regex::new(r"^[A-Za-z'_][A-Za-z0-9'_]*").unwrap()),
				(Kind::Label, Name::Arrow, Regex::new(r"^(->|→)").unwrap()),

				// Numbers
				(Kind::Binary, Name::Range, Regex::new(r"^\.\.").unwrap()),

				(Kind::Number, Name::Decimal, Regex::new(r"^[0-9']+\.[0-9']+").unwrap()),
				(Kind::Number, Name::Integer, Regex::new(r"^[0-9']+").unwrap()),
				(Kind::Number, Name::Boolean, Regex::new(r"^(false|true)\b").unwrap()),

				// Reserved Words
				(Kind::Reserved, Name::Reserved, Regex::new(r"^if\b").unwrap()),
				(Kind::Reserved, Name::Reserved, Regex::new(r"^else\b").unwrap()),

				// Operators
				(Kind::Binary, Name::Semicolon, Regex::new(r"^;").unwrap()),
				(Kind::Binary, Name::Colon, Regex::new(r"^:").unwrap()),
				(Kind::Binary, Name::Pattern, Regex::new(r"^[~]").unwrap()),
				(Kind::Binary, Name::Or, Regex::new(r"^[|]").unwrap()),
				(Kind::Binary, Name::And, Regex::new(r"^[&]").unwrap()),
				(Kind::Binary, Name::Eq, Regex::new(r"^((==)|(=))").unwrap()),
				(Kind::Binary, Name::Ne, Regex::new(r"^(!=)").unwrap()),
				(Kind::Binary, Name::Ge, Regex::new(r"^(>=)").unwrap()),
				(Kind::Binary, Name::Le, Regex::new(r"^(<=)").unwrap()),
				(Kind::Binary, Name::Gt, Regex::new(r"^(>)").unwrap()),
				(Kind::Binary, Name::Lt, Regex::new(r"^(<)").unwrap()),
				(Kind::Binary, Name::Add, Regex::new(r"^(\+)").unwrap()),
				(Kind::Binary, Name::Sub, Regex::new(r"^(-)").unwrap()),
				(Kind::Binary, Name::Mul, Regex::new(r"^(\*)").unwrap()),
				(Kind::Binary, Name::Div, Regex::new(r"^(/)").unwrap()),
				(Kind::Binary, Name::Exp, Regex::new(r"^(\^)").unwrap()),

				(Kind::Unary,  Name::Not, Regex::new(r"^(!)").unwrap()),
				(Kind::Unary,  Name::Length, Regex::new(r"^#").unwrap()),


				(Kind::Binary, Name::Select, Regex::new(r"^[.]").unwrap()),


				// parens
				(Kind::Paren, Name::ParenLF, Regex::new(r"^\(").unwrap()),
				(Kind::Paren, Name::ParenRT, Regex::new(r"^\)").unwrap()),

				(Kind::Squaren, Name::SquarenLF, Regex::new(r"^\[").unwrap()),
				(Kind::Squaren, Name::SquarenRT, Regex::new(r"^\]").unwrap()),

				(Kind::Bracket, Name::BracketLF, Regex::new(r"^\{").unwrap()),
				(Kind::Bracket, Name::BracketRT, Regex::new(r"^\}").unwrap()),


				(Kind::String, Name::String, Regex::new(r#"^"[^"]*("|$)"#).unwrap()),



				(Kind::Invalid, Name::Invalid, Regex::new(r"^.").unwrap()),
			];
	}

	let mut tokens: Vec<Token> = Vec::new();
	let mut cursor = 0;
	let mut line = 1;
	let length = input.len();

	let mut skip_initial_newlines = true;
	let mut last_token_was_newline = false;
	let mut last_token_was_comma = false;
	let mut last_token_was_operator = false;

	'outer: while cursor < length {
		for (kind, name, re) in &SPEC[..] {
			match re.find(&input[cursor..]) {
				Some(mat) => {
					let token_text = &input[cursor..cursor + mat.end()];

					let t = Token {
						of: Of {
							kind: *kind,
							name: *name,
						},
						meta: Meta {
							line,
							text: token_text.to_string(),
						},
					};

					match (kind, name) {
						(Kind::Skip, _) => {}
						(Kind::Stop, Name::Comma) => {
							if last_token_was_newline {
								tokens.pop();
							}
							if !last_token_was_comma {
								tokens.push(t);
								last_token_was_operator = false;
								last_token_was_comma = true;
								last_token_was_newline = false;
							}
						}
						(Kind::Stop, Name::Newline) => {
							if !last_token_was_operator
								&& !last_token_was_comma && !last_token_was_newline
								&& !skip_initial_newlines
							{
								tokens.push(t);
								last_token_was_operator = false;
								last_token_was_comma = false;
								last_token_was_newline = true;
							}
							line += 1;
						}

						(Kind::Paren, Name::ParenLF)
						| (Kind::Squaren, Name::SquarenLF)
						| (Kind::Bracket, Name::BracketLF) => {
							tokens.push(t);
							last_token_was_operator = true;
							last_token_was_comma = false;
							last_token_was_newline = false;
						}

						(Kind::Paren, Name::ParenRT)
						| (Kind::Squaren, Name::SquarenRT)
						| (Kind::Bracket, Name::BracketRT) => {
							if last_token_was_newline {
								tokens.pop();
							}
							tokens.push(t);
							last_token_was_operator = false;
							last_token_was_comma = false;
							last_token_was_newline = false;
						}

						(Kind::Binary, _) | (Kind::Label, Name::Arrow) => {
							if last_token_was_newline {
								tokens.pop();
							}
							tokens.push(t);
							last_token_was_operator = true;
							last_token_was_comma = false;
							last_token_was_newline = false;
						}
						_ => {
							tokens.push(t);

							last_token_was_operator = false;
							last_token_was_comma = false;
							last_token_was_newline = false;
							skip_initial_newlines = false;
						}
					}

					cursor += mat.end();
					continue 'outer;
				}
				None => {}
			}
		}
	}
	tokens
}
