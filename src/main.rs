use std::{collections::HashMap};
use serde_json::Value as Val;
use jsonptr::PointerBuf;

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
	current_path: PointerBuf,
	generated: Val,
}

fn main() -> anyhow::Result<()> {
	let path = "dev_schema.json";

	let file = std::fs::File::open(path)?;
	let reader = std::io::BufReader::new(file);
	let decl: parse::InputDeclaration = serde_json::from_reader(reader)?;

	let (levels_fields, decl) = inspect::inspect_input_declaration(decl)?;

	// let level = levels_fields.into_iter().find_map(|(k, v)| if k == "/" { Some(v) } else { None }).unwrap();
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
		rng: fake::rand::rng(), current_path: PointerBuf::root(), generated: Val::Null,
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

#[derive(Debug, Clone)]
pub enum Util {
	Ref{ pattern: jsonptr::PointerBuf, relative: bool },
	FirstName,
	LastName,
}

macro_rules! gen_locale {
	($faker_path:path, $info_ident:ident, $ctx_ident:ident) => {
		match $info_ident.default_locale {
			Locale::AR_SA => $faker_path(fake::locales::AR_SA).fake_with_rng(&mut $ctx_ident.rng),
			Locale::DE_DE => $faker_path(fake::locales::DE_DE).fake_with_rng(&mut $ctx_ident.rng),
			Locale::EN => $faker_path(fake::locales::EN).fake_with_rng(&mut $ctx_ident.rng),
			Locale::FR_FR => $faker_path(fake::locales::FR_FR).fake_with_rng(&mut $ctx_ident.rng),
			Locale::IT_IT => $faker_path(fake::locales::IT_IT).fake_with_rng(&mut $ctx_ident.rng),
			Locale::JA_JP => $faker_path(fake::locales::JA_JP).fake_with_rng(&mut $ctx_ident.rng),
			Locale::PT_BR => $faker_path(fake::locales::PT_BR).fake_with_rng(&mut $ctx_ident.rng),
			Locale::PT_PT => $faker_path(fake::locales::PT_PT).fake_with_rng(&mut $ctx_ident.rng),
			Locale::ZH_CN => $faker_path(fake::locales::ZH_CN).fake_with_rng(&mut $ctx_ident.rng),
			Locale::ZH_TW => $faker_path(fake::locales::ZH_TW).fake_with_rng(&mut $ctx_ident.rng),
		}
	};
}

fn generate_util(i: &Info, ctx: &mut Ctx, util: &Util) -> anyhow::Result<()> {
	use fake::{Fake, /*Faker*/};
	use Util::*;
	let val = match util {
		FirstName => Val::String(gen_locale!(fake::faker::name::raw::FirstName, i, ctx)),
		LastName => Val::String(gen_locale!(fake::faker::name::raw::LastName, i, ctx)),
		Ref { pattern, relative } => resolve_ref(pattern, relative, &ctx)?.clone(),
	};

	println!("{:?}", ctx.current_path);
	println!("{:?}", ctx.generated);
	// *ctx.generated.pointer_mut(ctx.current_path.as_str())
	// 	.ok_or_else(|| anyhow::anyhow!("uh oh"))? = val;
	ctx.current_path.assign(&mut ctx.generated, val)?;

	Ok(())
}

fn resolve_ref<'c>(pattern: &PointerBuf, relative: &bool, ctx: &'c Ctx) -> anyhow::Result<&'c Val> {
	if *relative {
		// TODO this is what we maybe avoid by fully qualifying refs upfront
		// this allocation to create a fully qualified one
		Ok(ctx.current_path.concat(pattern).resolve(&ctx.generated)?)
	}
	else {
		Ok(pattern.resolve(&ctx.generated)?)
	}
}


fn generate_object(i: &Info, ctx: &mut Ctx, obj: &Obj) -> anyhow::Result<()> {
	let mut fields = obj.keys().collect::<Vec<_>>();
	let ordering = i.levels_fields.get(&ctx.current_path)
		.map(|o| o.as_slice())
		.unwrap_or(&[])
		.iter().collect::<Vec<_>>();
	let ordering = dbg!(ordering);
	// sort by the ordering
	fields.sort_by_key(|item| ordering.iter().position(|ord| ord.decoded() == item.as_str()).unwrap_or(usize::MAX));
	let ordered_fields = dbg!(fields);

	*ctx.current_path.resolve_mut(&mut ctx.generated)? = serde_json::json!({});

	for field_name in ordered_fields {
		println!("field_name={field_name}");

		let field_decl = obj.get(field_name)
			.ok_or_else(|| anyhow::anyhow!("couldn't find declaration for {field_name}"))?;

		ctx.current_path.push_back(field_name);
		generate_declaration(i, ctx, field_decl)?;
		ctx.current_path.pop_back();
	}

	Ok(())
}


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
