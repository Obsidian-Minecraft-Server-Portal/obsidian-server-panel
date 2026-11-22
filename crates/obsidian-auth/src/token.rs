const TOKEN_BYTES: usize = 32;
const CODE_DIGITS: u32 = 6;
const CODE_MODULUS: u32 = 10_u32.pow(CODE_DIGITS);
const RECOVERY_BYTES: usize = 5;

pub fn generate_token() -> String {
	hex::encode(rand::random::<[u8; TOKEN_BYTES]>())
}

pub fn generate_code() -> String {
	format!(
		"{:0width$}",
		rand::random_range(0..CODE_MODULUS),
		width = CODE_DIGITS as usize
	)
}

pub fn generate_recovery_code() -> String {
	let raw = hex::encode(rand::random::<[u8; RECOVERY_BYTES]>());
	format!("{}-{}", &raw[..5], &raw[5..])
}

pub fn hash_token(token: &str) -> String {
	blake3::hash(token.as_bytes()).to_hex().to_string()
}
