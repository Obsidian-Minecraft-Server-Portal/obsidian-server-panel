# Obsidian Scheduler

A flexible, async-first timer library for Rust built on top of Tokio. Obsidian Scheduler provides two different timer paradigms to suit different use cases: **Callback Timers** and **Event Timers**.

## Features

- 🔄 **Two Timer Types**:
  - **CallbackTimer**: Execute async closures at regular intervals
  - **EventTimer**: Broadcast-based timer system with multiple subscribers
- ⚡ **Async/Await**: Built on Tokio for efficient async operations
- 🎯 **Type-Safe**: Leverages Rust's type system for compile-time guarantees
- 🔧 **Flexible**: Feature flags allow you to include only what you need
- 🧵 **Thread-Safe**: Uses Arc and Mutex for safe concurrent access
- 📡 **Broadcast Pattern**: EventTimer supports multiple receivers (pub-sub pattern)
- 🎮 **Timer Control**: Start, stop, reset, and check running status
- 🔗 **Self-Referencing**: Callbacks can control their own timer via TimerHandle

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
obsidian-scheduler = "0.1.0"
```

### Feature Flags

By default, only `callback-timers` is enabled. You can customize which features you need:

```toml
[dependencies]
obsidian-scheduler = { version = "0.1.0", features = ["event-timers"] }
```

Available features:
- `callback-timers` (default): Enable CallbackTimer functionality
- `event-timers`: Enable EventTimer functionality with broadcast support
- `log`: Enable logging support for error messages
- `serde`: Enable serialization support for timer structures
- `examples`: Enable all features needed to run examples

**Note**: At least one of `callback-timers` or `event-timers` must be enabled.

## Usage

### CallbackTimer

The `CallbackTimer` executes an async closure at regular intervals. The callback receives a `TimerHandle` that can be used to control the timer from within the callback itself.

#### Basic Example

```rust
use obsidian_scheduler::callback::CallbackTimer;
use obsidian_scheduler::timer_trait::Timer;
use std::time::Duration;

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

    timer.start();

    println!("Waiting for timer to fire...");

    while timer.is_running().await {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    
    println!("Timer stopped!");
}
```

#### Advanced Example with State

```rust
use obsidian_scheduler::callback::CallbackTimer;
use obsidian_scheduler::timer_trait::Timer;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let counter = Arc::new(Mutex::new(0));
    let counter_clone = Arc::clone(&counter);

    let timer = CallbackTimer::new(
        move |timer_handle| {
            let counter = counter_clone.clone();
            async move {
                let mut count = counter.lock().await;
                *count += 1;
                println!("Timer fired! Count: {}", *count);

                if *count >= 3 {
                    println!("Stopping timer after 3 fires");
                    timer_handle.stop();
                }

                Ok(())
            }
        },
        Duration::from_secs(2),
    );

    timer.start();

    while timer.is_running().await {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
```

### EventTimer

The `EventTimer` uses a broadcast channel pattern, allowing multiple subscribers to receive timer events. This is ideal for pub-sub scenarios where multiple components need to react to the same timer.

#### Basic Example

```rust
use obsidian_scheduler::event::EventTimer;
use obsidian_scheduler::timer_trait::Timer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let timer = EventTimer::new("my_timer", tokio::time::Duration::from_secs(5))?;

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

    // Stop the timer
    timer.stop().await?;
    println!("Timer stopped.");

    Ok(())
}
```

#### Multiple Subscribers Example

```rust
use obsidian_scheduler::event::EventTimer;
use obsidian_scheduler::timer_trait::Timer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let timer = EventTimer::new("my_timer", tokio::time::Duration::from_secs(3))?;

    // Subscribe multiple receivers to the same timer (broadcast pattern)
    let mut receiver1 = timer.subscribe();
    let mut receiver2 = timer.subscribe();
    let mut receiver3 = timer.subscribe();

    timer.start().await?;
    println!("Timer started with 3 subscribers...");

    // Spawn tasks for each receiver
    let task1 = tokio::spawn(async move {
        for i in 1..=3 {
            match receiver1.recv().await {
                Ok(event_name) => {
                    println!("Receiver 1 got event #{}: {}", i, event_name);
                }
                Err(e) => {
                    eprintln!("Receiver 1 error: {}", e);
                    break;
                }
            }
        }
    });

    let task2 = tokio::spawn(async move {
        for i in 1..=3 {
            match receiver2.recv().await {
                Ok(event_name) => {
                    println!("Receiver 2 got event #{}: {}", i, event_name);
                }
                Err(e) => {
                    eprintln!("Receiver 2 error: {}", e);
                    break;
                }
            }
        }
    });

    let task3 = tokio::spawn(async move {
        for i in 1..=3 {
            match receiver3.recv().await {
                Ok(event_name) => {
                    println!("Receiver 3 got event #{}: {}", i, event_name);
                }
                Err(e) => {
                    eprintln!("Receiver 3 error: {}", e);
                    break;
                }
            }
        }
    });

    // Wait for all receivers to complete
    let _ = tokio::join!(task1, task2, task3);

    timer.stop().await?;
    println!("Timer stopped. All receivers completed.");

    // Clean up the timer
    timer.drop().await;

    Ok(())
}
```

#### Cross-Thread Access Example

The `EventTimer` supports a global registry that allows you to retrieve timers by name from different threads or tasks. This is useful for scenarios where you need to access and control a timer from a different part of your application.

```rust
use obsidian_scheduler::event::EventTimer;
use obsidian_scheduler::timer_trait::Timer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create and start a timer in the main thread
    let timer = EventTimer::new("cross_thread_timer", tokio::time::Duration::from_secs(2))?;
    
    // Subscribe in the main thread
    let mut main_receiver = timer.subscribe();
    
    // Start the timer
    timer.start().await?;
    println!("Timer started in main thread");
    
    // Spawn a separate task that retrieves the timer by name
    let thread_task = tokio::spawn(async move {
        // Retrieve the timer by name from a different task/thread
        if let Some(retrieved_timer) = EventTimer::get_timer_by_name("cross_thread_timer").await {
            println!("Successfully retrieved timer from different task");
            
            // Subscribe to the timer from this thread
            let mut thread_receiver = retrieved_timer.subscribe();
            
            // Receive events
            for i in 1..=3 {
                match thread_receiver.recv().await {
                    Ok(event_name) => {
                        println!("Task received event #{}: {}", i, event_name);
                    }
                    Err(e) => {
                        eprintln!("Error receiving event: {}", e);
                        break;
                    }
                }
            }
            
            // Stop the timer from this thread
            retrieved_timer.stop().await?;
            println!("Timer stopped from different task");
        }
        Ok::<(), anyhow::Error>(())
    });
    
    // Main thread also receives events
    for i in 1..=3 {
        match main_receiver.recv().await {
            Ok(event_name) => {
                println!("Main thread received event #{}: {}", i, event_name);
            }
            Err(_) => break,
        }
    }
    
    thread_task.await??;
    
    Ok(())
}
```

This example demonstrates:
- Creating a named timer in one thread
- Retrieving the timer by name from a spawned task
- Multiple threads subscribing to and receiving events from the same timer
- Controlling the timer (starting/stopping) from different threads

## API Overview

### Timer Trait

Both `CallbackTimer` and `EventTimer` implement the `Timer` trait:

```rust
pub trait Timer {
    /// Starts the timer, the timer will run continuously until stopped
    async fn start(&self) -> Result<(), SchedulerError>;
    
    /// Aborts the timer early
    async fn stop(&self) -> Result<(), SchedulerError>;
    
    /// Resets the elapsed time to zero without stopping the timer
    async fn reset(&self) -> Result<(), SchedulerError>;
    
    /// Returns true if the timer is currently running
    async fn is_running(&self) -> bool;
}
```

### CallbackTimer

```rust
impl CallbackTimer {
    /// Creates a new CallbackTimer with the given async callback and interval
    pub fn new<F, Fut>(callback: F, interval: Duration) -> Arc<Self>
    where
        F: Fn(TimerHandle) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static;
}
```

**TimerHandle** methods:
- `stop()`: Stops the timer from within the callback

### EventTimer

```rust
impl EventTimer {
    /// Creates a new EventTimer with the given name and interval
    pub fn new(
        event_name: impl Into<String>,
        interval: tokio::time::Duration,
    ) -> Result<Self, SchedulerError>;
    
    /// Subscribe to receive timer events
    pub fn subscribe(&self) -> broadcast::Receiver<String>;
    
    /// Get a timer by name from the global registry
    pub fn get_timer_by_name(name: impl AsRef<str>) -> Option<EventTimer>;
    
    /// Unregisters the timer from the global registry
    /// This should be called after stopping the timer to prevent memory leaks
    pub async fn drop(&self);
}
```

### Error Handling

The library provides a comprehensive `SchedulerError` enum:

```rust
pub enum SchedulerError {
    TimerAlreadyExists(String),
    TimerNotFound(String),
    TimerNotRunning(String),
    TimerStartError(String),
    TimerStopError(String),
    BroadcastError(RecvError),
    JoinHandleError(String),
}
```

## Examples

The project includes several examples demonstrating different use cases:

### Running Examples

```bash
# Basic callback timer
cargo run --example callback_timer_basic --features examples

# Callback timer with state management
cargo run --example callback_timer --features examples

# Basic event timer
cargo run --example event_timer --features examples

# Event timer with multiple subscribers
cargo run --example event_timer_multiple --features examples

# Event timer with cross-thread access
cargo run --example event_timer_cross_thread --features examples
```

## Requirements

- Rust 2024 edition or later
- Tokio runtime with `time`, `rt`, and `sync` features
- For examples: `tokio/macros` feature

## Dependencies

- `tokio`: Async runtime and utilities
- `anyhow`: Error handling
- `thiserror`: Custom error types
- `log` (optional): Logging support
- `serde` (optional): Serialization support

## Design Decisions

### Why Two Timer Types?

- **CallbackTimer**: Best for self-contained timer logic where the timer needs to execute specific code at intervals. The callback has direct control over the timer and can carry state.

- **EventTimer**: Best for scenarios where multiple components need to react to the same timer. Uses a broadcast pattern allowing any number of subscribers to receive events.

### Why AsyncFnMut?

The library uses Rust's stabilized async closure features instead of `Pin<Box<dyn Future>>` for better ergonomics and performance. This allows natural async closure syntax while maintaining zero-cost abstractions.

### Thread Safety

Both timer types use `Arc<Mutex<T>>` for interior mutability, ensuring thread-safe access across async tasks. The `CallbackTimer` returns `Arc<Self>` from `new()` to make cloning and sharing explicit and efficient.

## License

(Add your license information here)

## Contributing

(Add contribution guidelines here)

## Changelog

### 0.1.0
- Initial release
- CallbackTimer with async closure support
- EventTimer with broadcast pattern
- Timer trait with start/stop/reset/is_running methods
- Feature flags for modular compilation
- Comprehensive error handling
