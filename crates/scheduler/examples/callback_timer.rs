use obsidian_scheduler::callback::CallbackTimer;
use obsidian_scheduler::timer_trait::Timer;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let exit_flag_counter = Arc::new(Mutex::new(0));
    let exit_flag_counter_clone = Arc::clone(&exit_flag_counter);
    println!("Starting callback timer ...");

    let counter_for_callback = Arc::clone(&exit_flag_counter_clone);
    let timer = CallbackTimer::new(
        move |timer_handle| {
            let counter = counter_for_callback.clone();
            async move {
                println!(
                    "Timer fired! Incrementing exit flag... {}",
                    *counter.lock().await
                );
                let mut flag = counter.lock().await;
                *flag += 1;

                if *flag >= 3 {
                    println!("Exit flag reached 3, stopping timer.");
                    timer_handle.stop();
                }

                Ok(())
            }
        },
        Duration::from_secs(5),
    );

    timer.start();

    println!("This should print immediately, waiting for timer to fire ...");

    while timer.is_running().await {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let counter = exit_flag_counter.lock().await;
        println!("Counter is at: {}", *counter);
        drop(counter); // Release the lock before sleeping
    }
    println!("Timer fired, exiting now.");
}
