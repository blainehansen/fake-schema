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

// use fake::faker::name::raw::Name;
use fake::{Fake, Faker};
// use fake::rand::SeedableRng;

fn generate_table(
	rng: &mut fake::rand::rngs::ThreadRng,
	row_count_range: Range<usize>,
	value_definition: HashMap<String, Spec>,
) -> anyhow::Result<Vec<serde_json::Map<String, Val>>> {
	let actual_count: usize = row_count_range.fake_with_rng(rng);

	(0..actual_count).map(|_| generate_row(rng, &value_definition)).collect()
}

fn generate_row(
	rng: &mut fake::rand::rngs::ThreadRng,
	value_definition: &HashMap<String, Spec>,
) -> anyhow::Result<serde_json::Map<String, Val>> {
	let mut row = serde_json::Map::new();
	let mut value_definitions: Vec<_> = value_definition.iter().collect();
	value_definitions.sort_by_key(|(_, v)| if let Spec::Fake(_) = v { false } else { true });

	for (key, value) in value_definitions {
		let generated_value: Val = match value {
			Spec::Fake(f) => f.fake_with_rng(rng),
			Spec::StrJoin { join_string, references } => {
				let formatted_string = references.into_iter().filter_map(|r| {
					match row.get(r).unwrap() {
						Val::String(s) => Some(s.clone()),
						Val::Null => None,
						Val::Bool(b) => Some(b.to_string()),
						Val::Number(n) => Some(n.to_string()),
						_ => None,
					}
				}).collect::<Vec<String>>().join(join_string);

				Val::String(formatted_string)
			},
		};

		row.insert(key.to_string(), generated_value);
	}

	Ok(row)
}


fn main() -> anyhow::Result<()> {
	let args = std::env::args().skip(1).collect::<Vec<String>>();
	let filename_arg = args.get(0).expect("must have a single command line arg");

	let file = std::fs::File::open(filename_arg)?;
	let input_schema: HashMap<String, Spec> = serde_json::from_reader(std::io::BufReader::new(file))?;

	let mut rng = fake::rand::rng();
	let generated_values = generate_table(&mut rng, 6..10, input_schema)?;
	for v in generated_values {
		let v = serde_json::to_string(&v)?;
		println!("{v}");
	}

	Ok(())


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
}




use nom::{
	IResult, Parser,
	number,
	branch::alt,
	bytes::{complete::tag, is_not, take_while1},
	combinator::{all_consuming, map, value},
	multi::separated_list1,
	sequence::{delimited, separated_pair},
};


#[derive(Debug, Deserialize)]
#[serde(try_from = "String")]
enum Spec {
	Fake(FakeSpec),
	StrJoin{ join_string: String, references: Vec<String> },
}

impl TryFrom<String> for Spec {
	type Error = anyhow::Error;

	fn try_from(input: String) -> Result<Self, Self::Error> {
		all_consuming(spec).parse_complete(input.as_str())
			.map(|(_, spec)| spec)
			.map_err(|e| anyhow::anyhow!("{e}"))
	}
}

fn spec(input: &str) -> IResult<&str, Spec> {
	alt((
		map(fake_spec, Spec::Fake),
		str_join,
	)).parse(input)
}
fn str_join(input: &str) -> IResult<&str, Spec> {
	let (input, (join_string, strs)) = delimited(
		tag("StrJoin("),
		separated_pair(delimited(nom::character::char('\''), is_not("'"), nom::character::char('\'')), tag(", "), separated_list1(tag(", "), ident)),
		tag(")"),
	).parse(input)?;

	Ok((input, Spec::StrJoin {
		join_string: join_string.to_string(),
		references: strs.iter().map(|s| s.to_string()).collect(),
	}))
}

fn ident(input: &str) -> IResult<&str, &str> {
	take_while1(|i: char| i == '_' || i.is_alphanumeric()).parse(input)
}


#[derive(Debug, Clone)]
enum FakeSpec {
	Primitive(FakePrimitive),
	Maybe { inner: FakePrimitive, some_weight: f32 },
}
fn fake_spec(input: &str) -> IResult<&str, FakeSpec> {
	alt((
		map(fake_primitive, FakeSpec::Primitive),
		maybe,
	)).parse(input)
}
fn maybe(input: &str) -> IResult<&str, FakeSpec> {
	map(
		delimited(tag("Maybe("), separated_pair(fake_primitive, tag(", "), |i| number::float().parse(i)), tag(")")),
		|(inner, some_weight)| FakeSpec::Maybe { inner, some_weight },
	).parse(input)
}

#[derive(Debug, Clone)]
enum FakePrimitive {
	FirstName,
	LastName,
}
fn fake_primitive(input: &str) -> IResult<&str, FakePrimitive> {
	alt((
		value(FakePrimitive::FirstName, tag("FirstName")),
		value(FakePrimitive::LastName, tag("LastName")),
	)).parse(input)
}


// fn yo<I, O, E: nom::error::ParseError<I>, F, G>(parser: F, f: G) -> impl Parser<I, Output = O, Error = E>
// where
//   F: Parser<I, Error = E>,
//   G: FnMut(<F as Parser<I>>::Output) -> O,
// {
//   parser.map(f)
// }



impl fake::Dummy<FakePrimitive> for Val {
	fn dummy_with_rng<R: fake::Rng + ?Sized>(config: &FakePrimitive, rng: &mut R) -> Self {
		 match config {
			FakePrimitive::FirstName => {
				Val::String(fake::faker::name::en::FirstName().fake_with_rng(rng))
			},
			FakePrimitive::LastName => {
				Val::String(fake::faker::name::en::LastName().fake_with_rng(rng))
			}
		}
	}
}

impl fake::Dummy<FakeSpec> for Val {
	fn dummy_with_rng<R: fake::Rng + ?Sized>(config: &FakeSpec, rng: &mut R) -> Self {
		match config {
			FakeSpec::Primitive(inner) => inner.fake_with_rng(rng),
			FakeSpec::Maybe{ inner, some_weight } => {
				// let is_some = rng.random_bool(some_weight);

				let random_roll: f32 = Faker.fake_with_rng(rng);
				// println!("{random_roll}, {some_weight}");
				if random_roll >= *some_weight { Val::Null }
				else { inner.fake_with_rng(rng) }
			},
		}
	}
}
