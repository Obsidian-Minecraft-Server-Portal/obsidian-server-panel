use obsidian_scheduler::event::EventTimer;
use obsidian_scheduler::timer_trait::Timer;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create and start a timer in the main thread
    let timer = EventTimer::new("cross_thread_timer", tokio::time::Duration::from_secs(2)).await?;
    
    // Subscribe in the main thread
    let mut main_receiver = timer.subscribe();
    
    // Start the timer
    timer.start().await?;
    println!("Timer 'cross_thread_timer' started in main thread");
    
    // Spawn a separate tokio task (simulating a different thread context)
    let thread_task = tokio::spawn(async move {
        println!("\n[Thread Task] Attempting to retrieve timer by name...");
        
        // Retrieve the timer by name from a different task/thread
        if let Some(retrieved_timer) = EventTimer::get_timer_by_name("cross_thread_timer").await {
            println!("[Thread Task] Successfully retrieved timer: {}", retrieved_timer.event_name);
            
            // Subscribe to the timer from this thread
            let mut thread_receiver = retrieved_timer.subscribe();
            
            println!("[Thread Task] Subscribed to timer, waiting for 3 events...\n");
            
            // Receive 3 events
            for i in 1..=3 {
                match thread_receiver.recv().await {
                    Ok(event_name) => {
                        println!("[Thread Task] Event #{}: Received '{}'", i, event_name);
                    }
                    Err(e) => {
                        eprintln!("[Thread Task] Error receiving event: {}", e);
                        break;
                    }
                }
            }
            
            println!("\n[Thread Task] Stopping timer from this thread...");
            if let Err(e) = retrieved_timer.stop().await {
                eprintln!("[Thread Task] Error stopping timer: {}", e);
            } else {
                println!("[Thread Task] Timer stopped successfully!");
            }
        } else {
            eprintln!("[Thread Task] Failed to retrieve timer by name!");
        }
    });
    
    // Main thread also receives events
    println!("\n[Main Thread] Waiting for events...\n");
    for i in 1..=3 {
        match tokio::time::timeout(Duration::from_secs(10), main_receiver.recv()).await {
            Ok(Ok(event_name)) => {
                println!("[Main Thread] Event #{}: Received '{}'", i, event_name);
            }
            Ok(Err(e)) => {
                println!("[Main Thread] Receiver closed: {}", e);
                break;
            }
            Err(_) => {
                println!("[Main Thread] Timeout waiting for event");
                break;
            }
        }
    }
    
    // Wait for the thread task to complete
    thread_task.await?;
    
    println!("\n=== Example completed successfully! ===");
    println!("This example demonstrated:");
    println!("  1. Creating a timer in the main thread");
    println!("  2. Retrieving the same timer by name from a different task");
    println!("  3. Subscribing to events from multiple threads");
    println!("  4. Controlling the timer (stopping) from a different thread");
    
    // Clean up the timer
    timer.drop().await;
    
    Ok(())
}
