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

        let mut queue_data_slot = self.queue.write();

        // Dropping the earliest entry to prevent the queue from growing too large.
        if queue_data_slot.len() == queue_data_slot.capacity() {
            queue_data_slot.pop_front();
        }

        queue_data_slot.push_back(data);

        self.wake_stream();
    }

    #[cfg(all(feature = "subscribe", feature = "std"))]
    pub(crate) fn invalidate(&self) {
        let mut is_valid = self.is_valid.write();
        *is_valid = false;
        self.wake_stream();
    }

    #[cfg(all(feature = "subscribe", feature = "std"))]
    fn wake_stream(&self) {
        if let Some(waker) = self.waker.write().take() {
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

        let mut waker_slot = self.waker.write();
        *waker_slot = Some(ctx.waker().clone());

        if let Some(data) = self.queue.write().pop_front() {
            Poll::Ready(Some(data))
        } else {
            Poll::Pending
        }
    }
}
