//! A module providing a `CallbackTimer` implementation to schedule and manage periodic task executions.
use crate::error::SchedulerError;
use crate::timer_trait::Timer;
use anyhow::Result;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

/// A struct representing a handle to control a timer from within its callback.
///
/// The `TimerHandle` provides a mechanism to manage and interact with a timer's async task.
/// It encapsulates the task handle inside an `Arc<Mutex<Option<JoinHandle<()>>>>`.
///
/// This allows for shared ownership of the handle across async tasks, ensuring
/// thread-safe operations on the `JoinHandle`.
///
/// # Derive
/// - `Clone`: The `TimerHandle` can be cloned by leveraging the `Arc` for shared ownership.
///
/// # Fields
/// - `handle`: An `Arc<Mutex<Option<JoinHandle<()>>>>` that represents the handle to the
///   timer's async task. The `Mutex` ensures safe, concurrent access, and the `Option`
///   allows for the possibility of the task handle being initialized or cleared.
///
/// # Notes
///
/// This struct is automatically created and passed to callbacks by `CallbackTimer`.
/// Users should not need to construct this type directly.
///
/// # Examples
///
/// ```rust
/// use obsidian_scheduler::callback::CallbackTimer;
/// use obsidian_scheduler::timer_trait::Timer;
/// use std::time::Duration;
///
/// #[tokio::main]
/// async fn main() {
///     let timer = CallbackTimer::new(
///         |timer_handle| async move {
///             println!("Timer fired!");
///             // Stop the timer from within the callback
///             timer_handle.stop();
///             Ok(())
///         },
///         Duration::from_secs(1)
///     );
///     timer.start().await.expect("Failed to start timer");
/// }
/// ```
#[derive(Clone)]
pub struct TimerHandle {
    handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl TimerHandle {
    /// Stops the timer by aborting its associated asynchronous task.
    ///
    /// This method is typically called from within a callback to stop the timer.
    /// It spawns a new async task to perform the cancellation, so it can be called
    /// from synchronous code.
    ///
    /// # Examples
    /// ```rust
    /// use obsidian_scheduler::callback::CallbackTimer;
    /// use obsidian_scheduler::timer_trait::Timer;
    /// use std::time::Duration;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let timer = CallbackTimer::new(
    ///         |timer_handle| async move {
    ///             println!("Timer fired!");
    ///             // Stop the timer after first execution
    ///             timer_handle.stop();
    ///             Ok(())
    ///         },
    ///         Duration::from_secs(1)
    ///     );
    ///     timer.start().await.expect("Failed to start timer");
    /// }
    /// ```
    ///
    /// # Notes
    /// - This method is thread-safe and can be called from any async context.
    /// - If there is no running task, calling this method has no effect.
    /// - The timer cannot be resumed after being stopped; you must call `start()` again.
    pub fn stop(&self) {
        let handle = Arc::clone(&self.handle);
        tokio::spawn(async move {
            let mut h = handle.lock().await;
            if let Some(task) = h.take() {
                task.abort();
            }
        });
    }
}

type BoxedCallback =
    Box<dyn Fn(TimerHandle) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> + Send + Sync>;

/// A structure representing a timer that periodically executes a callback function in an asynchronous context.
///
/// `CallbackTimer` allows scheduling a callback function to be executed at a fixed interval using Tokio's async runtime.
/// The timer runs in the background and provides mechanisms for managing its execution.
///
/// # Fields
///
/// - `callback`: An `Arc`-wrapped, boxed callback function that will be invoked periodically. The callback must have the type `Box<dyn Fn() + Send + Sync>`.
/// - `interval`: A `tokio::time::Duration` specifying the interval at which the callback is executed.
/// - `handle`: An `Arc`-wrapped `Mutex` containing an optional `JoinHandle<()>` that can be used to track and manage
///   the background task responsible for executing the timer.
///
/// # Example
///
/// ```rust
/// use obsidian_scheduler::callback::CallbackTimer;
/// use obsidian_scheduler::timer_trait::Timer;
/// use std::time::Duration;
///
/// #[tokio::main]
/// async fn main() {
///     let timer = CallbackTimer::new(
///         |_handle| async {
///             println!("Timer triggered!");
///             Ok(())
///         },
///         Duration::from_secs(1)
///     );
///
///     // Start the timer
///     timer.start().await.expect("Failed to start timer");
/// }
/// ```
///
/// # Notes
///
/// The `CallbackTimer` does not automatically start executing the callback.
/// You must call the `start()` method to begin timer execution.
pub struct CallbackTimer {
    callback: Arc<BoxedCallback>,
    interval: tokio::time::Duration,
    handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl CallbackTimer {
    /// Creates a new instance of `CallbackTimer`.
    ///
    /// This function initializes a `CallbackTimer` using the given callback function and interval.
    /// The callback function will be executed repeatedly at the specified interval. The timer is
    /// wrapped in an `Arc` to enable shared ownership.
    ///
    /// # Type Parameters
    /// - `F`: A function type that takes a `TimerHandle` as an argument and returns a future.
    /// - `Fut`: A `Future` type that the callback function returns, which resolves to a `Result<()>`.
    ///
    /// # Parameters
    /// - `callback`: A function that is triggered at every timer tick, receiving a `TimerHandle` to allow external control.
    ///   It must be `Send`, `Sync`, and have a static lifetime. The function should return a future that is executed when
    ///   the timer fires.
    /// - `interval`: A `tokio::time::Duration` specifying how often the callback function will be executed.
    ///
    /// # Returns
    /// Returns an `Arc<Self>` which encapsulates the created `CallbackTimer` instance, allowing for shared ownership and
    /// use across threads.
    ///
    /// # Example
    /// ```rust
    /// use obsidian_scheduler::callback::CallbackTimer;
    /// use obsidian_scheduler::timer_trait::Timer;
    /// use std::time::Duration;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let timer = CallbackTimer::new(
    ///         |handle| async move {
    ///             println!("Timer fired!");
    ///             // Use handle.stop() to stop the timer from within the callback
    ///             Ok(())
    ///         },
    ///         Duration::from_secs(1)
    ///     );
    ///     timer.start().await.expect("Failed to start timer");
    /// }
    /// ```
    ///
    /// # Notes
    /// - The `callback` function must handle its own internal errors, as it is expected to return a `Result<()>`.
    /// - The timer handle (`TimerHandle`) provided to the callback can be used to stop the timer from within the callback.
    ///
    /// # Panics
    /// This function does not explicitly panic, but incorrect usage of the callback or `TimerHandle` may result in runtime errors.
    ///
    /// # Concurrency
    /// The constructed timer can safely be shared across threads due to its use of `Arc`. The `Mutex` inside handles
    /// synchronization for internal state.
    pub fn new<F, Fut>(callback: F, interval: tokio::time::Duration) -> Arc<Self>
    where
        F: Fn(TimerHandle) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let boxed: BoxedCallback = Box::new(move |handle| Box::pin(callback(handle)));

        Arc::new(CallbackTimer {
            callback: Arc::new(boxed),
            interval,
            handle: Arc::new(Mutex::new(None)),
        })
    }
}

impl Timer for CallbackTimer {
    /// Starts the timer and begins executing its provided callback at regular intervals defined by `self.interval`.
    ///
    /// # Return
    /// Returns a `Result<(), SchedulerError>`. If the timer is already running, it returns a `SchedulerError::TimerAlreadyExists` error.
    ///
    /// # Behavior
    /// This function checks whether the timer is already running. If it is, an error is returned.
    /// If not, a new async task is spawned to repeatedly invoke the timer's callback at the specified interval.
    /// Each iteration uses `tokio::time::sleep` to introduce the delay between callback executions.
    ///
    /// A `TimerHandle` is passed to the callback to allow interaction between the callback and the timer.
    ///
    /// If the execution of the callback produces an error, it is logged or printed (depending on the active features),
    /// and the timer will continue with the next callback execution without interruption.
    ///
    /// The timer's running state is stored in `self.handle`, allowing the timer to later be stopped if required.
    ///
    /// # Errors
    /// - Returns a `SchedulerError::TimerAlreadyExists` if the timer is already running.
    ///
    /// # Logging
    /// - If the `log` feature is enabled, any errors that occur during callback execution are logged using the `log` crate.
    /// - If the `log` feature is not enabled, errors are printed to standard error using `eprintln!`.
    ///
    /// # Examples
    /// ```rust
    /// use obsidian_scheduler::callback::CallbackTimer;
    /// use obsidian_scheduler::timer_trait::Timer;
    /// use std::time::Duration;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let timer = CallbackTimer::new(
    ///         |_handle| async {
    ///             println!("Executing callback...");
    ///             Ok(())
    ///         },
    ///         Duration::from_secs(5)
    ///     );
    ///     timer.start().await.unwrap();
    /// }
    /// ```
    ///
    /// # Notes
    /// - This function uses `tokio::spawn` to create asynchronous tasks, so it requires a Tokio runtime.
    ///
    /// # Features
    /// - `log`: Enables logging of errors during callback execution using the `log` crate.
    async fn start(&self) -> Result<(), SchedulerError> {
        // Acquire lock first to make check-and-start atomic (prevents TOCTOU race condition)
        let mut handle_guard = self.handle.lock().await;
        
        if handle_guard.is_some() {
            return Err(SchedulerError::TimerAlreadyExists(
                "CallbackTimer".to_string(),
            ));
        }

        let callback = Arc::clone(&self.callback);
        let interval = self.interval;

        // Create a TimerHandle that can be passed to the callback
        let timer_handle = TimerHandle {
            handle: Arc::clone(&self.handle),
        };

        let task = tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                let future = callback(timer_handle.clone());
                if let Err(e) = future.await {
                    #[cfg(feature = "log")]
                    log::error!("Callback execution failed: {:?}", e);
                    #[cfg(not(feature = "log"))]
                    eprintln!("Callback execution failed: {}", e);
                }
            }
        });

        // Store the handle atomically while holding the lock
        *handle_guard = Some(task);
        drop(handle_guard);

        Ok(())
    }

    ///
    /// Stops the currently running timer if it is active.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the timer was successfully stopped.
    /// * `Err(SchedulerError::TimerNotRunning)` - If the timer is not running.
    ///
    /// # Errors
    ///
    /// This function returns a `SchedulerError::TimerNotRunning` error if
    /// the timer is not currently running.
    ///
    /// # Behavior
    ///
    /// If the timer is running, its associated handle is locked and the
    /// task is aborted asynchronously. This ensures the currently scheduled
    /// task is stopped.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let result = callback_timer.stop().await;
    /// match result {
    ///     Ok(_) => println!("Timer stopped successfully."),
    ///     Err(e) => println!("Failed to stop timer: {:?}", e),
    /// }
    /// ```
    ///
    /// # Notes
    ///
    /// This method spawns an asynchronous task for aborting the currently
    /// running task. The actual stopping process may be completed in
    /// the background depending on the runtime's task scheduling.
    ///
    /// This method requires the caller to ensure that `self` is a valid
    /// reference to the timer instance and is properly synchronized if
    /// accessed across multiple threads.
    ///
    async fn stop(&self) -> Result<(), SchedulerError> {
        if !self.is_running().await {
            return Err(SchedulerError::TimerNotRunning("CallbackTimer".to_string()));
        }

        let mut h = self.handle.lock().await;
        if let Some(task) = h.take() {
            task.abort();
        }

        Ok(())
    }

    /// Asynchronously resets the scheduler by stopping it and then starting it again.
    ///
    /// This method ensures that the scheduler is re-initialized to a clean state.
    /// It first stops the scheduler by invoking the `stop` method and, once it completes successfully,
    /// it proceeds to start the scheduler again using the `start` method. Any errors encountered
    /// during these operations are propagated back to the caller.
    ///
    /// # Returns
    /// * `Ok(())` - If the scheduler is successfully reset.
    /// * `Err(SchedulerError)` - If an error occurs during either the stop or start operation.
    ///
    /// # Example
    /// ```rust
    /// # use your_crate::{Scheduler, SchedulerError};
    /// # async fn example() -> Result<(), SchedulerError> {
    /// let scheduler = Scheduler::new();
    /// scheduler.reset().await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn reset(&self) -> Result<(), SchedulerError> {
        self.stop().await?;
        self.start().await?;
        Ok(())
    }

    /// Checks if the associated task or process is currently running.
    ///
    /// This asynchronous function determines whether the internal handle associated
    /// with the task or process is in use (i.e., not `None`). It achieves this by
    /// acquiring an asynchronous lock on the handle and checking its state.
    ///
    /// # Returns
    ///
    /// - `true`: If the handle is set (indicating that the task/process is running).
    /// - `false`: If the handle is not set (indicating that the task/process is not running).
    ///
    /// # Example
    ///
    /// ```rust
    /// let is_running = my_instance.is_running().await;
    /// if is_running {
    ///     println!("The task is currently running!");
    /// } else {
    ///     println!("The task is not running.");
    /// }
    /// ```
    ///
    /// # Notes
    ///
    /// - This function is asynchronous and should be `.await`ed to retrieve the result.
    /// - It uses an `Arc<Mutex<Option<T>>>` pattern to represent the shared state of the handle,
    ///   ensuring thread-safe access.
    ///
    /// # Panics
    ///
    /// This function will panic if the lock on the handle (`Mutex`) is poisoned.
    ///
    /// # Requirements
    ///
    /// Ensure that the type of `self.handle` is `Arc<tokio::sync::Mutex<Option<T>>>` where `T`
    /// represents the type of the handle/resource being checked.
    ///
    /// # Concurrency
    ///
    /// This function acquires a lock on `self.handle`. Ensure careful usage in concurrent
    /// contexts to avoid deadlocks or performance bottlenecks when accessing the same resource.
    async fn is_running(&self) -> bool {
        let handle = Arc::clone(&self.handle);
        let handle_guard = handle.lock().await;
        handle_guard.is_some()
    }
}
