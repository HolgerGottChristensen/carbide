use std::ops::Deref;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, RecvTimeoutError, Sender};
use crate::time::*;
use parking_lot::RwLock;
use crate::asynchronous::spawn;
use crate::asynchronous::EVENT_SINK;
use crate::event::CoreEvent;

#[derive(Clone)]
pub struct Timer<T> where T: Fn() + Clone + Send + 'static {
    interval: Arc<RwLock<Duration>>,
    repeat: Arc<AtomicBool>,
    triggered: T,

    // Running is determined whether the option is some or none
    channel: Arc<RwLock<Option<Sender<Duration>>>>,
}

impl<T: Fn() + Clone + Send + 'static> Timer<T> {

    /// Creates a new timer with the trigger
    ///
    /// The default timer will not repeat, is not running and has an interval of 1 sec.
    pub fn new(trigger: T) -> Timer<T> {
        let timer = Timer {
            interval: Arc::new(RwLock::new(Duration::new(1, 0))),
            repeat: Arc::new(AtomicBool::new(false)),
            triggered: trigger,
            channel: Arc::new(RwLock::new(None)),
        };

        timer
    }

    /// Set a custom interval of the current timer.
    /// Setting the interval of a running timer will set the current time to the new interval
    /// immediately and progress of the timer is reset.
    pub fn interval(self, interval: Duration) -> Self {
        *self.interval.write() = interval;
        if let Some(channel) = self.channel.read().deref() {
            let _ = channel.send(*self.interval.read());
        }
        self
    }

    /// Set the timer to repeat.
    pub fn repeat(self) -> Self {
        self.repeat.store(true, Ordering::Relaxed);
        self
    }

    /// Start the timer. If the timer is already running this will have no effect.
    pub fn start(self) -> Self {
        if self.channel.read().is_some() {
            return self;
        }

        let (sender, receiver) = channel::<Duration>();

        self.channel.write().replace(sender);

        let trigger = self.triggered.clone();
        let repeat = self.repeat.clone();
        let duration = *self.interval.read();

        let event_sink = EVENT_SINK.with(|e| e.borrow().clone());

        spawn(async move {
            let mut current_duration = duration;

            loop {
                match receiver.recv_timeout(current_duration) {
                    Ok(new_duration) => {
                        // We reset the timer and start again
                        current_duration = new_duration;
                    }
                    Err(RecvTimeoutError::Timeout) => {
                        // Trigger the function then start again with the same duration
                        trigger();
                        event_sink.send(CoreEvent::Async);
                    }
                    Err(RecvTimeoutError::Disconnected) => {
                        // The timer has been stopped, so we shut it down
                        break;
                    }
                }

                if !repeat.load(Ordering::Relaxed) {
                    break;
                }
            }
        });

        self
    }

    /// Stop the timer. If the timer is already stopped, this will have no effect.
    pub fn stop(self) -> Self {
        if self.channel.read().is_some() {
            self.channel.write().take();
        }

        self
    }

    /// Restart the timer. If the timer is already running, we reset the time to the interval.
    /// If the timer is stopped we start the timer.
    pub fn restart(self) -> Self  {
        if self.channel.read().is_none() {
            self.start()
        } else {
            if let Some(channel) = self.channel.read().deref() {
                let _ = channel.send(*self.interval.read());
            }
            self
        }
    }

    /// Returns true if the current timer is running
    pub fn is_running(&self) -> bool {
        self.channel.read().is_some()
    }

    /// Will stop the timer if it is running and start the timer if it is not running.
    pub fn toggle(self) -> Self {
        if self.is_running() {
            self.stop()
        } else {
            self.start()
        }
    }
}