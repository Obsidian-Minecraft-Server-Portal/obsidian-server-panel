use std::collections::BTreeMap;
use std::sync::LazyLock;

use regex::Regex;

pub(crate) const ANONYMOUS_UUID: &str = "00000000-0000-0000-0000-000000000000";

static ANSI: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\x1b\[[0-9;?]*[A-Za-z]").expect("ansi pattern is valid"));
static JOINED: LazyLock<Regex> = LazyLock::new(|| {
	Regex::new(r"^([A-Za-z0-9_.]{1,16})(?: \(formerly known as [A-Za-z0-9_.]{1,16}\))? joined the game$")
		.expect("join pattern is valid")
});
static LEFT: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r"^([A-Za-z0-9_.]{1,16}) left the game$").expect("leave pattern is valid"));
static LISTED: LazyLock<Regex> = LazyLock::new(|| {
	Regex::new(r"^There are \d+ of a max of \d+ players online:(.*)$").expect("list pattern is valid")
});

#[derive(Debug, PartialEq)]
enum Event {
	Joined(String),
	Left(String),
	Listed(Vec<String>),
}

#[derive(Debug, Default)]
pub struct Roster(BTreeMap<String, String>);

impl Roster {
	pub fn observe(&mut self, line: &str) {
		match parse(line) {
			Some(Event::Joined(name)) => {
				self.0.insert(name.to_ascii_lowercase(), name);
			}
			Some(Event::Left(name)) => {
				self.0.remove(&name.to_ascii_lowercase());
			}
			Some(Event::Listed(names)) => {
				self.0 = names.into_iter().map(|name| (name.to_ascii_lowercase(), name)).collect();
			}
			None => {}
		}
	}

	pub fn clear(&mut self) {
		self.0.clear();
	}

	pub fn names(&self) -> impl Iterator<Item = &String> {
		self.0.values()
	}
}

/// The first `]: ` ends the logger prefix; anything after it is message text a player may control.
fn parse(line: &str) -> Option<Event> {
	let plain = ANSI.replace_all(line, "");
	let message = plain.split_once("]: ").map_or(plain.as_ref(), |(_, message)| message).trim();
	if let Some(captures) = JOINED.captures(message) {
		return Some(Event::Joined(captures[1].to_owned()));
	}
	if let Some(captures) = LEFT.captures(message) {
		return Some(Event::Left(captures[1].to_owned()));
	}
	let captures = LISTED.captures(message)?;
	let names = captures[1].split(',').map(str::trim).filter(|name| !name.is_empty()).map(str::to_owned).collect();
	Some(Event::Listed(names))
}

#[cfg(test)]
mod tests {
	use super::{Event, Roster, parse};

	#[test]
	fn parses_join_and_leave_lines() {
		assert_eq!(
			parse("[18:30:50] [Server thread/INFO]: Shroototem joined the game"),
			Some(Event::Joined("Shroototem".into()))
		);
		assert_eq!(
			parse("[18:30:50] [Server thread/INFO] [minecraft/MinecraftServer]: Steve left the game"),
			Some(Event::Left("Steve".into()))
		);
		assert_eq!(
			parse("[12:00:00 INFO]: \x1b[0;33;22mAlex (formerly known as Al) joined the game\x1b[m"),
			Some(Event::Joined("Alex".into()))
		);
	}

	#[test]
	fn ignores_chat_that_imitates_joins() {
		assert_eq!(parse("[Server thread/INFO]: <Bob> Steve joined the game"), None);
		assert_eq!(parse("[Server thread/INFO]: [Bob] x]: Steve joined the game"), None);
	}

	#[test]
	fn list_output_replaces_the_roster() {
		let mut roster = Roster::default();
		roster.observe("[Server thread/INFO]: Ghost joined the game");
		roster.observe("[18:30:50] [Server thread/INFO]: There are 2 of a max of 20 players online: Shroototem, dcman58");
		assert_eq!(roster.names().cloned().collect::<Vec<_>>(), ["dcman58", "Shroototem"]);
		roster.observe("[Server thread/INFO]: There are 0 of a max of 20 players online: ");
		assert_eq!(roster.names().count(), 0);
	}
}
