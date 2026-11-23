type Version = [u32; 3];

const OPERATORS: [&str; 7] = ["<=", ">=", "<", ">", "=", "~", "^"];

#[derive(Clone, Copy)]
struct Interval {
	lower: Option<(Version, bool)>,
	upper: Option<(Version, bool)>,
}

impl Interval {
	fn contains(self, version: Version) -> bool {
		self.lower.is_none_or(|(bound, inclusive)| if inclusive { version >= bound } else { version > bound })
			&& self
				.upper
				.is_none_or(|(bound, inclusive)| if inclusive { version <= bound } else { version < bound })
	}
}

/// Whether a declared Minecraft range covers `minecraft`. `None` when either side cannot be read.
pub(crate) fn covers(range: &str, minecraft: &str) -> Option<bool> {
	let (version, precision, wildcard) = spec(minecraft)?;
	if wildcard || precision < 2 {
		return None;
	}
	let range = range.trim();
	if range.starts_with(['[', '(']) {
		maven_covers(range, version)
	} else {
		fabric_covers(range, version)
	}
}

fn fabric_covers(range: &str, version: Version) -> Option<bool> {
	let mut matched = false;
	for alternative in range.split("||") {
		let intervals = alternative
			.split(|character: char| character == ',' || character.is_whitespace())
			.filter(|token| !token.is_empty())
			.map(fabric_interval)
			.collect::<Option<Vec<Interval>>>()?;
		if intervals.is_empty() {
			return None;
		}
		matched |= intervals.iter().all(|interval| interval.contains(version));
	}
	Some(matched)
}

fn fabric_interval(token: &str) -> Option<Interval> {
	if matches!(token, "*" | "x" | "X") {
		return Some(Interval { lower: None, upper: None });
	}
	let (operator, text) = OPERATORS
		.into_iter()
		.find_map(|operator| token.strip_prefix(operator).map(|text| (operator, text)))
		.unwrap_or(("", token));
	let (version, precision, wildcard) = spec(text)?;
	let window = |upper| Interval {
		lower: Some((version, true)),
		upper: Some((upper, false)),
	};
	Some(match operator {
		">=" => Interval {
			lower: Some((version, true)),
			upper: None,
		},
		">" => Interval {
			lower: Some((version, false)),
			upper: None,
		},
		"<=" => Interval {
			lower: None,
			upper: Some((version, true)),
		},
		"<" => Interval {
			lower: None,
			upper: Some((version, false)),
		},
		"~" => window(bump(version, precision.clamp(1, 2))),
		"^" => window(bump(version, caret_index(version, precision) + 1)),
		"=" | "" if precision == 3 && !wildcard => Interval {
			lower: Some((version, true)),
			upper: Some((version, true)),
		},
		"=" | "" => window(bump(version, precision)),
		_ => return None,
	})
}

fn maven_covers(range: &str, version: Version) -> Option<bool> {
	let mut rest = range;
	let mut matched = false;
	let mut seen = false;
	while !rest.is_empty() {
		let open = rest.chars().next()?;
		let close_at = rest.find([']', ')'])?;
		let close = rest[close_at..].chars().next()?;
		let interval = maven_interval(open, &rest[1..close_at], close)?;
		matched |= interval.contains(version);
		seen = true;
		rest = rest[close_at + 1..].trim_start_matches(|character: char| character == ',' || character.is_whitespace());
	}
	seen.then_some(matched)
}

fn maven_interval(open: char, body: &str, close: char) -> Option<Interval> {
	let inclusive_lower = match open {
		'[' => true,
		'(' => false,
		_ => return None,
	};
	let inclusive_upper = match close {
		']' => true,
		')' => false,
		_ => return None,
	};
	match body.split_once(',') {
		None => {
			let (version, _, _) = spec(body)?;
			(inclusive_lower && inclusive_upper).then_some(Interval {
				lower: Some((version, true)),
				upper: Some((version, true)),
			})
		}
		Some((lower, upper)) => {
			let lower = match lower.trim() {
				"" => None,
				text => Some((spec(text)?.0, inclusive_lower)),
			};
			let upper = match upper.trim() {
				"" => None,
				text => Some((spec(text)?.0, inclusive_upper)),
			};
			Some(Interval { lower, upper })
		}
	}
}

fn spec(text: &str) -> Option<(Version, usize, bool)> {
	let mut parts: Vec<&str> = text.trim().split('.').collect();
	let wildcard = matches!(parts.last(), Some(&("x" | "X" | "*")));
	if wildcard {
		parts.pop();
	}
	if parts.is_empty() || parts.len() > 3 {
		return None;
	}
	let mut version = [0; 3];
	for (slot, part) in version.iter_mut().zip(&parts) {
		if part.is_empty() || !part.bytes().all(|byte| byte.is_ascii_digit()) {
			return None;
		}
		*slot = part.parse().ok()?;
	}
	Some((version, parts.len(), wildcard))
}

fn bump(version: Version, precision: usize) -> Version {
	let index = precision.clamp(1, 3) - 1;
	let mut next = [0; 3];
	next[..=index].copy_from_slice(&version[..=index]);
	next[index] += 1;
	next
}

fn caret_index(version: Version, precision: usize) -> usize {
	let precision = precision.clamp(1, 3);
	(0..precision).find(|&index| version[index] != 0).unwrap_or(precision - 1)
}

#[cfg(test)]
mod tests {
	use super::covers;

	#[test]
	fn fabric_ranges_cover_release_versions() {
		assert_eq!(covers(">=1.20", "1.20.1"), Some(true));
		assert_eq!(covers(">=1.20 <1.21", "1.20.4"), Some(true));
		assert_eq!(covers(">=1.20 <1.21", "1.21"), Some(false));
		assert_eq!(covers("1.20.x", "1.20.4"), Some(true));
		assert_eq!(covers("1.20.x", "1.21"), Some(false));
		assert_eq!(covers("~1.20.1", "1.20.4"), Some(true));
		assert_eq!(covers("~1.20.1", "1.21"), Some(false));
		assert_eq!(covers("^1.20.1", "1.21.4"), Some(true));
		assert_eq!(covers("1.20", "1.20.1"), Some(true));
		assert_eq!(covers("1.20", "1.21"), Some(false));
		assert_eq!(covers("1.20.1 || 1.21.4", "1.21.4"), Some(true));
		assert_eq!(covers("1.20.1 || 1.21.4", "1.21.3"), Some(false));
		assert_eq!(covers("*", "26.1"), Some(true));
		assert_eq!(covers(">1.20.1", "1.20.1"), Some(false));
		assert_eq!(covers("<=1.21.4", "1.21.4"), Some(true));
	}

	#[test]
	fn maven_intervals_cover_release_versions() {
		assert_eq!(covers("[1.20.1]", "1.20.1"), Some(true));
		assert_eq!(covers("[1.20.1]", "1.20.2"), Some(false));
		assert_eq!(covers("[1.20,1.21)", "1.20.4"), Some(true));
		assert_eq!(covers("[1.20,1.21)", "1.21"), Some(false));
		assert_eq!(covers("[1.20,)", "26.1"), Some(true));
		assert_eq!(covers("(,1.19)", "1.20"), Some(false));
		assert_eq!(covers("[1.19,1.20),[1.21,)", "1.21.4"), Some(true));
	}

	#[test]
	fn unreadable_ranges_and_snapshot_targets_are_unknown() {
		assert_eq!(covers("", "1.21"), None);
		assert_eq!(covers(">=banana", "1.21"), None);
		assert_eq!(covers("[1.20", "1.21"), None);
		assert_eq!(covers(">=1.20", "24w14a"), None);
		assert_eq!(covers(">=1.20", "1.21-rc1"), None);
	}
}
