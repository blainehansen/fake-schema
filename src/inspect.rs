use std::collections::HashMap;
use crate::{parse::{self, InputDeclaration, InputUtil}, utils, Declaration, Util};
use jsonptr::PointerBuf;


pub(crate) type Tok = jsonptr::Token<'static>;

// pub(crate) type Levels = HashMap<PointerBuf, Vec<Tok>>;
pub(crate) type Levels = HashMap<PointerBuf, Vec<PointerBuf>>;

#[derive(Debug, Default)]
struct LevelDag {
	node_indices: HashMap<PointerBuf, daggy::NodeIndex>,
	dag: daggy::Dag<PointerBuf, utils::Emp>,
}

pub(crate) fn inspect_input_declaration(decl: InputDeclaration) -> anyhow::Result<(Levels, Declaration)> {
	let mut dags = HashMap::new();
	let mut current_path = PointerBuf::new();

	let decl = inspect_decl(&mut dags, &mut current_path, decl)?;

	let mut new_dags = HashMap::new();
	for (level_name, level) in dags {
		let graph = level.dag.graph();
		use daggy::petgraph::visit::IntoNodeReferences;
		let node_map = graph.node_references().collect::<HashMap<_, _>>();

		let ordered_fields = daggy::petgraph::algo::toposort(graph, None)
			.map_err(|e| anyhow::anyhow!("{e:?}"))?
			.into_iter().map(|node_idx| node_map[&node_idx].to_owned())
			.collect::<Vec<_>>();

		new_dags.insert(level_name, ordered_fields);
	}

	Ok((new_dags, decl))
}

fn inspect_decl(
	dags: &mut HashMap<PointerBuf, LevelDag>,
	current_path: &mut PointerBuf,
	decl: InputDeclaration,
) -> anyhow::Result<Declaration> {
	match decl {
		InputDeclaration::ShorthandUtil(parse::ShorthandUtil(util)) | InputDeclaration::LonghandUtil(util)
			=> Ok(Declaration::Util(inspect_util(dags, current_path, util)?)),
		InputDeclaration::Object(obj)
			=> Ok(Declaration::Object(inspect_object(dags, current_path, obj)?)),
	}
}

fn inspect_object(
	dags: &mut HashMap<PointerBuf, LevelDag>,
	current_path: &mut PointerBuf,
	decl: HashMap<String, InputDeclaration>,
) -> anyhow::Result<HashMap<String, Declaration>> {
	let mut give = HashMap::new();

	for (field_name, field_decl) in decl {
		current_path.push_back(&field_name);

		let new_field_decl = inspect_decl(dags, current_path, field_decl)?;
		give.insert(field_name, new_field_decl);

		current_path.pop_back();
	}

	Ok(give)
}

fn inspect_util(
	dags: &mut HashMap<PointerBuf, LevelDag>,
	current_path: &PointerBuf,
	util: InputUtil,
) -> anyhow::Result<Util> {
	match util {
		InputUtil::Ref { pattern: pat } => {
			let (pat, pat_relative) = ref_to_pointer(pat);
			let (common_prefix, pointing_name, pointed_name) = find_ref_parts(&current_path, &pat, &pat_relative)?;

			let node_level = dags.entry(common_prefix).or_default();
			let pointing = node_level.node_indices.entry(pointing_name.clone()).or_insert_with(|| node_level.dag.add_node(pointing_name)).to_owned();
			let pointed = node_level.node_indices.entry(pointed_name.clone()).or_insert_with(|| node_level.dag.add_node(pointed_name)).to_owned();
			if node_level.dag.find_edge(pointed, pointing).is_none() {
				node_level.dag.add_edge(pointed, pointing, utils::Emp)?;
			}
			Ok(Util::Ref { pattern: pat, relative: pat_relative })
		},
		InputUtil::FirstName => Ok(Util::FirstName),
		InputUtil::LastName => Ok(Util::LastName),
	}
}

fn find_ref_parts(current_path: &PointerBuf, pat: &PointerBuf, pat_relative: &bool) -> anyhow::Result<(PointerBuf, PointerBuf, PointerBuf)> {
	// println!("current_path={current_path} pat={pat} pat_relative={pat_relative}");
	if *pat_relative {
		let common_prefix = current_path.parent().unwrap();
		let pointed_suffix = pat;
		let pointing_suffix = current_path.strip_prefix(&common_prefix).unwrap();
		// println!("common_prefix={common_prefix} pointed_suffix={pointed_suffix} pointing_suffix={pointing_suffix}");
		let pointing = PointerBuf::from_tokens([pointing_suffix.first().unwrap()]);
		let pointed = PointerBuf::from_tokens([pointed_suffix.first().unwrap()]);
		return Ok((common_prefix.to_buf(), pointing, pointed))
	}

	let common_prefix = current_path.intersection(pat);
	let pointing_suffix = current_path.strip_prefix(common_prefix).unwrap();
	let pointed_suffix = pat.strip_prefix(common_prefix).unwrap();

	let pointing_empty = pointing_suffix.is_empty();
	let pointed_empty = pointed_suffix.is_empty();

	if pointing_empty && pointed_empty {
		return Err(anyhow::anyhow!("pointing to itself? {current_path} => {pat}"));
	}
	if pointing_empty || pointed_empty {
		return Err(anyhow::anyhow!("this doesn't make sense? {current_path} => {pat}"));
	}

	// println!("common_prefix={common_prefix} pointed_suffix={pointed_suffix} pointing_suffix={pointing_suffix}");
	let pointing = PointerBuf::from_tokens([pointing_suffix.first().unwrap()]);
	let pointed = PointerBuf::from_tokens([pointed_suffix.first().unwrap()]);
	Ok((common_prefix.to_buf(), pointing, pointed))
}

fn ref_to_pointer(pat: impl AsRef<str>) -> (PointerBuf, bool) {
	let pat = pat.as_ref();
	let pat_absolute = pat.starts_with('/');
	let pat =
		if pat_absolute { PointerBuf::from_tokens(pat.split('/').skip(1)) }
		else { PointerBuf::from_tokens(pat.split('/')) };
	(pat, !pat_absolute)
}

#[cfg(test)]
mod tests {
	use super::*;
	fn pref(cur: &str, pat: &str) -> anyhow::Result<(PointerBuf, PointerBuf, PointerBuf)> {
		let cur = PointerBuf::parse(cur).unwrap();
		let (pat, pat_relative) = ref_to_pointer(pat);
		find_ref_parts(&cur, &pat, &pat_relative)
	}
	fn tup(a: &str, b: &str, c: &str) -> (PointerBuf, PointerBuf, PointerBuf) {
		(a.parse().unwrap(), b.parse().unwrap(), c.parse().unwrap())
	}
	// fn v(strs: &[&str]) -> Vec<String> {
	// 	strs.iter().map(|s| s.to_string()).collect()
	// }

	fn p(s: &'static str) -> &'static jsonptr::Pointer {
		jsonptr::Pointer::from_static(s)
	}
	fn b(s: &'static str) -> PointerBuf {
		jsonptr::Pointer::from_static(s).to_buf()
	}

	#[test]
	fn test_ref_to_pointer() {
		assert_eq!(ref_to_pointer("b"), (b("/b"), true));
		assert_eq!(ref_to_pointer("/b"), (b("/b"), false));

		assert_eq!(ref_to_pointer("//a/df//bb/d"), (b("//a/df//bb/d"), false));
		assert_eq!(PointerBuf::parse("//a/df//bb/d").unwrap(), p("//a/df//bb/d"));
		assert_eq!(ref_to_pointer("a/df//bb/d"), (b("/a/df//bb/d"), true));

		assert_eq!(ref_to_pointer("//a/~df//bb/d"), (b("//a/~0df//bb/d"), false));
		assert!(PointerBuf::parse("//a/~df//bb/d").is_err());
		assert_eq!(ref_to_pointer("a/~df//bb/d"), (b("/a/~0df//bb/d"), true));
	}

	#[test]
	fn test_find_ref_parts() {
		assert_eq!(
			pref("/b", "/a").unwrap(),
			tup("", "/b", "/a"),
		);

		assert_eq!(
			pref("/b", "a").unwrap(),
			tup("", "/b", "/a"),
		);

		assert_eq!(
			pref("/b/c", "a").unwrap(),
			tup("/b", "/c", "/a"),
		);

		assert_eq!(
			pref("/b/c", "/a").unwrap(),
			tup("", "/b", "/a"),
		);

		assert_eq!(
			pref("/b", "/a/d/e").unwrap(),
			tup("", "/b", "/a"),
		);

		assert!(pref("/b/c", "/b/c").unwrap_err().to_string().contains("pointing to itself?"));

		assert!(pref("/b/c", "/b/c/d").unwrap_err().to_string().contains("this doesn't make sense?"));

		assert!(pref("/b/c/d", "/b/c").unwrap_err().to_string().contains("this doesn't make sense?"));
	}


	fn is_before(v: &Vec<String>, a: &str, b: &str) -> bool {
		let a_idx = v.iter().position(|x| x == a).unwrap();
		let b_idx = v.iter().position(|x| x == b).unwrap();
		a_idx < b_idx
	}
	fn deps_correct(fields: &HashMap<PointerBuf, Vec<Tok>>, pairs: HashMap<&str, &[(&str, &str)]>) -> bool {
		for (level_name, ordered_fields) in fields {
			for (a, b) in *pairs.get(level_name.as_str()).unwrap() {
				if is_before(ordered_fields, a, b) {
					return false
				}
			}
		}

		return true
	}

	#[test]
	fn test_inspect_input_declaration() {
		let decl: InputDeclaration = serde_json::from_str(r##"{
			"a": "FirstName",
			"b": { "#util": "Ref", "pattern": "a" },
			"c": { "#util": "Ref", "pattern": "/b" }
		}"##).unwrap();

		let deps = inspect_input_declaration(decl).unwrap().0;
		assert!(deps_correct(&deps, HashMap::from([
			("", [
				("/b", "/a"),
				("/c", "/b"),
			].as_slice()),
		])));
		assert_eq!(deps.len(), 1);
		assert_eq!(deps.get(p("")).unwrap().len(), 3);


		let decl: InputDeclaration = serde_json::from_str(r##"{
			"obj": {
				"o3": "/a",
				"o2": "o1",
				"o1": "FirstName",
				"o4": "/obj/o2"
			},
			"a": "FirstName",
			"d": "obj/o3",
			"b": "/a",
			"c": "b"
		}"##).unwrap();

		let deps = inspect_input_declaration(decl).unwrap().0;
		assert!(deps_correct(&deps, HashMap::from([
			("", [
				("/b", "/a"),
				("/obj", "/a"),
				("/d", "/obj"),
				("/c", "/b"),
			].as_slice()),
			("/obj", &[
				("/o2", "/o1"),
				("/o4", "/o2"),
			]),
		])));
		assert_eq!(deps.len(), 2);
		assert_eq!(deps.get(p("")).unwrap().len(), 5);
		assert_eq!(deps.get(p("/obj")).unwrap().len(), 3);


		let decl: InputDeclaration = serde_json::from_str(r##"{
			"a": "FirstName",
			"b": "LastName",
			"obj": { "o2": "o1", "o1": "FirstName" },
			"s": { "k1": "FirstName", "k2": "LastName" }
		}"##).unwrap();

		let deps = inspect_input_declaration(decl).unwrap().0;
		assert!(deps_correct(&deps, HashMap::from([
			("/obj", [
				("/o2", "/o1"),
			].as_slice()),
		])));
		assert_eq!(deps.len(), 1);
		assert_eq!(deps.get(p("")), None);
		assert_eq!(deps.get(p("/s")), None);
		assert_eq!(deps.get(p("/obj")).unwrap().len(), 2);
	}
}
