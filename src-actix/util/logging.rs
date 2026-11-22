use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use chrono::{Local, Timelike};
use tar::Builder;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::{Layer, fmt, prelude::*};
use xz2::stream::{Check, Stream};
use xz2::write::XzEncoder;

const MAX_BYTES: u64 = 50 * 1024 * 1024;
const XZ_PRESET: u32 = 6;
const LOG_ENV: &str = "OBSIDIAN_LOG";
const DEFAULT_DIRECTIVES: &str = "info,obsidian=debug,sqlx=warn";
const ARCHIVING_DIR: &str = "archiving";
const LOG_FILES: [&str; 4] = ["debug.log", "latest.log", "error.log", "logs.ndjson"];
const DEBUG: usize = 0;
const LATEST: usize = 1;
const ERROR: usize = 2;
const JSON: usize = 3;

struct RollingLogs {
	dir: PathBuf,
	files: [File; 4],
	sizes: [u64; 4],
}

impl RollingLogs {
	fn open(dir: PathBuf) -> io::Result<Self> {
		let files = [
			fresh_file(&dir, LOG_FILES[DEBUG])?,
			fresh_file(&dir, LOG_FILES[LATEST])?,
			fresh_file(&dir, LOG_FILES[ERROR])?,
			fresh_file(&dir, LOG_FILES[JSON])?,
		];
		Ok(Self { dir, files, sizes: [0; 4] })
	}

	fn write_to(&mut self, target: usize, buf: &[u8]) -> io::Result<()> {
		self.files[target].write_all(buf)?;
		self.sizes[target] += buf.len() as u64;
		if self.sizes[target] >= MAX_BYTES {
			self.roll()?;
		}
		Ok(())
	}

	fn roll(&mut self) -> io::Result<()> {
		let staging = self.dir.join(ARCHIVING_DIR).join(archive_stem());
		fs::create_dir_all(&staging)?;
		for (index, file) in self.files.iter_mut().enumerate() {
			file.flush()?;
			fs::rename(self.dir.join(LOG_FILES[index]), staging.join(LOG_FILES[index]))?;
			*file = fresh_file(&self.dir, LOG_FILES[index])?;
			self.sizes[index] = 0;
		}
		let dir = self.dir.clone();
		std::thread::Builder::new().name("log-archiver".into()).spawn(move || {
			if archive(&staging, &dir).is_ok() {
				let _ = fs::remove_dir_all(&staging);
			}
		})?;
		Ok(())
	}
}

#[derive(Clone)]
struct LogWriter {
	inner: Arc<Mutex<RollingLogs>>,
	target: usize,
}

impl Write for LogWriter {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		self.inner.lock().unwrap_or_else(|e| e.into_inner()).write_to(self.target, buf)?;
		Ok(buf.len())
	}

	fn flush(&mut self) -> io::Result<()> {
		self.inner.lock().unwrap_or_else(|e| e.into_inner()).files[self.target].flush()
	}
}

impl<'a> MakeWriter<'a> for LogWriter {
	type Writer = LogWriter;
	fn make_writer(&'a self) -> Self::Writer {
		self.clone()
	}
}

pub fn setup_logging(dir: Option<&Path>) -> color_eyre::Result<()> {
	let console_filter = if crate::DEBUG { directives() } else { EnvFilter::new("info") };
	let console = fmt::layer().with_thread_names(true).with_writer(io::stdout).with_filter(console_filter);
	let Some(dir) = dir else {
		tracing_subscriber::registry().with(console).init();
		return Ok(());
	};

	fs::create_dir_all(dir)?;
	if has_existing_logs(dir) {
		archive(dir, dir)?;
	}
	let logs = Arc::new(Mutex::new(RollingLogs::open(dir.to_path_buf())?));
	let writer = |target| LogWriter { inner: Arc::clone(&logs), target };
	let text = |target, level| {
		fmt::layer()
			.with_ansi(false)
			.with_thread_names(true)
			.with_writer(writer(target))
			.with_filter(level)
			.boxed()
	};

	let layers = vec![
		text(DEBUG, directives()),
		text(LATEST, EnvFilter::new("info")),
		text(ERROR, EnvFilter::new("error")),
		fmt::layer().json().with_thread_names(true).with_writer(writer(JSON)).with_filter(directives()).boxed(),
		console.boxed(),
	];
	tracing_subscriber::registry().with(layers).init();
	Ok(())
}

fn has_existing_logs(dir: &Path) -> bool {
	LOG_FILES
		.iter()
		.any(|name| dir.join(name).metadata().map(|m| m.len() > 0).unwrap_or(false))
}

fn fresh_file(dir: &Path, name: &str) -> io::Result<File> {
	OpenOptions::new().create(true).write(true).truncate(true).open(dir.join(name))
}

fn directives() -> EnvFilter {
	EnvFilter::try_from_env(LOG_ENV).unwrap_or_else(|_| EnvFilter::new(DEFAULT_DIRECTIVES))
}

fn archive_stem() -> String {
	let now = Local::now();
	let ms_past_midnight = now.num_seconds_from_midnight() as u64 * 1000 + now.timestamp_subsec_millis() as u64;
	format!("{}.{ms_past_midnight}", now.format("%Y-%m-%d"))
}

fn archive(source: &Path, dest: &Path) -> io::Result<()> {
	let output = File::create(dest.join(format!("{}.tar.xz", archive_stem())))?;
	let stream = Stream::new_easy_encoder(XZ_PRESET, Check::Crc64).map_err(io::Error::other)?;
	let mut builder = Builder::new(XzEncoder::new_stream(output, stream));
	for name in LOG_FILES {
		match File::open(source.join(name)) {
			Ok(mut file) if file.metadata()?.len() > 0 => builder.append_file(name, &mut file)?,
			Ok(_) => {}
			Err(e) if e.kind() == io::ErrorKind::NotFound => {}
			Err(e) => return Err(e),
		}
	}
	builder.into_inner()?.finish()?;
	Ok(())
}
