use derive_more::{Display, LowerHex, UpperHex};
use erroneous::Error as EError;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{de, Deserialize, Deserializer};
use std::{ops::Deref, str::FromStr};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Display, PartialEq, Eq, Hash, UpperHex, LowerHex)]
#[display(fmt = "{:X} ({})", "_0.0", "self.url()")]
pub struct Item(steam::Item);

impl Item {
	pub fn url(&self) -> String {
		format!(
			"https://steamcommunity.com/sharedfiles/filedetails/?id={}",
			(self.0).0
		)
	}
}

impl From<steam::Item> for Item {
	fn from(i: steam::Item) -> Item {
		Item(i)
	}
}

impl Deref for Item {
	type Target = steam::Item;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

#[derive(Debug, Display, EError)]
#[display(fmt = "Could not find any workshop item ID in the input")]
pub struct ItemParseError;

impl FromStr for Item {
	type Err = ItemParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		lazy_static! {
			// Intentionally not `\?id`
			static ref ID_RE: Regex = Regex::new(r"id=([0-9]*)\b").expect("Could not generate regex");
		}

		let i = u64::from_str_radix(s, 16)
			.ok()
			.or_else(|| ID_RE.captures(s).and_then(|s| s[1].parse().ok()))
			.ok_or(ItemParseError)?;

		Ok(Item(steam::Item(i)))
	}
}

impl<'a> Deserialize<'a> for Item {
	fn deserialize<D: Deserializer<'a>>(d: D) -> Result<Self, D::Error> {
		let s = String::deserialize(d)?;
		FromStr::from_str(&s).map_err(de::Error::custom)
	}
}
