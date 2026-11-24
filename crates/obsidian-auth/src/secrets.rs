use std::fs::OpenOptions;
use std::io::{ErrorKind, Write};
use std::path::Path;

use chacha20poly1305::aead::Aead;
use chacha20poly1305::{ChaCha20Poly1305, KeyInit};
use obsidian_common::{CommonError, Result};

const KEY_BYTES: usize = 32;
const NONCE_BYTES: usize = 12;
const KEY_FILE: &str = "secret.key";

#[derive(Clone)]
pub struct SecretBox {
	cipher: ChaCha20Poly1305,
	key: [u8; KEY_BYTES],
}

impl std::fmt::Debug for SecretBox {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str("SecretBox")
	}
}

impl SecretBox {
	pub fn load_or_create(data_dir: &Path) -> Result<Self> {
		let path = data_dir.join(KEY_FILE);
		let key = match std::fs::read_to_string(&path) {
			Ok(text) => hex::decode(text.trim()).map_err(|e| CommonError::Other(format!("invalid secret key: {e}")))?,
			Err(error) if error.kind() == ErrorKind::NotFound => {
				let key = rand::random::<[u8; KEY_BYTES]>().to_vec();
				write_key(&path, &key)?;
				key
			}
			Err(error) => return Err(error.into()),
		};
		let key: [u8; KEY_BYTES] = key
			.try_into()
			.map_err(|_| CommonError::Other("secret key has an invalid length".into()))?;
		Ok(Self::from_key(key))
	}

	pub fn from_key(key: [u8; KEY_BYTES]) -> Self {
		Self { cipher: ChaCha20Poly1305::new(&key.into()), key }
	}

	/// BLAKE3 keyed hash under a key derived per `context`, so a tag made for one purpose never verifies for another.
	pub fn mac(&self, context: &str, message: &[u8]) -> String {
		self.keyed(context, message).to_hex().to_string()
	}

	pub fn verify_mac(&self, context: &str, message: &[u8], tag: &str) -> bool {
		blake3::Hash::from_hex(tag).is_ok_and(|tag| tag == self.keyed(context, message))
	}

	fn keyed(&self, context: &str, message: &[u8]) -> blake3::Hash {
		blake3::keyed_hash(&blake3::derive_key(context, &self.key), message)
	}

	pub fn seal(&self, plaintext: &[u8]) -> Result<String> {
		let nonce = rand::random::<[u8; NONCE_BYTES]>();
		let ciphertext = self
			.cipher
			.encrypt(&nonce.into(), plaintext)
			.map_err(|_| CommonError::Other("failed to encrypt secret".into()))?;
		Ok(hex::encode([nonce.as_slice(), ciphertext.as_slice()].concat()))
	}

	pub fn open(&self, sealed: &str) -> Result<Vec<u8>> {
		let bytes = hex::decode(sealed).map_err(|e| CommonError::Other(format!("corrupt secret: {e}")))?;
		if bytes.len() < NONCE_BYTES {
			return Err(CommonError::Other("corrupt secret".into()));
		}
		let (nonce, ciphertext) = bytes.split_at(NONCE_BYTES);
		let nonce: [u8; NONCE_BYTES] = nonce
			.try_into()
			.map_err(|_| CommonError::Other("corrupt secret".into()))?;
		self.cipher
			.decrypt(&nonce.into(), ciphertext)
			.map_err(|_| CommonError::Other("failed to decrypt secret".into()))
	}
}

fn write_key(path: &Path, key: &[u8]) -> Result<()> {
	let mut options = OpenOptions::new();
	options.write(true).create_new(true);
	#[cfg(unix)]
	{
		use std::os::unix::fs::OpenOptionsExt;
		options.mode(0o600);
	}
	let mut file = options.open(path)?;
	file.write_all(hex::encode(key).as_bytes())?;
	file.sync_all()?;
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::SecretBox;

	#[test]
	fn macs_verify_only_for_their_context_and_message() {
		let secrets = SecretBox::from_key([7; 32]);
		let tag = secrets.mac("a", b"message");
		assert!(secrets.verify_mac("a", b"message", &tag));
		assert!(!secrets.verify_mac("b", b"message", &tag));
		assert!(!secrets.verify_mac("a", b"other", &tag));
		assert!(!secrets.verify_mac("a", b"message", "zz"));
		assert!(!SecretBox::from_key([8; 32]).verify_mac("a", b"message", &tag));
	}
}
