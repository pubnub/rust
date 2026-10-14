//! # Data stream module
//!
//! This module contains the [`DataStream`] struct.

use futures::Stream;
use spin::RwLock;

use crate::lib::{
    alloc::{collections::VecDeque, sync::Arc},
    core::{
        ops::{Deref, DerefMut},
        pin::Pin,
        task::{Context, Poll, Waker},
    },
};

/// A generic data stream.
///
/// [`DataStream`] provides functionality which allows to `poll` any new data
/// which has been pushed into data queue.
#[derive(Debug, Default)]
pub struct DataStream<D> {
    inner: Arc<DataStreamRef<D>>,
}

impl<D> Deref for DataStream<D> {
    type Target = DataStreamRef<D>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<D> DerefMut for DataStream<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::get_mut(&mut self.inner)
            .expect("Multiple mutable references to the DataStream are not allowed")
    }
}

impl<D> Clone for DataStream<D> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

/// A generic data stream reference.
///
/// This struct contains the actual data stream state.
/// It is wrapped in an `Arc` by [`DataStream`] and uses interior mutability for
/// its internal state.
///
/// Not intended to be used directly. Use [`DataStream`] instead.
#[derive(Debug, Default)]
pub struct DataStreamRef<D> {
    /// Queue with data for stream listener.
    queue: RwLock<VecDeque<D>>,

    /// Data stream waker.
    ///
    /// Handler used each time when new data available for a stream listener.
    waker: RwLock<Option<Waker>>,

    /// Whether data stream still valid or not.
    is_valid: RwLock<bool>,
}

impl<D> DataStream<D> {
    /// Creates a new `DataStream` with a default queue size of 100.
    ///
    /// # Example
    ///
    /// ```
    /// use pubnub::core::DataStream;
    ///
    /// let stream: DataStream<i32> = DataStream::new();
    /// ```
    pub fn new() -> DataStream<D> {
        Self::with_queue_size(100)
    }

    /// Creates a new `DataStream` with a specified queue size.
    ///
    /// The `with_queue_size` function creates a new `DataStream` with an empty
    /// queue and the specified `size`. The `size` parameter determines the
    /// maximum number of elements that can be stored in the queue before
    /// old elements are dropped to make room for new ones.
    ///
    /// # Arguments
    ///
    /// * `size` - The maximum number of elements that can be stored in the
    ///   queue.
    ///
    /// # Returns
    ///
    /// A new `DataStream` with the specified queue size.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::collections::VecDeque;
    /// use pubnub::core::DataStream;
    ///
    /// let data_stream = DataStream::<usize>::with_queue_size(10);
    /// ```
    pub fn with_queue_size(size: usize) -> DataStream<D> {
        Self::with_queue_data(VecDeque::new(), size)
    }

    /// Creates a new `DataStream` with a given queue `data` and `size`.
    /// The `data` is put into a `VecDeque` with capacity `size`.
    ///
    /// # Arguments
    ///
    /// * `data` - A `VecDeque` of type `D` that contains the initial data to be
    ///   put into the queue.
    /// * `size` - The maximum capacity of the queue.
    ///
    /// # Return value
    ///
    /// Returns a new `DataStream` containing the queue data.
    ///
    /// # Examples
    /// ```
    /// use std::collections::VecDeque;
    /// use pubnub::core::DataStream;
    ///
    /// let data: VecDeque<i32> = VecDeque::from(vec![1, 2, 3]);
    /// let stream: DataStream<i32> = DataStream::with_queue_data(data, 5);
    /// ```
    pub fn with_queue_data(data: VecDeque<D>, size: usize) -> DataStream<D> {
        let mut queue_data = VecDeque::with_capacity(size);

        if !data.is_empty() {
            queue_data.extend(data.into_iter().take(queue_data.capacity()));
        }

        Self {
            inner: Arc::new(DataStreamRef {
                queue: RwLock::new(queue_data),
                waker: RwLock::new(None),
                is_valid: RwLock::new(true),
            }),
        }
    }

    #[cfg(all(feature = "subscribe", feature = "std"))]
    pub(crate) fn push_data(&self, data: D) {
        if !*self.is_valid.read() {
            return;
        }

        {
            let mut queue_data_slot = self.queue.write();

            // Dropping the earliest entry to prevent the queue from growing too large.
            if queue_data_slot.len() == queue_data_slot.capacity() {
                queue_data_slot.pop_front();
            }

            queue_data_slot.push_back(data);
        }

        // Wake the listener only after the queue lock has been released.
        // Waking while holding it can deadlock with a concurrent `poll_next`.
        self.wake_stream();
    }

    #[cfg(all(feature = "subscribe", feature = "std"))]
    pub(crate) fn invalidate(&self) {
        *self.is_valid.write() = false;
        self.wake_stream();
    }

    /// Wake up the stream listener (if any).
    ///
    /// The waker is taken out of its slot first, so no stream lock is held
    /// while `wake()` runs.
    #[cfg(all(feature = "subscribe", feature = "std"))]
    fn wake_stream(&self) {
        let waker = self.waker.write().take();
        if let Some(waker) = waker {
            waker.wake();
        }
    }
}

impl<D> Stream for DataStream<D> {
    type Item = D;

    fn poll_next(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if !*self.is_valid.read() {
            return Poll::Ready(None);
        }

        // Register the waker before checking the stream state, so data pushed
        // (or invalidation) after this point wakes the listener. Locks are
        // taken one at a time to avoid lock-order deadlocks with producers.
        *self.waker.write() = Some(ctx.waker().clone());

        if !*self.is_valid.read() {
            return Poll::Ready(None);
        }

        let data = self.queue.write().pop_front();
        if let Some(data) = data {
            Poll::Ready(Some(data))
        } else {
            Poll::Pending
        }
    }
}

#[cfg(all(test, feature = "subscribe", feature = "std"))]
mod should {
    use super::*;
    use std::{
        sync::{
            atomic::{AtomicBool, Ordering},
            mpsc,
        },
        task::Wake,
        thread,
        time::{Duration, Instant},
    };

    /// How long lock-order tests wait for the expected lock state.
    const LOCK_WAIT: Duration = Duration::from_secs(2);

    /// Waker which records whether it has been called.
    #[derive(Default)]
    struct FlagWaker {
        woken: AtomicBool,
    }

    impl Wake for FlagWaker {
        fn wake(self: Arc<Self>) {
            self.woken.store(true, Ordering::SeqCst);
        }
    }

    /// Waker which records whether stream locks were free when it has been
    /// called.
    struct LockCheckWaker {
        stream: DataStream<usize>,
        woken: AtomicBool,
        locks_free: AtomicBool,
    }

    impl LockCheckWaker {
        fn new(stream: DataStream<usize>) -> Self {
            Self {
                stream,
                woken: AtomicBool::new(false),
                locks_free: AtomicBool::new(false),
            }
        }
    }

    impl Wake for LockCheckWaker {
        fn wake(self: Arc<Self>) {
            let locks_free = self.stream.queue.try_write().is_some()
                && self.stream.waker.try_write().is_some()
                && self.stream.is_valid.try_write().is_some();
            self.locks_free.store(locks_free, Ordering::SeqCst);
            self.woken.store(true, Ordering::SeqCst);
        }
    }

    /// Waker which unparks the thread that created it.
    struct ThreadWaker(thread::Thread);

    impl Wake for ThreadWaker {
        fn wake(self: Arc<Self>) {
            self.0.unpark();
        }
    }

    fn poll_once<D>(stream: &mut DataStream<D>, waker: &Waker) -> Poll<Option<D>> {
        let mut ctx = Context::from_waker(waker);
        Pin::new(stream).poll_next(&mut ctx)
    }

    #[test]
    fn release_queue_lock_before_waking_listener() {
        let stream = DataStream::<usize>::with_queue_size(10);
        let waker_guard = stream.waker.write();

        let producer_stream = stream.clone();
        let producer = thread::spawn(move || producer_stream.push_data(1));

        let deadline = Instant::now() + LOCK_WAIT;
        let mut queue_released = false;
        while Instant::now() < deadline {
            if stream
                .queue
                .try_read()
                .is_some_and(|queue| queue.len() == 1)
            {
                queue_released = true;
                break;
            }
            thread::yield_now();
        }

        drop(waker_guard);
        producer.join().expect("producer thread panicked");

        assert!(
            queue_released,
            "push_data() held the queue lock while waiting for the waker lock"
        );
    }

    #[test]
    fn register_waker_without_holding_queue_lock() {
        let stream = DataStream::<usize>::with_queue_size(10);
        let queue_guard = stream.queue.write();

        let mut consumer_stream = stream.clone();
        let consumer = thread::spawn(move || {
            let waker = Waker::from(Arc::new(FlagWaker::default()));
            poll_once(&mut consumer_stream, &waker)
        });

        let deadline = Instant::now() + LOCK_WAIT;
        let mut waker_released = false;
        while Instant::now() < deadline {
            if stream.waker.try_read().is_some_and(|slot| slot.is_some()) {
                waker_released = true;
                break;
            }
            thread::yield_now();
        }

        drop(queue_guard);
        let poll = consumer.join().expect("consumer thread panicked");

        assert!(
            waker_released,
            "poll_next() held the waker lock while waiting for the queue lock"
        );
        assert!(poll.is_pending());
    }

    #[test]
    fn wake_listener_on_push_without_holding_stream_locks() {
        let mut stream = DataStream::<usize>::with_queue_size(10);
        let check = Arc::new(LockCheckWaker::new(stream.clone()));
        let waker = Waker::from(check.clone());

        assert!(poll_once(&mut stream, &waker).is_pending());
        stream.push_data(1);

        assert!(check.woken.load(Ordering::SeqCst));
        assert!(
            check.locks_free.load(Ordering::SeqCst),
            "push_data() woke the listener while holding a stream lock"
        );
    }

    #[test]
    fn wake_listener_on_invalidate_without_holding_stream_locks() {
        let mut stream = DataStream::<usize>::with_queue_size(10);
        let check = Arc::new(LockCheckWaker::new(stream.clone()));
        let waker = Waker::from(check.clone());

        assert!(poll_once(&mut stream, &waker).is_pending());
        stream.invalidate();

        assert!(check.woken.load(Ordering::SeqCst));
        assert!(
            check.locks_free.load(Ordering::SeqCst),
            "invalidate() woke the listener while holding a stream lock"
        );
    }

    #[test]
    fn wake_pending_listener_when_data_pushed() {
        let mut stream = DataStream::<usize>::with_queue_size(10);
        let flag = Arc::new(FlagWaker::default());
        let waker = Waker::from(flag.clone());

        assert!(poll_once(&mut stream, &waker).is_pending());
        stream.push_data(7);

        assert!(flag.woken.load(Ordering::SeqCst));
        assert_eq!(poll_once(&mut stream, &waker), Poll::Ready(Some(7)));
        assert!(poll_once(&mut stream, &waker).is_pending());
    }

    #[test]
    fn wake_pending_listener_and_end_stream_when_invalidated() {
        let mut stream = DataStream::<usize>::with_queue_size(10);
        let flag = Arc::new(FlagWaker::default());
        let waker = Waker::from(flag.clone());

        assert!(poll_once(&mut stream, &waker).is_pending());
        stream.invalidate();

        assert!(flag.woken.load(Ordering::SeqCst));
        assert_eq!(poll_once(&mut stream, &waker), Poll::Ready(None));
    }

    #[test]
    fn deliver_all_items_in_order_while_producer_and_listener_race() {
        const ROUNDS: usize = 200;
        const ITEMS: usize = 100;

        for round in 0..ROUNDS {
            let mut consumer_stream = DataStream::<usize>::with_queue_size(ITEMS);
            let producer_stream = consumer_stream.clone();
            let (result_tx, result_rx) = mpsc::channel();

            thread::spawn(move || {
                let waker = Waker::from(Arc::new(ThreadWaker(thread::current())));
                let mut received = Vec::with_capacity(ITEMS);
                while received.len() < ITEMS {
                    match poll_once(&mut consumer_stream, &waker) {
                        Poll::Ready(Some(item)) => received.push(item),
                        Poll::Ready(None) => break,
                        Poll::Pending => thread::park(),
                    }
                }
                let _ = result_tx.send(received);
            });
            let producer =
                thread::spawn(move || (0..ITEMS).for_each(|item| producer_stream.push_data(item)));

            let received = result_rx
                .recv_timeout(Duration::from_secs(10))
                .unwrap_or_else(|_| {
                    panic!("round {round}: listener stuck (deadlock or lost wake-up)")
                });
            producer.join().expect("producer thread panicked");

            assert_eq!(received, (0..ITEMS).collect::<Vec<_>>(), "round {round}");
        }
    }
}
