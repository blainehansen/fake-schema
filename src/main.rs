// #[derive(Debug)]
// enum Node {
// 	Field(FieldNode),
// 	Data(DataNode),
// }

// #[derive(Debug)]
// struct FieldNode {
// 	decl: Decl,
// 	field_dag: daggy::Dag<String, ()>,
// 	children: HashMap<String, Node>,
// }

// #[derive(Debug)]
// struct DataNode {
// 	decl: Decl,
// 	data: serde_json::Value,
// }
// struct Ctx {
// 	rng: fake::rand::rngs::ThreadRng,
// 	current_path: Vec<String>,
// 	root_node: FieldNode,
// }

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum Decl {
	// ShortUtil(ShortUtil),
	Util(Util),
	Object(HashMap<String, Decl>),
}


fn generate_root(
	// rng: fake::rand::rngs::ThreadRng,
	decl: HashMap<String, Decl>,
) -> LevelDag {
	// first we have to inspect root, to construct the dag of the fields at this level
	// then we can begin generating each field in order

	let mut dags = HashMap::from([
		("~".to_string(), LevelDag { node_indices: Default::default(), dag: daggy::Dag::new() }),
	]);

	let mut current_path = vec!["~".to_string()];

	inspect_object(&mut dags, &mut current_path, &decl);

	dags.into_iter().find_map(|(k, v)| if k == "~" { Some(v) } else { None }).unwrap()

	// let root_node = FieldNode { decl: Decl::Object(decl.clone()), field_dag: daggy::Dag::new(), children: HashMap::new() };

	// // iterate over the fields of decl (somehow demand it is of object shape for now)
	// // first inspect the fields to construct the field_dag
	// let mut ctx = Ctx { rng, current_path: vec![], root_node };

	// for (field_name, field_decl) in decl.iter() {
	// 	ctx.current_path.push(field_name.to_string());
	// 	ctx.current_path.pop();

	// 	let child_node = match field_decl {
	// 		Decl::ShortUtil(ShortUtil(util)) => generate_util(&mut ctx, util),
	// 		Decl::Util(util) => generate_util(&mut ctx, util),
	// 		Decl::Object(obj_decl) => Node::Field(generate_object(&mut ctx, obj_decl)),
	// 	};

	// 	ctx.root_node.children.insert(field_name.to_string(), child_node);
	// }

	// ctx.root_node
}

#[derive(Debug, Default)]
struct LevelDag {
	node_indices: HashMap<String, daggy::NodeIndex>,
	dag: daggy::Dag<String, Emp>,
}



type Dags = HashMap<String, LevelDag>;

fn inspect_object(
	dags: &mut Dags,
	current_path: &mut Vec<String>,
	decl: &HashMap<String, Decl>,
) -> () {
	for (field_name, field_decl) in decl {
		current_path.push(field_name.to_string());

		match field_decl {
			// Decl::ShortUtil(ShortUtil(util)) => inspect_util(dags, current_path, util),
			Decl::Util(util) => inspect_util(dags, current_path, util),
			Decl::Object(obj_decl) => inspect_object(dags, current_path, obj_decl),
		}

		current_path.pop();
	}
}

fn inspect_util(
	dags: &mut Dags,
	current_path: &mut Vec<String>,
	util: &Util,
) -> () {
	match util {
		Util::Ref(pat) => {
			let (common_prefix, depending_name, depended_name) = find_common_prefix(&current_path, pat);
			let node_level = dags.entry(common_prefix).or_default();
			let depending = node_level.node_indices.entry(depending_name.clone()).or_insert_with(|| node_level.dag.add_node(depending_name)).clone();
			let depended = node_level.node_indices.entry(depended_name.clone()).or_insert_with(|| node_level.dag.add_node(depended_name)).clone();
			if node_level.dag.find_edge(depended, depending).is_none() {
				// TODO basically all unwraps should return error
				node_level.dag.add_edge(depended, depending, Emp).unwrap();
			}
		},
		Util::FirstName => {},
	}
}

fn find_common_prefix(current_path: &Vec<String>, pat: &Pat) -> (String, String, String) {
	// TODO assumption that pat is never fully qualified
	// if !pat.starts_with("~") {
	// 	pat.exten
	// }
	let split_pat: Vec<_> = ["~"].into_iter().chain(pat.split(".")).collect();

	let mut common_prefix = vec![];
	for (current_segment, pat_segment) in zip_longest(current_path, split_pat) {
		// println!("{current_segment:?}, {pat_segment:?}");
		match (current_segment, pat_segment) {
			(Some(current_segment), Some(pat_segment)) => {
				if *current_segment == pat_segment {
					common_prefix.push(pat_segment);
				} else {
					// here this means they aren't equal, which means we're done
					// the point we're currently at *needs* the segment referred to in the pattern
					return (common_prefix.join("."), current_segment.to_string(), pat_segment.to_string())
				}
			},
			// we have a pattern that's longer than our current point
			// ~.country.locale => ~.country
			// this doesn't make sense, this is pointing merely back up the tree, but without diverting at all, so this would expand infinitely

			// we have a pattern that's shorter than our current point
			// ~.country => ~.country.locale
			// this doens't make sense, because it's pointing downward at itself, trying to say the full object should be one of it's own fields
			_ => {
				panic!("this doesn't make sense? {pat:?} => {current_path:?}");
			},
		}
	}

	// if we get to this point that means they're exactly equal, which means the pat is empty? something's wrong, panic
	// TODO this should be an error in the future
	panic!("pointing to itself? {pat:?} => {current_path:?}");
}

// fn generate_util(ctx: &mut Ctx, util: &Util) -> Node {
// 	unimplemented!()
// }

// fn generate_object(ctx: &mut Ctx, decl: &HashMap<String, Decl>) -> FieldNode {
// 	unimplemented!()
// }

// fn generate_list(
// 	rng: &mut fake::rand::rngs::ThreadRng,
// ) -> RetType {
// 	unimplemented!()
// }


// use std::{io::Write};

// use daggy::petgraph::{self, visit::IntoNodeReferences, dot::{Dot, Config}};

// fn main() {
// 	let mut dag = daggy::Dag::<String, &'static str>::new();

// 	let a = dag.add_node("a".to_string());
// 	let (_, b) = dag.add_parent(a, "", "b".to_string());
// 	let (_, c) = dag.add_parent(a, "", "c".to_string());
// 	let (_, d) = dag.add_parent(c, "", "d".to_string());

// 	let graph = dag.graph();
// 	let dot_output = Dot::with_config(graph, &[Config::EdgeNoLabel]);
// 	let mut file = std::fs::File::create("graph.dot").unwrap();
// 	file.write_all(dot_output.to_string().as_bytes()).unwrap();

// 	let topo_order = petgraph::algo::toposort(graph, None).unwrap();
// 	let node_map: std::collections::HashMap<_, _> = graph.node_references().collect();

// 	for node_idx in topo_order {
// 		println!("Node: {}", node_map[&node_idx]);
// 	}
// }


use std::{collections::HashMap, ops::Range};
use serde::Deserialize;
use serde_json::{Value as Val};

fn main() -> anyhow::Result<()> {
	let level = generate_root(HashMap::from([
		("a".to_string(), Decl::Util(Util::FirstName)),
		("b".to_string(), Decl::Util(Util::Ref("a".to_string()))),
		("c".to_string(), Decl::Util(Util::Ref("a".to_string()))),
		("d".to_string(), Decl::Util(Util::Ref("c".to_string()))),
	]));

	let graph = level.dag.graph();
	let topo_order = daggy::petgraph::algo::toposort(graph, None).unwrap();
	use daggy::petgraph::visit::IntoNodeReferences;
	let node_map: std::collections::HashMap<_, _> = graph.node_references().collect();
	for node_idx in topo_order {
		println!("Node: {}", node_map[&node_idx]);
	}

	let dot_output = daggy::petgraph::dot::Dot::with_config(graph, &[daggy::petgraph::dot::Config::EdgeNoLabel]);
	let mut file = std::fs::File::create("graph.dot").unwrap();
	use std::{io::Write};
	file.write_all(dot_output.to_string().as_bytes()).unwrap();

	// let args = std::env::args().skip(1).collect::<Vec<String>>();
	// let filename_arg = args.get(0).expect("must have a single command line arg");

	// let file = std::fs::File::open(filename_arg)?;
	// let input_schema: HashMap<String, Spec> = serde_json::from_reader(std::io::BufReader::new(file))?;

	// let mut rng = fake::rand::rng();
	// let generated_values = generate_table(&mut rng, 6..10, input_schema)?;
	// for v in generated_values {
	// 	let v = serde_json::to_string(&v)?;
	// 	println!("{v}");
	// }



	// // TODO at some point it will be necessary to preprocess and analyze the map to find Ref dependencies between them

	// // at the end of this we want to have created our own map of tables to a Vec
	// let mut generated_tables = std::collections::HashMap::new();

	// for (table_name, table_definition) in input_schema.iter() {
	// 	println!("doing table {table_name}");

	// 	// let table_definition = match table_value {
	// 	// 	Val::Object(table_definition) => table_definition,
	// 	// 	_ => panic!("json must be a map of table definitions"),
	// 	// };

	// 	// let input_fake = all_consuming(fake_primitive).parse_complete(arg).expect("wasn't able to parse input").1;
	// 	let value: String = Basic;

	// 	println!("{value}");
	// 	unimplemented!();
	// }

	Ok(())
}



// use fake::faker::name::raw::Name;
use fake::{Fake, Faker};
// use fake::rand::SeedableRng;

// fn generate_table(
// 	rng: &mut fake::rand::rngs::ThreadRng,
// 	row_count_range: Range<usize>,
// 	value_definition: HashMap<String, Spec>,
// ) -> anyhow::Result<Vec<serde_json::Map<String, Val>>> {
// 	let actual_count: usize = row_count_range.fake_with_rng(rng);

// 	(0..actual_count).map(|_| generate_row(rng, &value_definition)).collect()
// }

// fn generate_row(
// 	rng: &mut fake::rand::rngs::ThreadRng,
// 	value_definition: &HashMap<String, Spec>,
// ) -> anyhow::Result<serde_json::Map<String, Val>> {
// 	let mut row = serde_json::Map::new();
// 	let mut value_definitions: Vec<_> = value_definition.iter().collect();
// 	value_definitions.sort_by_key(|(_, v)| if let Spec::Fake(_) = v { false } else { true });

// 	for (key, value) in value_definitions {
// 		let generated_value: Val = match value {
// 			Spec::Fake(f) => f.fake_with_rng(rng),
// 			Spec::StrJoin { join_string, references } => {
// 				let formatted_string = references.into_iter().filter_map(|r| {
// 					match row.get(r).unwrap() {
// 						Val::String(s) => Some(s.clone()),
// 						Val::Null => None,
// 						Val::Bool(b) => Some(b.to_string()),
// 						Val::Number(n) => Some(n.to_string()),
// 						_ => None,
// 					}
// 				}).collect::<Vec<String>>().join(join_string);

// 				Val::String(formatted_string)
// 			},
// 		};

// 		row.insert(key.to_string(), generated_value);
// 	}

// 	Ok(row)
// }






type Pat = String;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "#util")]
enum Util {
	Ref(Pat),
	FirstName,
	// List(List),
	// ChooseOne(ChooseOne),
	// ChooseN(ChooseN),
}

// #[derive(Debug, Clone, Deserialize)]
// struct List {
// 	#[serde(rename = "#count")]
// 	count: usize,
// }



// use nom::{
// 	IResult, Parser,
// 	number,
// 	branch::alt,
// 	bytes::{complete::tag, is_not, take_while1},
// 	combinator::{all_consuming, map, value},
// 	multi::separated_list1,
// 	sequence::{delimited, separated_pair},
// };
// #[derive(Debug, Clone, Deserialize)]
// #[serde(try_from = "String")]
// struct ShortUtil(Util);

// impl TryFrom<String> for ShortUtil {
// 	type Error = anyhow::Error;

// 	fn try_from(input: String) -> Result<Self, Self::Error> {
// 		Ok(ShortUtil(todo!()))
// 	}
// }




// #[derive(Debug, Deserialize)]
// #[serde(try_from = "String")]
// enum Spec {
// 	Fake(FakeSpec),
// 	StrJoin{ join_string: String, references: Vec<String> },
// }

// impl TryFrom<String> for Spec {
// 	type Error = anyhow::Error;

// 	fn try_from(input: String) -> Result<Self, Self::Error> {
// 		all_consuming(spec).parse_complete(input.as_str())
// 			.map(|(_, spec)| spec)
// 			.map_err(|e| anyhow::anyhow!("{e}"))
// 	}
// }

// fn spec(input: &str) -> IResult<&str, Spec> {
// 	alt((
// 		map(fake_spec, Spec::Fake),
// 		str_join,
// 	)).parse(input)
// }
// fn str_join(input: &str) -> IResult<&str, Spec> {
// 	let (input, (join_string, strs)) = delimited(
// 		tag("StrJoin("),
// 		separated_pair(delimited(nom::character::char('\''), is_not("'"), nom::character::char('\'')), tag(", "), separated_list1(tag(", "), ident)),
// 		tag(")"),
// 	).parse(input)?;

// 	Ok((input, Spec::StrJoin {
// 		join_string: join_string.to_string(),
// 		references: strs.iter().map(|s| s.to_string()).collect(),
// 	}))
// }

// fn ident(input: &str) -> IResult<&str, &str> {
// 	take_while1(|i: char| i == '_' || i.is_alphanumeric()).parse(input)
// }


// #[derive(Debug, Clone)]
// enum FakeSpec {
// 	Primitive(FakePrimitive),
// 	Maybe { inner: FakePrimitive, some_weight: f32 },
// }
// fn fake_spec(input: &str) -> IResult<&str, FakeSpec> {
// 	alt((
// 		map(fake_primitive, FakeSpec::Primitive),
// 		maybe,
// 	)).parse(input)
// }
// fn maybe(input: &str) -> IResult<&str, FakeSpec> {
// 	map(
// 		delimited(tag("Maybe("), separated_pair(fake_primitive, tag(", "), |i| number::float().parse(i)), tag(")")),
// 		|(inner, some_weight)| FakeSpec::Maybe { inner, some_weight },
// 	).parse(input)
// }

// #[derive(Debug, Clone)]
// enum FakePrimitive {
// 	FirstName,
// 	LastName,
// }
// fn fake_primitive(input: &str) -> IResult<&str, FakePrimitive> {
// 	alt((
// 		value(FakePrimitive::FirstName, tag("FirstName")),
// 		value(FakePrimitive::LastName, tag("LastName")),
// 	)).parse(input)
// }


// // fn yo<I, O, E: nom::error::ParseError<I>, F, G>(parser: F, f: G) -> impl Parser<I, Output = O, Error = E>
// // where
// //   F: Parser<I, Error = E>,
// //   G: FnMut(<F as Parser<I>>::Output) -> O,
// // {
// //   parser.map(f)
// // }



// impl fake::Dummy<FakePrimitive> for Val {
// 	fn dummy_with_rng<R: fake::Rng + ?Sized>(config: &FakePrimitive, rng: &mut R) -> Self {
// 		 match config {
// 			FakePrimitive::FirstName => {
// 				Val::String(fake::faker::name::en::FirstName().fake_with_rng(rng))
// 			},
// 			FakePrimitive::LastName => {
// 				Val::String(fake::faker::name::en::LastName().fake_with_rng(rng))
// 			}
// 		}
// 	}
// }

// impl fake::Dummy<FakeSpec> for Val {
// 	fn dummy_with_rng<R: fake::Rng + ?Sized>(config: &FakeSpec, rng: &mut R) -> Self {
// 		match config {
// 			FakeSpec::Primitive(inner) => inner.fake_with_rng(rng),
// 			FakeSpec::Maybe{ inner, some_weight } => {
// 				// let is_some = rng.random_bool(*some_weight);

// 				let random_roll: f32 = Faker.fake_with_rng(rng);
// 				// println!("{random_roll}, {some_weight}");
// 				if random_roll >= *some_weight { Val::Null }
// 				else { inner.fake_with_rng(rng) }
// 			},
// 		}
// 	}
// }



fn zip_longest<I, J>(left: I, right: J) -> impl Iterator<Item = (Option<I::Item>, Option<J::Item>)>
where
	I: IntoIterator,
	J: IntoIterator,
{
	let mut left = left.into_iter();
	let mut right = right.into_iter();
	std::iter::from_fn(move || match (left.next(), right.next()) {
		(None, None) => None,
		(x, y) => Some((x, y)),
	})
}

#[derive(Debug)]
struct Emp;

// manually for the type.
impl std::fmt::Display for Emp {
	// This trait requires `fmt` with this exact signature.
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		// Write strictly the first element into the supplied output
		// stream: `f`. Returns `fmt::Result` which indicates whether the
		// operation succeeded or failed. Note that `write!` uses syntax which
		// is very similar to `println!`.
		write!(f, "")
	}
}
