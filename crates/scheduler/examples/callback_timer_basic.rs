use std::time::Duration;
use obsidian_scheduler::callback::CallbackTimer;
use obsidian_scheduler::timer_trait::Timer;

#[tokio::main]
async fn main() {
	let timer = CallbackTimer::new(
		async |timer_handle| {
			println!("Timer fired!");
			timer_handle.stop(); // Stop the timer after the first fire
			Ok(())
		},
		Duration::from_secs(5),
	);

	timer.start().await.expect("Failed to start timer");

	println!("This should print immediately, waiting for timer to fire ...");

	while timer.is_running().await {
		tokio::time::sleep(Duration::from_secs(1)).await;
	}
	println!("Timer fired, exiting now.");
}
