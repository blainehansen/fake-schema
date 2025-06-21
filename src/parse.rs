use std::collections::HashMap;

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
pub(crate) enum InputDeclaration {
	ShorthandUtil(ShorthandUtil),
	LonghandUtil(InputUtil),
	Object(HashMap<String, InputDeclaration>),
}

#[derive(Debug, serde::Deserialize)]
#[serde(try_from = "String")]
pub(crate) struct ShorthandUtil(pub InputUtil);

impl TryFrom<String> for ShorthandUtil {
	type Error = anyhow::Error;
	fn try_from(input: String) -> Result<Self, Self::Error> {
		let util = all_consuming(parse_util).parse_complete(input.as_str())
			.map(|(_, spec)| spec)
			.map_err(|e| anyhow::anyhow!("{e}"))?;

		Ok(ShorthandUtil(util))
	}
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "#util")]
pub enum InputUtil {
	Ref{ pattern: String },
	FirstName,
	LastName,
}


use nom::{
 IResult, Parser,
 branch::alt,
 bytes::complete::tag,
 character::complete::{char as ch, alpha1, alphanumeric1},
 combinator::{all_consuming, recognize, map, value},
 multi::{many0, separated_list1},
 sequence::pair,
 // sequence::{delimited, separated_pair},
};

fn parse_util(input: &str) -> IResult<&str, InputUtil> {
	alt((
		parse_atom,
		parse_ref,
	)).parse(input)
}

fn ident(input: &str) -> IResult<&str, &str> {
	recognize(
		pair(
			alt((alpha1, tag("_"))),
			many0(alt((alphanumeric1, tag("_"))))
		),
	).parse(input)
}

fn parse_ref(input: &str) -> IResult<&str, InputUtil> {
	map(alt((
		recognize(pair(ch('/'), separated_list1(ch('/'), ident))),
		recognize(separated_list1(ch('/'), ident)),
	)), |pattern| InputUtil::Ref { pattern: pattern.to_string() }).parse(input)
}

fn parse_atom(input: &str) -> IResult<&str, InputUtil> {
	alt((
		value(InputUtil::FirstName, tag("FirstName")),
		value(InputUtil::LastName, tag("LastName")),
	)).parse(input)
}


// #[derive(Debug, Deserialize)]
// #[serde(try_from = "String")]
// enum Spec {
//  Fake(FakeSpec),
//  StrJoin{ join_string: String, references: Vec<String> },
// }

// impl TryFrom<String> for Spec {
//  type Error = anyhow::Error;

//  fn try_from(input: String) -> Result<Self, Self::Error> {
//    all_consuming(spec).parse_complete(input.as_str())
//      .map(|(_, spec)| spec)
//      .map_err(|e| anyhow::anyhow!("{e}"))
//  }
// }

// fn spec(input: &str) -> IResult<&str, Spec> {
//  alt((
//    map(fake_spec, Spec::Fake),
//    str_join,
//  )).parse(input)
// }
// fn str_join(input: &str) -> IResult<&str, Spec> {
//  let (input, (join_string, strs)) = delimited(
//    tag("StrJoin("),
//    separated_pair(delimited(nom::character::char('\''), is_not("'"), nom::character::char('\'')), tag(", "), separated_list1(tag(", "), ident)),
//    tag(")"),
//  ).parse(input)?;

//  Ok((input, Spec::StrJoin {
//    join_string: join_string.to_string(),
//    references: strs.iter().map(|s| s.to_string()).collect(),
//  }))
// }

// fn ident(input: &str) -> IResult<&str, &str> {
//  take_while1(|i: char| i == '_' || i.is_alphanumeric()).parse(input)
// }


// #[derive(Debug, Clone)]
// enum FakeSpec {
//  Primitive(FakePrimitive),
//  Maybe { inner: FakePrimitive, some_weight: f32 },
// }
// fn fake_spec(input: &str) -> IResult<&str, FakeSpec> {
//  alt((
//    map(fake_primitive, FakeSpec::Primitive),
//    maybe,
//  )).parse(input)
// }
// fn maybe(input: &str) -> IResult<&str, FakeSpec> {
//  map(
//    delimited(tag("Maybe("), separated_pair(fake_primitive, tag(", "), |i| number::float().parse(i)), tag(")")),
//    |(inner, some_weight)| FakeSpec::Maybe { inner, some_weight },
//  ).parse(input)
// }

// #[derive(Debug, Clone)]
// enum FakePrimitive {
//  FirstName,
//  LastName,
// }
// fn fake_primitive(input: &str) -> IResult<&str, FakePrimitive> {
//  alt((
//    value(FakePrimitive::FirstName, tag("FirstName")),
//    value(FakePrimitive::LastName, tag("LastName")),
//  )).parse(input)
// }


// // fn yo<I, O, E: nom::error::ParseError<I>, F, G>(parser: F, f: G) -> impl Parser<I, Output = O, Error = E>
// // where
// //   F: Parser<I, Error = E>,
// //   G: FnMut(<F as Parser<I>>::Output) -> O,
// // {
// //   parser.map(f)
// // }
