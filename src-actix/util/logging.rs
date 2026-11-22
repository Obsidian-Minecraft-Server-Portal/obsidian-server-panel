use std::fs::{self, File, OpenOptions};
use std::io::{self, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use chrono::{Local, Timelike};
use tar::Builder;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::{fmt, prelude::*, Layer};
use xz2::stream::{Check, Stream};
use xz2::write::XzEncoder;

/// Roll a log file once it grows past this many bytes.
const MAX_BYTES: u64 = 50 * 1024 * 1024;

/// `liblzma` extreme preset flag (`LZMA_PRESET_EXTREME`). OR'd with the
/// numeric preset (9) this yields 7-zip "Ultra" LZMA2 compression.
const LZMA_PRESET_EXTREME: u32 = 1 << 31;

/// Log file names, ordered to match the [`Target`] indices below.
const LOG_FILES: [&str; 4] = ["debug.log", "latest.log", "error.log", "logs.ndjson"];

/// Index into [`RollingLogs`] file/size arrays. Order matches [`LOG_FILES`].
mod target {
	pub const DEBUG: usize = 0;
	pub const LATEST: usize = 1;
	pub const ERROR: usize = 2;
	pub const JSON: usize = 3;
}

/// Shared, mutex-guarded state backing every log layer.
///
/// All four destination files share one lock so that a size-triggered roll
/// archives them together into a single snapshot.
struct RollingLogs {
	dir: PathBuf,
	files: [File; 4],
	sizes: [u64; 4],
}

impl RollingLogs {
	/// Open (truncating) all four log files in `dir`, ready for a fresh launch.
	fn open(dir: PathBuf) -> io::Result<Self> {
		let files = [
			fresh_file(&dir, LOG_FILES[0])?,
			fresh_file(&dir, LOG_FILES[1])?,
			fresh_file(&dir, LOG_FILES[2])?,
			fresh_file(&dir, LOG_FILES[3])?,
		];
		Ok(Self { dir, files, sizes: [0; 4] })
	}

	/// Append `buf` to one destination file, rolling everything if it crosses
	/// the size threshold afterwards.
	fn write_to(&mut self, target: usize, buf: &[u8]) -> io::Result<()> {
		self.files[target].write_all(buf)?;
		self.sizes[target] += buf.len() as u64;
		if self.sizes[target] >= MAX_BYTES {
			self.roll()?;
		}
		Ok(())
	}

	/// Archive the current log files, then truncate them in place so the same
	/// handles keep working (avoids Windows sharing violations from reopening).
	fn roll(&mut self) -> io::Result<()> {
		for file in &mut self.files {
			file.flush()?;
		}
		archive(&self.dir)?;
		for (file, size) in self.files.iter_mut().zip(self.sizes.iter_mut()) {
			file.set_len(0)?;
			file.seek(SeekFrom::Start(0))?;
			*size = 0;
		}
		Ok(())
	}
}

/// A `tracing` writer bound to one destination file in the shared state.
#[derive(Clone)]
struct LogWriter {
	inner: Arc<Mutex<RollingLogs>>,
	target: usize,
}

impl LogWriter {
	fn new(inner: &Arc<Mutex<RollingLogs>>, target: usize) -> Self {
		Self { inner: Arc::clone(inner), target }
	}
}

impl Write for LogWriter {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		let mut logs = self.inner.lock().unwrap_or_else(|e| e.into_inner());
		logs.write_to(self.target, buf)?;
		Ok(buf.len())
	}

	fn flush(&mut self) -> io::Result<()> {
		let mut logs = self.inner.lock().unwrap_or_else(|e| e.into_inner());
		logs.files[self.target].flush()
	}
}

impl<'a> MakeWriter<'a> for LogWriter {
	type Writer = LogWriter;
	fn make_writer(&'a self) -> Self::Writer {
		self.clone()
	}
}

/// Initialise tracing with rolling, multi-level file logs plus console output.
///
/// Writes a `logs/` directory next to the running executable containing:
/// - `debug.log`  — every level (TRACE..=ERROR), human readable
/// - `latest.log` — INFO and above
/// - `error.log`  — ERROR only
/// - `logs.ndjson` — same content as `debug.log`, structured as newline-delimited JSON
///
/// Existing logs are archived on every launch, and any file that grows past
/// 50 MB triggers a roll. A roll snapshots all four files into
/// `logs/{YYYY-MM-DD}.{ms-past-midnight}.tar.xz` using LZMA2 "Ultra"
/// compression, then truncates them.
pub fn setup_logging() -> color_eyre::Result<()> {
	let dir = logs_dir()?;

	// Per-launch roll: archive whatever the previous run left behind.
	if has_existing_logs(&dir) {
		archive(&dir)?;
	}

	let logs = Arc::new(Mutex::new(RollingLogs::open(dir)?));

	let debug_layer = fmt::layer()
		.with_ansi(false)
		.with_thread_names(true)
		.with_writer(LogWriter::new(&logs, target::DEBUG))
		.with_filter(LevelFilter::TRACE);

	let latest_layer = fmt::layer()
		.with_ansi(false)
		.with_thread_names(true)
		.with_writer(LogWriter::new(&logs, target::LATEST))
		.with_filter(LevelFilter::INFO);

	let error_layer = fmt::layer()
		.with_ansi(false)
		.with_thread_names(true)
		.with_writer(LogWriter::new(&logs, target::ERROR))
		.with_filter(LevelFilter::ERROR);

	let json_layer = fmt::layer()
		.json()
		.with_thread_names(true)
		.with_writer(LogWriter::new(&logs, target::JSON))
		.with_filter(LevelFilter::TRACE);

	let console_filter = if crate::DEBUG { LevelFilter::TRACE } else { LevelFilter::INFO };
	let console_layer = fmt::layer()
		.with_thread_names(true)
		.with_writer(io::stdout)
		.with_filter(console_filter);

	tracing_subscriber::registry()
		.with(debug_layer)
		.with(latest_layer)
		.with(error_layer)
		.with(json_layer)
		.with(console_layer)
		.init();

	Ok(())
}

/// `logs/` directory next to the current executable, created if missing.
fn logs_dir() -> io::Result<PathBuf> {
	let exe = std::env::current_exe()?;
	let dir = exe.parent().unwrap_or_else(|| Path::new(".")).join("logs");
	fs::create_dir_all(&dir)?;
	Ok(dir)
}

/// True if any prior log file exists with content worth archiving.
fn has_existing_logs(dir: &Path) -> bool {
	LOG_FILES
		.iter()
		.any(|name| dir.join(name).metadata().map(|m| m.len() > 0).unwrap_or(false))
}

/// Create/truncate a single log file, returning a writable handle at offset 0.
fn fresh_file(dir: &Path, name: &str) -> io::Result<File> {
	OpenOptions::new().create(true).write(true).truncate(true).open(dir.join(name))
}

/// Snapshot the current log files into a timestamped LZMA2 "Ultra" tarball:
/// `{YYYY-MM-DD}.{ms-past-midnight}.tar.xz`.
fn archive(dir: &Path) -> io::Result<()> {
	let now = Local::now();
	let ms_past_midnight =
		now.num_seconds_from_midnight() as u64 * 1000 + now.timestamp_subsec_millis() as u64;
	let file_name = format!("{}.{}.tar.xz", now.format("%Y-%m-%d"), ms_past_midnight);

	let output = File::create(dir.join(&file_name))?;
	let stream = Stream::new_easy_encoder(9 | LZMA_PRESET_EXTREME, Check::Crc64)
		.map_err(io::Error::other)?;
	let mut builder = Builder::new(XzEncoder::new_stream(output, stream));

	for name in LOG_FILES {
		match File::open(dir.join(name)) {
			Ok(mut file) => {
				let len = file.metadata()?.len();
				if len > 0 {
					builder.append_file(name, &mut file)?;
				}
			}
			Err(e) if e.kind() == io::ErrorKind::NotFound => {}
			Err(e) => return Err(e),
		}
	}

	builder.into_inner()?.finish()?;
	Ok(())
}
