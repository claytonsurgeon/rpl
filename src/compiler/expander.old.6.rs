// create Btree of ast nodes
// create Btree of typ nodes

/**
 * add unnmaed points to btree, use their index as their key
 * for example: 6 -> "6"
 *
 * btree nodes have indexes, use that instead of a vector of trees
 * this will be easier to work with later when recompiling a graph and
 * trying to merge changes
 */
// #![allow(dead_code)]
// #![allow(unused_variables)]
use super::parser::{Ast, EID};
use super::tokenizer::Name;
use std::collections::BTreeMap;

pub type Maps = Vec<BTreeMap<String, Ast>>;
pub type Typs = Vec<BTreeMap<String, Typ>>;

pub enum Typ {}

#[derive(Debug, Clone)]
pub struct Parse {
	pub envs: Maps,
}

pub fn expander(ast: &Ast) -> Result<(Ast, Maps), String> {
	let mut parse = Parse {
		envs: vec![BTreeMap::new()], // 0th un-used, parent_eid 0 used to mean "no-parent"
	};

	let a = parse.map(ast, 0)?;
	dbg!(&parse.envs);
	Ok((a, parse.envs))
}

fn get_eid(ast: &Ast) -> Result<EID, String> {
	match ast {
		Ast::Graph(eid, _) => Ok(*eid),
		_ => Err(format!("{:?} is not a Graph, thus no eid", &ast)),
	}
}

fn get_label(ast: &Ast) -> Result<String, String> {
	match ast {
		Ast::Ref(_, label) => Ok(label.clone()),
		_ => Err(format!("{:?} is not a ref, thus no label", &ast)),
	}
}

fn key_match(point_s: &Ast, point_m: &Ast) -> bool {
	match point_s {
		Ast::Key(label_s, _) => {
			match point_m {
				Ast::Key(label_m, _) => {
					// note, rust auto-derefs references during comparisons
					label_s == label_m
				}
				_ => false,
			}
		}
		_ => false,
	}
}

fn has_key(points: &Vec<Ast>, point_s: &Ast) -> bool {
	for point_m in points {
		if key_match(point_s, point_m) {
			return true;
		}
	}
	false
}

fn get_key(points: &Vec<Ast>, point_s: &Ast) -> Option<Ast> {
	for point_m in points {
		if key_match(point_s, point_m) {
			return Some(point_m.clone());
		}
	}
	None
}

impl Parse {
	fn map(&mut self, ast: &Ast, parent_eid: EID) -> Result<Ast, String> {
		match ast {
			Ast::Graph(_, points) => {
				let index = self.envs.len();
				let mut env = BTreeMap::new();
				env.insert("<parent>".to_string(), Ast::Parent(parent_eid));

				self.envs.push(env);

				let mut expanded_points = Vec::new();
				for point in points {
					expanded_points.push(self.map(point, index)?);
				}

				Ok(Ast::Graph(index, expanded_points))
			}

			Ast::Key(label, point) => {
				let point = self.map(point, parent_eid)?;
				self.envs[parent_eid].insert(label.clone(), point.clone());
				// .insert(label.clone(), (parent_eid, point.clone()));
				Ok(Ast::Key(label.clone(), Box::new(point)))
			}

			Ast::Apply(source, modifier) => {
				let index = self.envs.len();
				let mut env = BTreeMap::new();
				env.insert("<parent>".to_string(), Ast::Parent(parent_eid));
				self.envs.push(env);
				// removed the concept of ordinal args, because I could not create an intuative set of replacement rules, especially with intermediary points
				// removed the concept of the intermediary points, because it also creates confusion

				// ref -> expander::ref
				let source = self.map(source, parent_eid)?;
				// ref -> ast
				let source = self.deref(&source)?;

				let s_points = match &source {
					Ast::Graph(_eid, points) => (*points).clone(),
					_ => Vec::new(),
				};

				let m_points = match (**modifier).clone() {
					Ast::Graph(_, points) => points,
					_ => Vec::new(),
				};

				// throw error for modifier points that don't exist in source
				for m_point in &m_points {
					if !has_key(&s_points, &m_point) {
						return Err(format!(
							"{:?} does not exist in {:?}",
							&m_point, &s_points
						));
					}
				}

				let mut points = Vec::new();
				for s_point in s_points {
					points.push(match get_key(&m_points, &s_point) {
						Some(point) => point,
						None => s_point.clone(),
					});
				}

				let mut expanded_points = Vec::new();
				for point in points {
					expanded_points.push(self.map(&point, index)?);
				}

				Ok(Ast::Graph(index, expanded_points))
			}

			Ast::Ref(_, label) => {
				// dbg!(self.lookup(label, parent));
				dbg!(self.deref(&Ast::Ref(parent_eid, label.clone()))?);
				Ok(Ast::Ref(parent_eid, label.clone()))
			}
			Ast::Op2(name, left, right) => match name {
				Name::Select => {
					let left = self.map(left, parent_eid)?;
					let left_ns = self.deref(&left)?;
					let eid = get_eid(&left_ns)?;

					// shouldn't need to map the right operand, it should
					// always be a simple ref, just get the label and go
					let right = self.map(right, eid)?;
					let label = get_label(&right)?;
					Ok(Ast::Ref(eid, label))
				}
				_ => Ok(Ast::Nothing),
			},
			Ast::Op1(name, operand) => Ok(Ast::Nothing),
			Ast::Op0(name) => Ok(Ast::Nothing),
			Ast::Space(sizes) => Ok(Ast::Nothing),

			Ast::Decimal(i) => Ok(Ast::Decimal(i.clone())),
			Ast::Integer(i) => Ok(Ast::Integer(i.clone())),
			Ast::String(i) => Ok(Ast::String(i.clone())),
			Ast::Clock(t, i) => Ok(Ast::Clock(t.clone(), i.clone())),
			Ast::Size(i) => Ok(Ast::Size(i.clone())),
			Ast::Nothing => Ok(Ast::Nothing),
			x => return Err(format!("No map match defined for {:?}", x)),
		}
	}
	// fn deref(&self, ref_: &Ast) -> Result<&Item, String> {
	fn deref(&self, ref_: &Ast) -> Result<&Ast, String> {
		match (*ref_).clone() {
			Ast::Ref(eid, label) => self.lookup(&label, eid),
			ast => Err(format!("{:?} is not a Reference", ast)),
		}
	}

	// fn lookup(&self, label: &String, eid: EID) -> Result<&Item, String> {
	fn lookup(&self, label: &String, eid: EID) -> Result<&Ast, String> {
		if eid == 0 {
			return Err(format!("Point {} is undefined", label));
		}
		match self.envs[eid].get(label) {
			Some(result) => Ok(result),
			None => self.lookup(
				label,
				// self.envs[eid].get(&"<parent>".to_string()).unwrap().0,
				get_parent_eid(
					self.envs[eid].get(&"<parent>".to_string()).unwrap(),
				),
			),
		}
	}
}

fn get_parent_eid(ast: &Ast) -> EID {
	match ast {
		Ast::Parent(eid) => *eid,
		_ => panic!("get_parent_eid used on non-parent node"),
	}
}
