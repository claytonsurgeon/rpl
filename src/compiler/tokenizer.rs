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
	Size,
	//
	String,
	Number,
	Clock,
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
	// Label,  // 		:			symbol				label expression
	Or,  // 			|			bitwise OR			bitwise OR
	And, // 			&			bitwise AND			bitwise AND
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
	Las,
	Key,
	Sym,
	Ref,
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
	//
	Second(Metric),
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Metric {
	Tera,
	Giga,
	Mega,
	Kilo,
	Hecto,
	Deca,
	Base,
	Deci,
	Centi,
	Milli,
	Micro,
	Nano,
	Pico,
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


				// reserved sizes
				(Kind::Size, Name::I8, Regex::new(r"^i8\b").unwrap()),
				(Kind::Size, Name::I16, Regex::new(r"^i16\b").unwrap()),
				(Kind::Size, Name::I32, Regex::new(r"^i32\b").unwrap()),
				(Kind::Size, Name::I64, Regex::new(r"^i64\b").unwrap()),
				(Kind::Size, Name::I128, Regex::new(r"^i128\b").unwrap()),

				(Kind::Size, Name::U8, Regex::new(r"^u8\b").unwrap()),
				(Kind::Size, Name::U16, Regex::new(r"^u16\b").unwrap()),
				(Kind::Size, Name::U32, Regex::new(r"^u32\b").unwrap()),
				(Kind::Size, Name::U64, Regex::new(r"^u64\b").unwrap()),
				(Kind::Size, Name::U128, Regex::new(r"^u128\b").unwrap()),

				(Kind::Size, Name::C8, Regex::new(r"^c8\b").unwrap()),
				(Kind::Size, Name::C16, Regex::new(r"^c16\b").unwrap()),
				(Kind::Size, Name::C32, Regex::new(r"^c32\b").unwrap()),

				(Kind::Size, Name::F32, Regex::new(r"^f32\b").unwrap()),
				(Kind::Size, Name::F64, Regex::new(r"^f64\b").unwrap()),

				// Reserved Words
				(Kind::Word, Name::Reserved, Regex::new(r"^if\b").unwrap()),
				(Kind::Word, Name::Reserved, Regex::new(r"^else\b").unwrap()),




				(Kind::Operator, Name::Pattern, Regex::new(r"^::").unwrap()),
				// // Symbol
				// (Kind::Word, Name::Sym, Regex::new(r"^:[A-Za-z'_][A-Za-z0-9'_]*").unwrap()),
				// (Kind::Word, Name::Sym, Regex::new(r"^:[$#|&=`><*/^+-]+").unwrap()),
				// Label
				(Kind::Word, Name::Key, Regex::new(r"^[A-Za-z'_][A-Za-z0-9'_]*:").unwrap()),
				(Kind::Word, Name::Key, Regex::new(r"^[$#|&=`><*/^+-]+:").unwrap()),
				// Label as such
				(Kind::Word, Name::Las, Regex::new(r"^[A-Za-z'_][A-Za-z0-9'_]*;").unwrap()),
				(Kind::Word, Name::Las, Regex::new(r"^[$#|&=`><*/^+-]+;").unwrap()),
				//
				// Words
				(Kind::Word, Name::Ref, Regex::new(r"^[A-Za-z'_][A-Za-z0-9'_]*").unwrap()),


				(Kind::Clock, Name::Second(Metric::Tera),  Regex::new(r"^[0-9]+Ts").unwrap()),
				(Kind::Clock, Name::Second(Metric::Giga),  Regex::new(r"^[0-9]+Gs").unwrap()),
				(Kind::Clock, Name::Second(Metric::Mega),  Regex::new(r"^[0-9]+Ms").unwrap()),
				(Kind::Clock, Name::Second(Metric::Kilo),  Regex::new(r"^[0-9]+Ks").unwrap()),
				(Kind::Clock, Name::Second(Metric::Hecto), Regex::new(r"^[0-9]+Hs").unwrap()),
				(Kind::Clock, Name::Second(Metric::Deca),  Regex::new(r"^[0-9]+Ds").unwrap()),

				(Kind::Clock, Name::Second(Metric::Base),  Regex::new(r"^[0-9]+s").unwrap()),

				(Kind::Clock, Name::Second(Metric::Deci),  Regex::new(r"^[0-9]+ds").unwrap()),
				(Kind::Clock, Name::Second(Metric::Centi), Regex::new(r"^[0-9]+cs").unwrap()),
				(Kind::Clock, Name::Second(Metric::Milli), Regex::new(r"^[0-9]+ms").unwrap()),
				(Kind::Clock, Name::Second(Metric::Micro), Regex::new(r"^[0-9]+[uμ]s").unwrap()),
				(Kind::Clock, Name::Second(Metric::Nano),  Regex::new(r"^[0-9]+ns").unwrap()),
				(Kind::Clock, Name::Second(Metric::Pico),  Regex::new(r"^[0-9]+ps").unwrap()),
				// Numbers
				(Kind::Number, Name::Decimal, Regex::new(r"^[0-9]+\.[0-9]*").unwrap()),
				(Kind::Number, Name::Decimal, Regex::new(r"^[0-9]*\.[0-9]+").unwrap()),
				(Kind::Number, Name::Integer, Regex::new(r"^[0-9']+").unwrap()),
				(Kind::Number, Name::Boolean, Regex::new(r"^(false|true)\b").unwrap()),

				// Operators
				(Kind::Operator, Name::Pattern, Regex::new(r"^~").unwrap()),
				//
				(Kind::Operator, Name::Arrow, Regex::new(r"^(->|→)").unwrap()),
				(Kind::Operator, Name::Bleed, Regex::new(r"^\.\.").unwrap()),
				(Kind::Operator, Name::Select, Regex::new(r"^[.]").unwrap()),

				(Kind::Operator, Name::Signal, Regex::new(r"^\?").unwrap()),
				// (Kind::Operator, Name::Label, Regex::new(r"^:").unwrap()),
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
