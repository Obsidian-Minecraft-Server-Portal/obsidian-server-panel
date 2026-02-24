use obsidian_scheduler::event::EventTimer;
use obsidian_scheduler::timer_trait::Timer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let timer = EventTimer::new("my_timer", tokio::time::Duration::from_secs(5)).await?;

    // Subscribe to receive timer events
    let mut receiver = timer.subscribe();

    // Start the timer
    timer.start().await?;
    println!("Timer started, waiting for events...");

    // Wait for the first event
    match receiver.recv().await {
        Ok(event_name) => {
            println!("Timer event received: {}", event_name);
        }
        Err(e) => {
            eprintln!("Error receiving event: {}", e);
        }
    }

    // Stop the timer after receiving the first event
    timer.stop().await?;
    println!("Timer stopped.");
    
    // Clean up the timer
    // This will prevent any further events from being sent
    timer.drop().await;

    Ok(())
}
