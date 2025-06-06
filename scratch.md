```rust
// #[derive(Debug)]
// enum Node {
//  Field(FieldNode),
//  Data(DataNode),
// }

// #[derive(Debug)]
// struct FieldNode {
//  decl: Decl,
//  field_dag: daggy::Dag<String, ()>,
//  children: HashMap<String, Node>,
// }

// #[derive(Debug)]
// struct DataNode {
//  decl: Decl,
//  data: serde_json::Value,
// }
// struct Ctx {
//  rng: fake::rand::rngs::ThreadRng,
//  current_path: Vec<String>,
//  root_node: FieldNode,
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

  dbg!(dags).into_iter().find_map(|(k, v)| if k == "~" { Some(v) } else { None }).unwrap()

  // let root_node = FieldNode { decl: Decl::Object(decl.clone()), field_dag: daggy::Dag::new(), children: HashMap::new() };

  // // iterate over the fields of decl (somehow demand it is of object shape for now)
  // // first inspect the fields to construct the field_dag
  // let mut ctx = Ctx { rng, current_path: vec![], root_node };

  // for (field_name, field_decl) in decl.iter() {
  //  ctx.current_path.push(field_name.to_string());
  //  ctx.current_path.pop();

  //  let child_node = match field_decl {
  //    Decl::ShortUtil(ShortUtil(util)) => generate_util(&mut ctx, util),
  //    Decl::Util(util) => generate_util(&mut ctx, util),
  //    Decl::Object(obj_decl) => Node::Field(generate_object(&mut ctx, obj_decl)),
  //  };

  //  ctx.root_node.children.insert(field_name.to_string(), child_node);
  // }

  // ctx.root_node
}


// fn generate_util(ctx: &mut Ctx, util: &Util) -> Node {
//  unimplemented!()
// }

// fn generate_object(ctx: &mut Ctx, decl: &HashMap<String, Decl>) -> FieldNode {
//  unimplemented!()
// }

// fn generate_list(
//  rng: &mut fake::rand::rngs::ThreadRng,
// ) -> RetType {
//  unimplemented!()
// }


// use std::{io::Write};

// use daggy::petgraph::{self, visit::IntoNodeReferences, dot::{Dot, Config}};

// fn main() {
//  let mut dag = daggy::Dag::<String, &'static str>::new();

//  let a = dag.add_node("a".to_string());
//  let (_, b) = dag.add_parent(a, "", "b".to_string());
//  let (_, c) = dag.add_parent(a, "", "c".to_string());
//  let (_, d) = dag.add_parent(c, "", "d".to_string());

//  let graph = dag.graph();
//  let dot_output = Dot::with_config(graph, &[Config::EdgeNoLabel]);
//  let mut file = std::fs::File::create("graph.dot").unwrap();
//  file.write_all(dot_output.to_string().as_bytes()).unwrap();

//  let topo_order = petgraph::algo::toposort(graph, None).unwrap();
//  let node_map: std::collections::HashMap<_, _> = graph.node_references().collect();

//  for node_idx in topo_order {
//    println!("Node: {}", node_map[&node_idx]);
//  }
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

    ("o".to_string(), Decl::Object(HashMap::from([
      ("1".to_string(), Decl::Util(Util::FirstName)),
      ("2".to_string(), Decl::Util(Util::Ref("1".to_string()))),
      ("3".to_string(), Decl::Util(Util::Ref("~.b".to_string()))),
      ("4".to_string(), Decl::Util(Util::Ref("~.c".to_string()))),
    ])))
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
  //  let v = serde_json::to_string(&v)?;
  //  println!("{v}");
  // }



  // // TODO at some point it will be necessary to preprocess and analyze the map to find Ref dependencies between them

  // // at the end of this we want to have created our own map of tables to a Vec
  // let mut generated_tables = std::collections::HashMap::new();

  // for (table_name, table_definition) in input_schema.iter() {
  //  println!("doing table {table_name}");

  //  // let table_definition = match table_value {
  //  //  Val::Object(table_definition) => table_definition,
  //  //  _ => panic!("json must be a map of table definitions"),
  //  // };

  //  // let input_fake = all_consuming(fake_primitive).parse_complete(arg).expect("wasn't able to parse input").1;
  //  let value: String = Basic;

  //  println!("{value}");
  //  unimplemented!();
  // }

  Ok(())
}



// use fake::faker::name::raw::Name;
use fake::{Fake, Faker};
// use fake::rand::SeedableRng;

// fn generate_table(
//  rng: &mut fake::rand::rngs::ThreadRng,
//  row_count_range: Range<usize>,
//  value_definition: HashMap<String, Spec>,
// ) -> anyhow::Result<Vec<serde_json::Map<String, Val>>> {
//  let actual_count: usize = row_count_range.fake_with_rng(rng);

//  (0..actual_count).map(|_| generate_row(rng, &value_definition)).collect()
// }

// fn generate_row(
//  rng: &mut fake::rand::rngs::ThreadRng,
//  value_definition: &HashMap<String, Spec>,
// ) -> anyhow::Result<serde_json::Map<String, Val>> {
//  let mut row = serde_json::Map::new();
//  let mut value_definitions: Vec<_> = value_definition.iter().collect();
//  value_definitions.sort_by_key(|(_, v)| if let Spec::Fake(_) = v { false } else { true });

//  for (key, value) in value_definitions {
//    let generated_value: Val = match value {
//      Spec::Fake(f) => f.fake_with_rng(rng),
//      Spec::StrJoin { join_string, references } => {
//        let formatted_string = references.into_iter().filter_map(|r| {
//          match row.get(r).unwrap() {
//            Val::String(s) => Some(s.clone()),
//            Val::Null => None,
//            Val::Bool(b) => Some(b.to_string()),
//            Val::Number(n) => Some(n.to_string()),
//            _ => None,
//          }
//        }).collect::<Vec<String>>().join(join_string);

//        Val::String(formatted_string)
//      },
//    };

//    row.insert(key.to_string(), generated_value);
//  }

//  Ok(row)
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
//  #[serde(rename = "#count")]
//  count: usize,
// }
```
