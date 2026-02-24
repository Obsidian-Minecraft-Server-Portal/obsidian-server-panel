use crate::error::SchedulerError;
use crate::timer_trait::Timer;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use tokio::sync::{Mutex, broadcast};
use tokio::task::JoinHandle;

static ACTIVE_TIMERS: OnceLock<Mutex<HashMap<String, Arc<Mutex<EventTimer>>>>> = OnceLock::new();

#[derive(Clone, Debug)]
pub struct EventTimer {
    pub event_name: String,
    pub interval: tokio::time::Duration,
    sender: broadcast::Sender<String>,
    handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl EventTimer {
    /// Creates a new instance of `EventTimer` and initializes it with the given event name
    /// and time interval.
    ///
    /// # Arguments
    ///
    /// * `event_name` - A value that can be converted into a `String`, representing
    ///   the name of the event associated with the timer.
    /// * `interval` - A `tokio::time::Duration` object that specifies the
    ///   time interval for the timer.
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` - Returns the newly created `EventTimer` instance if successfully initialized.
    /// * `Err(SchedulerError)` - Returns a `SchedulerError` if the timer registration fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// let timer = EventTimer::new("my_event", tokio::time::Duration::from_secs(5));
    /// match timer {
    ///     Ok(timer) => println!("Timer initialized successfully!"),
    ///     Err(err) => println!("Failed to initialize timer: {:?}", err),
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error of type `SchedulerError` if the call
    /// to `register_timer` fails.
    ///
    /// # Implementation Details
    ///
    /// - Internally, this function creates a broadcast channel with a capacity of 100
    ///   to manage message sending.
    /// - The timer's `.register_timer()` function is invoked to configure the timer,
    ///   and if it fails, the error is propagated to the caller.
    /// - The returned `EventTimer` is wrapped in an `Arc` and guarded by a `Mutex` for
    ///   safe concurrent access.
    pub async fn new(
        event_name: impl Into<String>,
        interval: tokio::time::Duration,
    ) -> Result<Self, SchedulerError> {
        let (sender, _) = broadcast::channel(100);
        let timer = EventTimer {
            event_name: event_name.into(),
            interval,
            sender,
            handle: Arc::new(Mutex::new(None)),
        };
        timer.register_timer().await?;

        Ok(timer)
    }

    /// Subscribes to the broadcast channel and returns a receiver for the channel.
    ///
    /// This method creates a new subscriber to the underlying `broadcast` channel,
    /// allowing the caller to receive messages sent through the channel. Each subscriber
    /// gets its own `broadcast::Receiver`, ensuring that every message broadcasted is delivered
    /// to all active subscribers.
    ///
    /// # Returns
    ///
    /// A `broadcast::Receiver<String>` that can be used to receive messages. The receiver
    /// will buffer messages and allow asynchronous access to each broadcasted message.
    ///
    /// # Example
    ///
    /// ```rust
    /// let subscription = my_instance.subscribe();
    /// tokio::spawn(async move {
    ///     while let Ok(message) = subscription.recv().await {
    ///         println!("Received: {}", message);
    ///     }
    /// });
    /// ```
    ///
    /// # Panics
    ///
    /// This function does not panic unless there is an internal failure in the underlying
    /// `broadcast` mechanism, which is highly unlikely during normal operation.
    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.sender.subscribe()
    }

    /// Registers a timer event by associating it with the scheduler.
    ///
    /// This function checks if a timer for the given event name already exists. If it does,
    /// an error is returned. Otherwise, the timer is added to the active timers collection.
    ///
    /// # Errors
    /// Returns a `SchedulerError::TimerAlreadyExists` error if a timer with the
    /// specified event name has already been registered.
    ///
    /// # Returns
    /// - `Ok(())` if the timer was successfully registered.
    /// - `Err(SchedulerError)` if registration fails due to a conflict.
    ///
    /// # Example
    /// ```rust
    /// let scheduler = TimerScheduler { event_name: "example_event".to_string() };
    /// match scheduler.register_timer() {
    ///     Ok(()) => println!("Timer registered successfully!"),
    ///     Err(e) => eprintln!("Failed to register timer: {:?}", e),
    /// }
    /// ```
    ///
    /// # Panics
    /// This function may panic if the `timers_mutex` lock is poisoned or another thread
    /// has caused an unrecoverable error while accessing the `ACTIVE_TIMERS` collection.
    ///
    /// # Thread Safety
    /// This function uses a `Mutex` to ensure safe access to the `ACTIVE_TIMERS`
    /// collection across threads.
    async fn register_timer(&self) -> Result<(), SchedulerError> {
        let timers_mutex = ACTIVE_TIMERS.get_or_init(|| Mutex::new(HashMap::new()));
        let mut timers = timers_mutex.lock().await;
        if timers.contains_key(&self.event_name) {
            return Err(SchedulerError::TimerAlreadyExists(self.event_name.clone()));
        }
        timers.insert(self.event_name.clone(), Arc::new(Mutex::new(self.clone())));

        Ok(())
    }
    
    /// Unregisters a timer by removing it from the active timers collection.
    ///
    /// This function removes the timer from the global `ACTIVE_TIMERS` registry,
    /// preventing memory leaks from accumulating stopped timers.
    ///
    /// # Thread Safety
    /// This function uses a `Mutex` to ensure safe access to the `ACTIVE_TIMERS`
    /// collection across threads.
    pub async fn drop(&self) {
        let timers_mutex = ACTIVE_TIMERS.get_or_init(|| Mutex::new(HashMap::new()));
        let mut timers = timers_mutex.lock().await;
        timers.remove(&self.event_name);
    }
    
    /// Retrieves a timer by its name from an active timers collection.
    ///
    /// This function asynchronously fetches an `EventTimer` object associated
    /// with a given name. It searches through a global collection of active timers
    /// stored in a `HashMap` that is protected by a `StdMutex`. The lock is briefly
    /// held to check for the existence of the timer, and then released.
    /// If the timer exists, the function awaits the lock on the internal `Arc<Mutex<EventTimer>>`
    /// and returns a cloned `EventTimer`. If no timer with the specified name exists,
    /// the function returns `None`.
    ///
    /// # Parameters
    /// - `name`: A string-like value (`impl AsRef<str>`) representing the name of the timer to fetch.
    ///
    /// # Returns
    /// - `Some(EventTimer)`: If a timer with the given name exists.
    /// - `None`: If no timer with the specified name exists.
    ///
    /// # Example
    /// ```rust
    /// use obsidian_scheduler::event::EventTimer;
    /// use obsidian_scheduler::timer_trait::Timer;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     if let Some(timer) = EventTimer::get_timer_by_name("my_timer").await {
    ///         println!("Timer found: {:?}", timer);
    ///         // You can now use the timer
    ///         timer.start().await.expect("Failed to start timer");
    ///     } else {
    ///         println!("No timer found with the given name.");
    ///     }
    /// }
    /// ```
    ///
    /// # Notes
    /// - The `ACTIVE_TIMERS` is a lazy static global variable of type `Mutex<HashMap<String, Arc<Mutex<EventTimer>>>>`.
    /// - The function uses an asynchronous lock (`await`) on the internal `Arc<Mutex<EventTimer>>`, so it must be called
    ///   in an async context.
    /// - Ensure thread-safety when managing global shared data like `ACTIVE_TIMERS`.
    pub async fn get_timer_by_name(name: impl AsRef<str>) -> Option<EventTimer> {
        let timers_mutex = ACTIVE_TIMERS.get_or_init(|| Mutex::new(HashMap::new()));
        let timer_arc = {
            let timers = timers_mutex.lock().await;
            timers.get(name.as_ref()).cloned()
        }; // Mutex lock is released here
        
        if let Some(arc) = timer_arc {
            let timer = arc.lock().await;
            Some(timer.clone())
        } else {
            None
        }
    }
}

impl Timer for EventTimer {
    /// Starts the timer if it is not already running. This method creates an asynchronous task
    /// that periodically sends an event to all subscribers at the specified interval. It uses a shared lock to
    /// ensure that the task can be tracked and stopped later.
    ///
    /// # Returns
    /// - `Ok(())`: If the timer was successfully started.
    /// - `Err(SchedulerError::TimerAlreadyExists)`: If the timer is already running for the specified event.
    ///
    /// # Errors
    /// Returns a `SchedulerError::TimerAlreadyExists` if the timer is already running.
    async fn start(&self) -> Result<(), SchedulerError> {
        // Acquire lock first to make check-and-start atomic (prevents TOCTOU race condition)
        let mut handle_guard = self.handle.lock().await;
        
        if handle_guard.is_some() {
            return Err(SchedulerError::TimerAlreadyExists(self.event_name.clone()));
        }
        
        let sender = self.sender.clone();
        let interval = self.interval;
        let event_name = self.event_name.clone();

        let task = tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                // Send the event to all subscribers
                // Ignore send errors (happens when no receivers are listening)
                let _ = sender.send(event_name.clone());
            }
        });

        // Store the handle atomically while holding the lock
        *handle_guard = Some(task);
        drop(handle_guard);
        Ok(())
    }

    /// Stops the running timer asynchronously if it is currently active.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - if the timer was successfully stopped.
    /// * `Err(SchedulerError::TimerNotRunning)` - if the timer is not currently running.
    ///
    /// # Errors
    ///
    /// Returns a `SchedulerError::TimerNotRunning` if the timer for the associated event
    /// (indicated by `event_name`) is not currently running.
    async fn stop(&self) -> Result<(), SchedulerError> {
        
        if !self.is_running().await {
            return Err(SchedulerError::TimerNotRunning(self.event_name.clone()));
        }
        
        let mut h = self.handle.lock().await;
        if let Some(task) = h.take() {
            task.abort();
        }
        drop(h); // Release the lock before calling unregister_timer
        
        Ok(())
    }

    /// Resets the timer by stopping and restarting it.
    ///
    /// This asynchronous function performs two main actions:
    /// 1. Calls the `stop` method to halt the current execution of the timer.
    /// 2. Calls the `start` method to restart the timer.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the timer successfully stops and starts without errors.
    /// * `Err(SchedulerError)` - If an error occurs during the `stop` or `start` operations.
    ///
    /// # Errors
    /// This function propagates any errors encountered from the `stop` or `start` methods.
    async fn reset(&self) -> Result<(), SchedulerError> {
        self.stop().await?;
        self.start().await?;
        Ok(())
    }

    /// Checks if the current instance is running by inspecting its associated handle.
    ///
    /// # Returns
    ///
    /// Returns `true` if the `handle` is not `None`,
    /// indicating that the instance is currently running.
    /// Otherwise, returns `false` if the `handle` is `None`.
    ///
    /// # Async Behavior
    ///
    /// This is an asynchronous function as it acquires a lock on the `handle`
    /// using an `async` lock mechanism.
    /// Ensure that this function is called within an asynchronous runtime.
    ///
    /// # Example
    async fn is_running(&self) -> bool {
        let handle = Arc::clone(&self.handle);
        let handle_guard = handle.lock().await;
        handle_guard.is_some()
    }
}
