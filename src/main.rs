use std::collections::HashMap;
use serde_json::Value as Val;

mod inspect;
mod parse;
mod utils;

#[derive(Debug)]
enum Declaration {
	Util(Util),
	Object(Obj),
}

type Obj = HashMap<String, Declaration>;

#[derive(Debug)]
struct Info {
	levels_fields: inspect::Levels,
	default_locale: Locale,
}

#[derive(Debug)]
struct Ctx {
	rng: fake::rand::rngs::ThreadRng,
	current_path: Vec<String>,
	generated: Val,
}

fn main() -> anyhow::Result<()> {
	let path = "dev_schema.json";

	let file = std::fs::File::open(path)?;
	let reader = std::io::BufReader::new(file);
	let decl: parse::InputDeclaration = serde_json::from_reader(reader)?;

	let (levels_fields, decl) = inspect::inspect_input_declaration(decl)?;

	// let level = levels_fields.into_iter().find_map(|(k, v)| if k == "~" { Some(v) } else { None }).unwrap();
	// let graph = level.dag.graph();
	// let topo_order = daggy::petgraph::algo::toposort(graph, None).unwrap();
	// use daggy::petgraph::visit::IntoNodeReferences;
	// let node_map: std::collections::HashMap<_, _> = graph.node_references().collect();
	// for node_idx in topo_order {
	//   println!("Node: {}", node_map[&node_idx]);
	// }
	// let dot_output = daggy::petgraph::dot::Dot::with_config(graph, &[daggy::petgraph::dot::Config::EdgeNoLabel]);
	// let mut file = std::fs::File::create("graph.dot").unwrap();
	// use std::{io::Write};
	// file.write_all(dot_output.to_string().as_bytes()).unwrap();

	let default_locale = Locale::EN;
	let i = Info { levels_fields, default_locale };
	let mut ctx = Ctx {
		rng: fake::rand::rng(), current_path: vec!["~".to_string()], generated: Val::Null,
	};

	generate_declaration(&i, &mut ctx, &decl)?;
	let actual_data = ctx.generated;
	// let generated: Generated = generate_util(decl)?;
	// let actual_data: serde_json::Value = reduce_to_actual(generated)?;
	// maybe use this to simplify serialization
	// #[serde(serialize_with = "path")]

	let mut writer = std::io::BufWriter::new(std::io::stdout());
	serde_json::to_writer(&mut writer, &actual_data)?;

	Ok(())
}

fn generate_declaration(i: &Info, ctx: &mut Ctx, decl: &Declaration) -> anyhow::Result<()> {
	match decl {
		Declaration::Util(util) => generate_util(i, ctx, util)?,
		Declaration::Object(obj) => generate_object(i, ctx, obj)?,
	}

	Ok(())
}

fn generate_util(i: &Info, ctx: &mut Ctx, util: &Util) -> anyhow::Result<()> {
	unimplemented!()
}

fn generate_object(i: &Info, ctx: &mut Ctx, obj: &Obj) -> anyhow::Result<()> {
	let level_name = ctx.current_path.join(".");
	let ordered_fields = i.levels_fields.get(&level_name)
		.ok_or_else(|| anyhow::anyhow!("couldn't find fields for {level_name}"))?;

	let mut generated = HashMap::new();
	for field_name in ordered_fields {
		let field_decl = obj.get(field_name)
			.ok_or_else(|| anyhow::anyhow!("couldn't find declaration for {field_name}"))?;

		// TODO hmmm
		// I think we need to both return what we just generated *and* stick it in the generated context
		let v = generate_declaration(i, ctx, field_decl);

		generated.insert(field_name, v);
	}

	*ctx.generated.pointer_mut(&level_name)
		.ok_or_else(|| anyhow::anyhow!("couldn't find declaration for {level_name}"))? = generated.into();
	Ok(())
}

// fn generate_root_declaration(
// 	rng: &mut ThreadRng,
// 	decl: InputDeclaration,
// 	default_locale: Locale,
// ) -> anyhow::Result<serde_json::Value> {
// 	// use fake::{Fake, /*Faker*/};
// 	match decl {
// 		InputDeclaration::ShorthandUtil(parse::ShorthandUtil(util)) => generate_util(rng, util, default_locale),
// 		InputDeclaration::LonghandUtil(util) => generate_util(rng, util, default_locale),
// 		InputDeclaration::Object(obj) => ,
// 	}


// 	Ok(val)
// }


// macro_rules! gen_locale {
// 	($faker_path:path, $locale_var:ident, $rng:ident) => {
// 		match $locale_var {
// 			Locale::AR_SA => $faker_path(fake::locales::AR_SA).fake_with_rng($rng),
// 			Locale::DE_DE => $faker_path(fake::locales::DE_DE).fake_with_rng($rng),
// 			Locale::EN => $faker_path(fake::locales::EN).fake_with_rng($rng),
// 			Locale::FR_FR => $faker_path(fake::locales::FR_FR).fake_with_rng($rng),
// 			Locale::IT_IT => $faker_path(fake::locales::IT_IT).fake_with_rng($rng),
// 			Locale::JA_JP => $faker_path(fake::locales::JA_JP).fake_with_rng($rng),
// 			Locale::PT_BR => $faker_path(fake::locales::PT_BR).fake_with_rng($rng),
// 			Locale::PT_PT => $faker_path(fake::locales::PT_PT).fake_with_rng($rng),
// 			Locale::ZH_CN => $faker_path(fake::locales::ZH_CN).fake_with_rng($rng),
// 			Locale::ZH_TW => $faker_path(fake::locales::ZH_TW).fake_with_rng($rng),
// 		}
// 	};
// }

type Pattern = String;

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "#util")]
pub enum Util {
	Ref(Pattern),
	FirstName,
	LastName,
}

// fn generate_util(
// 	rng: &mut ThreadRng,
// 	util: Util,
// 	default_locale: Locale,
// ) -> anyhow::Result<serde_json::Value> {
// 	use fake::{Fake, /*Faker*/};
// 	use Util::*;
// 	let val = match util {
// 		FirstName => Val::String(gen_locale!(fake::faker::name::raw::FirstName, default_locale, rng)),
// 		LastName => Val::String(gen_locale!(fake::faker::name::raw::LastName, default_locale, rng)),
// 	};

// 	Ok(val)
// }

#[derive(Debug, serde::Deserialize)]
#[allow(non_camel_case_types)]
pub enum Locale {
	AR_SA, DE_DE, EN, FR_FR, IT_IT, JA_JP, PT_BR, PT_PT, ZH_CN, ZH_TW,
}



// impl fake::Dummy<FakePrimitive> for Val {
//  fn dummy_with_rng<R: fake::Rng + ?Sized>(config: &FakePrimitive, rng: &mut R) -> Self {
//     match config {
//      FakePrimitive::FirstName => {
//        Val::String(fake::faker::name::en::FirstName().fake_with_rng(rng))
//      },
//      FakePrimitive::LastName => {
//        Val::String(fake::faker::name::en::LastName().fake_with_rng(rng))
//      }
//    }
//  }
// }

// impl fake::Dummy<FakeSpec> for Val {
//  fn dummy_with_rng<R: fake::Rng + ?Sized>(config: &FakeSpec, rng: &mut R) -> Self {
//    match config {
//      FakeSpec::Primitive(inner) => inner.fake_with_rng(rng),
//      FakeSpec::Maybe{ inner, some_weight } => {
//        // let is_some = rng.random_bool(*some_weight);

//        let random_roll: f32 = Faker.fake_with_rng(rng);
//        // println!("{random_roll}, {some_weight}");
//        if random_roll >= *some_weight { Val::Null }
//        else { inner.fake_with_rng(rng) }
//      },
//    }
//  }
// }
