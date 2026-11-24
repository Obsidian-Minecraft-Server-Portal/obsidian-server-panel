use sha1::{Digest, Sha1};

const CURSEFORGE_SEED: u32 = 1;

pub(super) struct FileHashes {
	pub sha1: String,
	pub fingerprint: u32,
}

pub(super) fn hash(bytes: &[u8]) -> FileHashes {
	FileHashes {
		sha1: hex::encode(Sha1::digest(bytes)),
		fingerprint: curseforge_fingerprint(bytes),
	}
}

pub(super) fn curseforge_fingerprint(bytes: &[u8]) -> u32 {
	let normalized: Vec<u8> = bytes
		.iter()
		.copied()
		.filter(|byte| !matches!(byte, 9 | 10 | 13 | 32))
		.collect();
	murmur2::murmur2(&normalized, CURSEFORGE_SEED)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn curseforge_fingerprint_ignores_whitespace() {
		assert_eq!(curseforge_fingerprint(b"a b\tc\r\nd"), curseforge_fingerprint(b"abcd"));
		assert_ne!(curseforge_fingerprint(b"abcd"), curseforge_fingerprint(b"abce"));
	}

	#[test]
	fn sha1_is_lowercase_hex() {
		assert_eq!(hash(b"abc").sha1, "a9993e364706816aba3e25717850c26c9cd0d89d");
	}
}
