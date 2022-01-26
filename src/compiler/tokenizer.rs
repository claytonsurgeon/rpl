use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
	Invalid,
	Skip,
	Stop,
	Operator,
	//
	Word,
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
	Skip,      // 	comments, spaces, \r
	Newline,   //	\n
	Comma,     //	,
	Semicolon, //	;
	//								unadic,				dyadic
	Pattern, // 	~			n/a					pattern
	Signal, // 		?			make-signal 		gated-expression					(think !!), 	controls reativity explicitly
	Sizer,  //		!			capacity				set capacity
	Label,  // 		:			symbol				label expression
	Or,     // 		|			bitwise OR			bitwise OR
	And,    // 		&			bitwise AND			bitwise AND
	Not, // 			`			bitwise negate		bitmask filter						regular bitwise negate, filter arrays with bitmask
	Add, // 			+			magnitude			add
	Sub, // 			-			negate				subtract
	Mul, // 			*			n/a					multiply
	Div, // 			/			invert				divide								10 + /10 = 10.1
	Exp, // 			^			n/a					exponentiate
	//
	Eq, // 			=			n/a					bitwise equality
	Ne, // 			!=			n/a					bitwise inequality
	Gt, // 			>			inf ascending		greater than 						(>7) => 8,9,10,11,12...
	Lt, // 			<			iota ascending		less than							(<7) => 0,1,2,3,4,5,6
	Ge, // 			>=			inf ascending		greater than or equal			(>=7) => 7,8,9,10,11,12...
	Le, // 			<= 		iota ascending		less than or equal				(<=7) =>0,1,2,3,4,5,6,7
	//
	Shape, // 		$			get-shape			reshape
	Index, // 		#			length/tally		select by index					#array = length,	array#[10] = element of array
	//
	Select, //		.			n/a					select by label
	Bleed,  //		..			bleed into			join									arraya:{..arrayb},		arraya .. arrayb
	//
	Arrow, //		->		   return				match return

	Word,
	Reserved,
	//
	String,
	//
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

pub fn tokenizer(input: &String) -> Result<Vec<Token>, String> {
	lazy_static! {
		static ref SPEC: Vec<(Kind, Name, Regex)> =
			vec![
				// insignificant whitespace
				(Kind::Skip, Name::Skip, Regex::new(r"^\r").unwrap()),
				(Kind::Skip, Name::Skip, Regex::new(r"^[[:blank:]]+").unwrap()),

				// Comments
				(Kind::Skip, Name::Skip, Regex::new(r"^//.*").unwrap()),
				// ** messes up line number counting
				(Kind::Skip, Name::Skip, Regex::new(r"^/\*[\s\S]*?\*/").unwrap()),

				(Kind::Stop, Name::Newline, Regex::new(r"^\n").unwrap()),
				(Kind::Stop, Name::Comma, Regex::new(r"^,").unwrap()),

				// Words
				(Kind::Word, Name::Word, Regex::new(r"^[A-Za-z'_][A-Za-z0-9'_]*").unwrap()),
				// Reserved Words
				(Kind::Word, Name::Reserved, Regex::new(r"^if\b").unwrap()),
				(Kind::Word, Name::Reserved, Regex::new(r"^else\b").unwrap()),

				// Numbers
				(Kind::Number, Name::Decimal, Regex::new(r"^[0-9]+\.[0-9]*").unwrap()),
				(Kind::Number, Name::Decimal, Regex::new(r"^[0-9]*\.[0-9]+").unwrap()),
				(Kind::Number, Name::Integer, Regex::new(r"^[0-9']+").unwrap()),
				(Kind::Number, Name::Boolean, Regex::new(r"^(false|true)\b").unwrap()),

				// Operators
				(Kind::Operator, Name::Semicolon, Regex::new(r"^;").unwrap()),
				//
				(Kind::Operator, Name::Arrow, Regex::new(r"^(->|→)").unwrap()),
				(Kind::Operator, Name::Bleed, Regex::new(r"^\.\.").unwrap()),
				(Kind::Operator, Name::Select, Regex::new(r"^[.]").unwrap()),

				(Kind::Operator, Name::Signal, Regex::new(r"^\?").unwrap()),
				(Kind::Operator, Name::Label, Regex::new(r"^:").unwrap()),
				(Kind::Operator, Name::Sizer, Regex::new(r"^!").unwrap()),
				(Kind::Operator, Name::Shape, Regex::new(r"^$").unwrap()),
				(Kind::Operator,  Name::Index, Regex::new(r"^#").unwrap()),

				(Kind::Operator, Name::Or, Regex::new(r"^[|]").unwrap()),
				(Kind::Operator, Name::And, Regex::new(r"^[&]").unwrap()),
				(Kind::Operator, Name::Not, Regex::new(r"^[`]").unwrap()),
				(Kind::Operator, Name::Eq, Regex::new(r"^((==)|(=))").unwrap()),
				(Kind::Operator, Name::Ne, Regex::new(r"^(!=)").unwrap()),
				(Kind::Operator, Name::Ge, Regex::new(r"^(>=)").unwrap()),
				(Kind::Operator, Name::Le, Regex::new(r"^(<=)").unwrap()),
				(Kind::Operator, Name::Gt, Regex::new(r"^(>)").unwrap()),
				(Kind::Operator, Name::Lt, Regex::new(r"^(<)").unwrap()),
				(Kind::Operator, Name::Add, Regex::new(r"^(\+)").unwrap()),
				(Kind::Operator, Name::Sub, Regex::new(r"^(-)").unwrap()),
				(Kind::Operator, Name::Mul, Regex::new(r"^(\*)").unwrap()),
				(Kind::Operator, Name::Div, Regex::new(r"^(/)").unwrap()),
				(Kind::Operator, Name::Exp, Regex::new(r"^(\^)").unwrap()),





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
						//
						(Kind::Stop, Name::Comma)
						| (Kind::Stop, Name::Semicolon) => {
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
						//
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
						//
						(Kind::Paren, Name::ParenLF)
						| (Kind::Squaren, Name::SquarenLF)
						| (Kind::Bracket, Name::BracketLF) => {
							tokens.push(t);
							last_token_was_operator = true;
							last_token_was_comma = false;
							last_token_was_newline = false;
						}
						//
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
						//
						(Kind::Operator, _) => {
							if last_token_was_newline {
								tokens.pop();
							}
							tokens.push(t);
							last_token_was_operator = true;
							last_token_was_comma = false;
							last_token_was_newline = false;
						}
						//
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
	Ok(tokens)
}
