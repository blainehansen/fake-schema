use std::collections::HashMap;
use crate::{utils, parse::{self, InputDeclaration}, Declaration, Util, Pattern};

pub(crate) type Levels = HashMap<String, Vec<String>>;

#[derive(Debug, Default)]
struct LevelDag {
	node_indices: HashMap<String, daggy::NodeIndex>,
	dag: daggy::Dag<String, utils::Emp>,
}

pub(crate) fn inspect_input_declaration(decl: InputDeclaration) -> anyhow::Result<(Levels, Declaration)> {
	let mut dags = HashMap::from([
		("~".to_string(), LevelDag { node_indices: Default::default(), dag: daggy::Dag::new() }),
	]);

	let mut current_path = vec!["~".to_string()];

	let decl = inspect_decl(&mut dags, &mut current_path, decl)?;

	let mut new_dags = HashMap::new();
	for (level_name, level) in dags {
		let graph = level.dag.graph();
		use daggy::petgraph::visit::IntoNodeReferences;
		let node_map = graph.node_references().collect::<HashMap<_, _>>();

		let ordered_fields = daggy::petgraph::algo::toposort(graph, None)
			.map_err(|e| anyhow::anyhow!("{e:?}"))?
			.into_iter().map(|node_idx| node_map[&node_idx].to_string())
			.collect::<Vec<_>>();

		new_dags.insert(level_name, ordered_fields);
	}

	Ok((new_dags, decl))
}

fn inspect_decl(
	dags: &mut HashMap<String, LevelDag>,
	current_path: &mut Vec<String>,
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
	dags: &mut HashMap<String, LevelDag>,
	current_path: &mut Vec<String>,
	decl: HashMap<String, InputDeclaration>,
) -> anyhow::Result<HashMap<String, Declaration>> {
	let mut give = HashMap::new();

	for (field_name, field_decl) in decl {
		current_path.push(field_name.to_string());

		let new_field_decl = match field_decl {
			InputDeclaration::ShorthandUtil(parse::ShorthandUtil(util)) | InputDeclaration::LonghandUtil(util)
				=> Declaration::Util(inspect_util(dags, current_path, util)?),
			InputDeclaration::Object(obj_decl)
				=> Declaration::Object(inspect_object(dags, current_path, obj_decl)?),
		};
		give.insert(field_name, new_field_decl);

		current_path.pop();
	}

	Ok(give)
}

fn inspect_util(
	dags: &mut HashMap<String, LevelDag>,
	current_path: &mut Vec<String>,
	util: Util,
) -> anyhow::Result<Util> {
	match util {
		Util::Ref(pat) => {
			let (common_prefix, depending_name, depended_name) = find_common_prefix(&current_path, &pat)?;
			let fully_qualified = [common_prefix.as_str(), depended_name.as_str()].join(".");

			let node_level = dags.entry(common_prefix).or_default();
			let depending = node_level.node_indices.entry(depending_name.clone()).or_insert_with(|| node_level.dag.add_node(depending_name)).clone();
			let depended = node_level.node_indices.entry(depended_name.clone()).or_insert_with(|| node_level.dag.add_node(depended_name)).clone();
			if node_level.dag.find_edge(depended, depending).is_none() {
				node_level.dag.add_edge(depended, depending, utils::Emp)?;
			}
			Ok(Util::Ref(fully_qualified))
		},
		util => Ok(util),
	}
}

fn find_common_prefix(current_path: &Vec<String>, pat: &Pattern) -> anyhow::Result<(String, String, String)> {
	let split_pat: Vec<_> =
		if pat.starts_with("~") {
			pat.split(".").map(|i| i.to_string()).collect()
		} else {
			current_path.iter().take(current_path.len() - 1).map(|i| i.to_string())
				.chain(pat.split(".").map(|i| i.to_string()))
				.collect()
		};
	// println!("current_path={current_path:?} split_pat={split_pat:?} pat={pat:?}");

	let mut common_prefix = vec![];
	for (current_segment, pat_segment) in utils::zip_longest(current_path, split_pat) {
		// println!("{current_segment:?}, {pat_segment:?}");
		match (current_segment, pat_segment) {
			(Some(current_segment), Some(pat_segment)) => {
				if *current_segment == pat_segment {
					common_prefix.push(pat_segment);
				} else {
					// here this means they aren't equal, which means we're done
					// the point we're currently at *needs* the segment referred to in the pattern
					return Ok((common_prefix.join("."), current_segment.to_string(), pat_segment.to_string()))
				}
			},
			// we have a pattern that's longer than our current point
			// ~.country.locale => ~.country
			// this doesn't make sense, this is pointing merely back up the tree, but without diverting at all, so this would expand infinitely

			// we have a pattern that's shorter than our current point
			// ~.country => ~.country.locale
			// this doens't make sense, because it's pointing downward at itself, trying to say the full object should be one of it's own fields
			_ => {
				return Err(anyhow::anyhow!("this doesn't make sense? {pat:?} => {current_path:?}"));
			},
		}
	}

	// if we get to this point that means they're exactly equal, which means the pat is empty? something's wrong
	Err(anyhow::anyhow!("pointing to itself? {pat:?} => {current_path:?}"))
}
