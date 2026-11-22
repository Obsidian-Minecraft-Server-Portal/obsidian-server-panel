use std::io::Read;

use memchr::memmem;

use crate::archive;

const CLASS_LIMIT: u64 = 8 << 20;
const CLIENT_PACKAGE: &[u8] = b"net/minecraft/client/";
const SERVER_PACKAGE: &[u8] = b"net/minecraft/server/";
const FABRIC_ENVIRONMENT: &[u8] = b"Lnet/fabricmc/api/Environment;";
const FORGE_DIST: &[u8] = b"/distmarker/OnlyIn;";

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct ClassRefs {
	pub client: bool,
	pub server: bool,
}

pub(crate) fn scan(jar: &[u8]) -> ClassRefs {
	let Ok(mut archive) = archive::open(jar) else {
		return ClassRefs::default();
	};
	let mut refs = ClassRefs::default();
	for index in 0..archive.len() {
		let Ok(mut entry) = archive.by_index(index) else {
			continue;
		};
		let is_class = matches!(entry.name(), Ok(name) if name.ends_with(".class"));
		if !is_class || entry.size() > CLASS_LIMIT {
			continue;
		}
		let mut bytes = Vec::with_capacity(entry.size() as usize);
		if entry.read_to_end(&mut bytes).is_err() {
			continue;
		}
		let (client, server) = class_refs(&bytes);
		refs.client |= client;
		refs.server |= server;
		if refs.client && refs.server {
			break;
		}
	}
	refs
}

fn class_refs(bytes: &[u8]) -> (bool, bool) {
	let mut client = memmem::find(bytes, CLIENT_PACKAGE).is_some();
	let mut server = memmem::find(bytes, SERVER_PACKAGE).is_some();
	if memmem::find(bytes, FABRIC_ENVIRONMENT).is_some() || memmem::find(bytes, FORGE_DIST).is_some() {
		client |= memmem::find(bytes, b"CLIENT").is_some();
		server |= memmem::find(bytes, b"SERVER").is_some();
	}
	(client, server)
}
