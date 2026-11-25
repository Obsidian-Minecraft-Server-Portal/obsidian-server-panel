//! Map work runs at the lowest priority the platform offers, so a render never starves the Minecraft server beside it. Each
//! render and each detail draw gets a thread of its own, and section writes fan out over a small pool whose threads start
//! lowered too.

use std::sync::OnceLock;

use obsidian_common::{CommonError, Result};
use thread_priority::{ThreadPriority, set_current_thread_priority};

static POOL: OnceLock<rayon::ThreadPool> = OnceLock::new();

pub(crate) fn pool() -> &'static rayon::ThreadPool {
	POOL.get_or_init(|| {
		let cores = std::thread::available_parallelism().map_or(1, |cores| cores.get());
		rayon::ThreadPoolBuilder::new()
			.num_threads(cores.saturating_sub(2).clamp(1, 8))
			.thread_name(|index| format!("worldmap-{index}"))
			.start_handler(|_| lower_current_thread())
			.build()
			.expect("the world map thread pool")
	})
}

/// Runs `work` on a thread of its own and waits for the result without holding an async worker.
pub(crate) async fn spawn<T: Send + 'static>(work: impl FnOnce() -> Result<T> + Send + 'static) -> Result<T> {
	let (sender, receiver) = tokio::sync::oneshot::channel();
	std::thread::Builder::new()
		.name("worldmap".into())
		.spawn(move || {
			lower_current_thread();
			let _ = sender.send(work());
		})
		.map_err(|error| CommonError::Other(error.to_string()))?;
	receiver
		.await
		.map_err(|_| CommonError::Other("the world map thread stopped before it finished".into()))?
}

/// Lowers the calling thread as far as the platform allows. A step that fails leaves the thread at the level before it.
fn lower_current_thread() {
	let _ = set_current_thread_priority(ThreadPriority::Min);
	lower_os_thread();
}

#[cfg(windows)]
fn lower_os_thread() {
	use windows_sys::Win32::System::Threading::{GetCurrentThread, SetThreadPriority, THREAD_MODE_BACKGROUND_BEGIN};
	// SAFETY: GetCurrentThread returns a pseudo-handle to the calling thread, which needs no closing.
	unsafe {
		SetThreadPriority(GetCurrentThread(), THREAD_MODE_BACKGROUND_BEGIN);
	}
}

#[cfg(target_os = "macos")]
fn lower_os_thread() {
	// SAFETY: the call only changes the quality-of-service class of the calling thread.
	unsafe {
		libc::pthread_set_qos_class_self_np(libc::qos_class_t::QOS_CLASS_BACKGROUND, 0);
	}
}

#[cfg(target_os = "linux")]
fn lower_os_thread() {
	const IOPRIO_WHO_PROCESS: libc::c_int = 1;
	const IOPRIO_CLASS_IDLE: libc::c_int = 3 << 13;
	// SAFETY: ioprio_set changes only the I/O class of the thread named by its id, and on Linux that id is a thread id.
	unsafe {
		let tid = libc::syscall(libc::SYS_gettid) as libc::c_int;
		libc::syscall(libc::SYS_ioprio_set, IOPRIO_WHO_PROCESS, tid, IOPRIO_CLASS_IDLE);
	}
}

#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
fn lower_os_thread() {}

#[cfg(all(test, target_os = "linux"))]
mod tests {
	use super::*;

	fn thread_id() -> libc::c_int {
		// SAFETY: gettid takes no arguments and cannot fail.
		unsafe { libc::syscall(libc::SYS_gettid) as libc::c_int }
	}

	fn nice() -> libc::c_int {
		// SAFETY: getpriority only reads the nice value of the named thread.
		unsafe { libc::getpriority(libc::PRIO_PROCESS, thread_id() as libc::id_t) }
	}

	fn io_class() -> libc::c_long {
		const IOPRIO_WHO_PROCESS: libc::c_int = 1;
		// SAFETY: ioprio_get only reads the I/O priority of the named thread.
		let ioprio = unsafe { libc::syscall(libc::SYS_ioprio_get, IOPRIO_WHO_PROCESS, thread_id()) };
		ioprio >> 13
	}

	#[test]
	fn pool_threads_run_at_the_lowest_nice_value_with_idle_io() {
		assert_eq!(pool().install(|| (nice(), io_class())), (19, 3));
	}

	#[tokio::test]
	async fn spawned_threads_run_at_the_lowest_nice_value_with_idle_io() {
		assert_eq!(spawn(|| Ok((nice(), io_class()))).await.unwrap(), (19, 3));
	}
}
