#[macro_use]
extern crate nom;

use nom::{types::CompleteStr, Err};

use std::collections::HashMap;

named!(ident<CompleteStr, CompleteStr>, take_while!(|c: char| c.is_alphanumeric() || c == '_'));

named!(space<CompleteStr, CompleteStr>, take_while!(char::is_whitespace));

named!(declaration<CompleteStr, (CompleteStr, CompleteStr)>,
	do_parse!(
		opt!(space) >>
		key: ident >>
		opt!(space) >>
		tag!("=") >>
		opt!(space) >>
		val: alt!(
			delimited!(
				tag!("\""),
				take_until!("\""),
				tag!("\"")
			) |
			do_parse!(
				tag!("[") >>
				spaces: fold_many0!(tag!("="), 0, |acc: usize, _| acc + 1) >>
				tag!("[") >>
				content: take_until_and_consume!(("]".to_owned() + &"=".repeat(spaces) + "]").as_str()) >>
				(content)
			)
		) >>
		(key, val)
	)
);

named!(declarations<CompleteStr, Vec<(CompleteStr, CompleteStr)> >, many0!(declaration));

pub type Error<'a> = Err<CompleteStr<'a>>;
pub fn parse(input: &str) -> Result<HashMap<&str, &str>, Error> {
	let decls = declarations(input.into())?;
	Ok(decls.1.into_iter().map(|(a, b)| (*a, *b)).collect())
}

#[cfg(test)]
#[test]
fn test() {
	const EXAMPLE: &str = include_str!("example.lua");
	let map = parse(EXAMPLE).unwrap();
	assert_eq!(map["name"], "NS2 Community Fixes");
	assert_eq!(map["source_dir"], "");
	assert_eq!(map["output_dir"], "output");
	assert_eq!(map["description"], "test description[]][");
	assert_eq!(map["image"], "preview.jpg");
	assert_eq!(map["tag_modtype"], "Gameplay Tweak");
	assert_eq!(map["tag_support"], "Must be run on Server");
	assert_eq!(map["publish_id"], "4292cdec");
}
