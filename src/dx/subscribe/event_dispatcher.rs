//! # Event dispatcher module
//!
//! This module contains the [`EventDispatcher`] type, which is used by
//! [`PubNubClientInstance`], [`Subscription2`] and [`SubscriptionSet`] to let
//! users attach listeners to the specific event types.

use spin::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::fmt::Debug;

use crate::{
    core::DataStream,
    lib::{
        alloc::{collections::VecDeque, vec::Vec},
        core::{default::Default, ops::Drop},
    },
    subscribe::{
        AppContext, ConnectionStatus, EventEmitter, File, Message, MessageAction, Presence,
        SubscribeStreamEvent, Update,
    },
};

#[derive(Debug)]
pub(crate) struct EventDispatcher {
    /// Whether listener streams has been created or not.
    has_streams: RwLock<bool>,

    /// A collection of data streams for message events.
    ///
    /// This struct holds a vector of `DataStream<Message>` instances, which
    /// provide a way to handle message events in a streaming fashion.
    pub(crate) message_streams: RwLock<Option<Vec<DataStream<Message>>>>,

    /// A collection of data streams for signal events.
    ///
    /// This struct holds a vector of `DataStream<Message>` instances, which
    /// provide a way to handle signal events in a streaming fashion.
    pub(crate) signal_streams: RwLock<Option<Vec<DataStream<Message>>>>,

    /// A collection of data streams for message reaction events.
    ///
    /// This struct holds a vector of `DataStream<MessageAction>` instances,
    /// which provide a way to handle message reactions events in a streaming
    /// fashion.
    pub(crate) message_reaction_streams: RwLock<Option<Vec<DataStream<MessageAction>>>>,

    /// A collection of data streams for file events.
    ///
    /// This struct holds a vector of `DataStream<File>` instances, which
    /// provide a way to handle file events in a streaming fashion.
    pub(crate) file_streams: RwLock<Option<Vec<DataStream<File>>>>,

    /// A collection of data streams for application context (Channel and User)
    /// events.
    ///
    /// This struct holds a vector of `DataStream<AppContext>` instances,
    /// which allow for handling application context events in a streaming
    /// fashion.
    pub(crate) app_context_streams: RwLock<Option<Vec<DataStream<AppContext>>>>,

    /// A collection of data streams for presence events.
    ///
    /// This struct holds a vector of `DataStream<Presence>` instances, which
    /// provide a way to handle presence events in a streaming fashion.
    pub(crate) presence_streams: RwLock<Option<Vec<DataStream<Presence>>>>,

    /// A collection of data streams for connection status change events.
    ///
    /// This struct holds a vector of `DataStream<ConnectionStatus>` instances,
    /// which provide a way to handle connection status change events in a
    /// streaming fashion.
    pub(crate) status_streams: RwLock<Option<Vec<DataStream<ConnectionStatus>>>>,

    /// A collection of data streams for update events.
    ///
    /// This struct holds a vector of `DataStream<Update>` instances, which
    /// provide a way to handle update events in a streaming fashion.
    pub(crate) streams: RwLock<Option<Vec<DataStream<Update>>>>,

    /// List of updates to be delivered to stream listener.
    pub(crate) updates: RwLock<VecDeque<SubscribeStreamEvent>>,
}

impl EventDispatcher {
    /// Create event dispatcher instance.
    ///
    /// Dispatcher, responsible for handling status and events and pushing them
    /// to the specific data streams that listen to them. Internal event queues
    /// prevent situations when events have been received before any listener
    /// has been attached (as soon as there will be at least one listener, the
    /// queue won't be filled).
    ///
    /// # Returns
    ///
    /// Returns [`EventDispatcher`] instance with pre-configured set of data
    /// streams.
    pub(crate) fn new() -> Self {
        Self {
            has_streams: Default::default(),
            message_streams: Default::default(),
            signal_streams: Default::default(),
            message_reaction_streams: Default::default(),
            file_streams: Default::default(),
            app_context_streams: Default::default(),
            presence_streams: Default::default(),
            status_streams: Default::default(),
            streams: Default::default(),
            updates: RwLock::new(VecDeque::with_capacity(100)),
        }
    }
    pub fn status_stream(&self) -> DataStream<ConnectionStatus> {
        let statuses = self.dequeue_matching_events(|event| match event {
            SubscribeStreamEvent::Status(status) => Some(status.clone()),
            _ => None,
        });

        self.create_stream_in_list(self.status_streams.write(), statuses)
    }

    /// Dispatch received connection status change.
    ///
    /// Dispatch events to the designated stream types.
    pub fn handle_status(&self, status: ConnectionStatus) {
        if !*self.has_streams.read() {
            let mut updates_slot = self.updates.write();
            updates_slot.push_back(SubscribeStreamEvent::Status(status));
            return;
        }

        self.push_event_to_stream(&status, &self.status_streams.read());
    }

    /// Dispatch received updates.
    ///
    /// Dispatch events to the designated stream types.
    pub fn handle_events(&self, events: Vec<Update>) {
        if !*self.has_streams.read() {
            let mut updates_slot = self.updates.write();
            updates_slot.extend(events.into_iter().map(SubscribeStreamEvent::Update));
            return;
        }

        let message_streams = self.message_streams.read();
        let signal_streams = self.signal_streams.read();
        let message_reactions_streams = self.message_reaction_streams.read();
        let file_streams = self.file_streams.read();
        let app_context_streams = self.app_context_streams.read();
        let presence_streams = self.presence_streams.read();
        let streams = self.streams.read();

        for event in events {
            match event.clone() {
                Update::Message(message) if message_streams.is_some() => {
                    self.push_event_to_stream(&message, &message_streams)
                }
                Update::Signal(signal) if signal_streams.is_some() => {
                    self.push_event_to_stream(&signal, &signal_streams)
                }
                Update::MessageAction(action) if message_reactions_streams.is_some() => {
                    self.push_event_to_stream(&action, &message_reactions_streams)
                }
                Update::File(file) if file_streams.is_some() => {
                    self.push_event_to_stream(&file, &file_streams)
                }
                Update::AppContext(object) if app_context_streams.is_some() => {
                    self.push_event_to_stream(&object, &app_context_streams)
                }
                Update::Presence(presence) if presence_streams.is_some() => {
                    self.push_event_to_stream(&presence, &presence_streams)
                }
                _ => {}
            }

            self.push_event_to_stream(&event, &streams);
        }
    }

    /// Create a new `DataStream` and add it to the given list of streams.
    ///
    /// # Arguments
    ///
    /// - `streams`: A mutable reference to an `Option<Vec<DataStream<S>>>`,
    ///   representing the list of streams.
    ///
    /// # Returns
    ///
    /// Returns the newly created `DataStream<S>`.
    fn create_stream_in_list<S>(
        &self,
        mut streams: RwLockWriteGuard<Option<Vec<DataStream<S>>>>,
        data: Option<VecDeque<S>>,
    ) -> DataStream<S>
    where
        S: Debug,
    {
        let mut has_streams_slot = self.has_streams.write();
        *has_streams_slot = true;
        let stream = if let Some(data) = data {
            DataStream::with_queue_data(data, 100)
        } else {
            DataStream::new()
        };

        if let Some(streams) = streams.as_mut() {
            streams.push(stream.clone())
        } else {
            *streams = Some(vec![stream.clone()])
        }

        stream
    }

    /// Pushes an event to each stream in the provided `streams`.
    ///
    /// # Arguments
    ///
    /// * `event` - A reference to the event to be pushed to the streams.
    /// * `streams` - A read-only lock guard that provides access to the option
    ///   containing the list of data streams.
    fn push_event_to_stream<S>(
        &self,
        event: &S,
        streams: &RwLockReadGuard<Option<Vec<DataStream<S>>>>,
    ) where
        S: Clone,
    {
        let Some(streams) = streams.as_ref() else {
            return;
        };

        streams
            .iter()
            .for_each(|stream| stream.push_data(event.clone()))
    }

    /// Dequeues and returns a vector of matching events from the queue.
    ///
    /// The `dequeue_matching_events` function takes a closure `condition` as
    /// input, which is used to determine whether an event matches the
    /// condition. The closure should accept a reference to a
    /// `SubscribeStreamEvent` and return a `bool`.
    ///
    /// If the event queue is not empty, the function iterates over the events
    /// in the queue and checks if each event matches the condition. If an
    /// event matches the condition, it is removed from the queue and added
    /// to a new `VecDeque` called `filtered`.
    ///
    /// After iterating over all events, if no events match the condition, the
    /// function returns `None`. Otherwise, it returns `Some(filtered)`,
    /// where `filtered` is a `VecDeque` containing the matching events.
    ///
    /// # Arguments
    ///
    /// * `condition` - A closure that determines whether an event matches the
    ///   condition.
    /// It should accept a reference to a `SubscribeStreamEvent` and return a
    /// `bool`.
    ///
    /// # Returns
    ///
    /// An `Option<VecDeque<E>>` - `Some(filtered)` if there are matching
    /// events, or `None` if no events match the condition.
    fn dequeue_matching_events<C, E>(&self, condition_map: C) -> Option<VecDeque<E>>
    where
        C: Fn(&SubscribeStreamEvent) -> Option<E>,
    {
        let mut updates = self.updates.write();
        let mut events: Option<VecDeque<E>> = None;

        if !updates.is_empty() {
            let mut filtered = VecDeque::with_capacity(100);
            let mut idx: usize = 0;

            while idx != updates.len() {
                if let Some(update) = condition_map(&updates[idx]) {
                    updates.remove(idx);
                    filtered.push_back(update);
                } else {
                    idx += 1;
                }
            }

            (!filtered.is_empty()).then(|| events = Some(filtered));
        }

        events
    }

    /// Invalidates all streams in the instance.
    pub(crate) fn invalidate(&self) {
        let mut has_streams_slot = self.has_streams.write();
        if !*has_streams_slot {
            return;
        }
        *has_streams_slot = false;

        if let Some(streams) = self.message_streams.write().as_mut() {
            streams.iter_mut().for_each(|stream| stream.invalidate());
            streams.clear();
        }

        if let Some(streams) = self.signal_streams.write().as_mut() {
            streams.iter_mut().for_each(|stream| stream.invalidate());
            streams.clear();
        }

        if let Some(streams) = self.message_reaction_streams.write().as_mut() {
            streams.iter_mut().for_each(|stream| stream.invalidate());
            streams.clear();
        }

        if let Some(streams) = self.file_streams.write().as_mut() {
            streams.iter_mut().for_each(|stream| stream.invalidate());
            streams.clear();
        }

        if let Some(streams) = self.app_context_streams.write().as_mut() {
            streams.iter_mut().for_each(|stream| stream.invalidate());
            streams.clear();
        }

        if let Some(streams) = self.presence_streams.write().as_mut() {
            streams.iter_mut().for_each(|stream| stream.invalidate());
            streams.clear();
        }

        if let Some(streams) = self.status_streams.write().as_mut() {
            streams.iter_mut().for_each(|stream| stream.invalidate());
            streams.clear();
        }

        if let Some(streams) = self.streams.write().as_mut() {
            streams.iter_mut().for_each(|stream| stream.invalidate());
            streams.clear();
        }
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for EventDispatcher {
    fn drop(&mut self) {
        self.invalidate()
    }
}

impl EventEmitter for EventDispatcher {
    fn messages_stream(&self) -> DataStream<Message> {
        let messages = self.dequeue_matching_events(|event| match event {
            SubscribeStreamEvent::Update(Update::Message(message)) => Some(message.clone()),
            _ => None,
        });

        self.create_stream_in_list(self.message_streams.write(), messages)
    }

    fn signal_stream(&self) -> DataStream<Message> {
        let signals = self.dequeue_matching_events(|event| match event {
            SubscribeStreamEvent::Update(Update::Signal(signal)) => Some(signal.clone()),
            _ => None,
        });

        self.create_stream_in_list(self.signal_streams.write(), signals)
    }

    fn message_actions_stream(&self) -> DataStream<MessageAction> {
        let reactions = self.dequeue_matching_events(|event| match event {
            SubscribeStreamEvent::Update(Update::MessageAction(reaction)) => Some(reaction.clone()),
            _ => None,
        });

        self.create_stream_in_list(self.message_reaction_streams.write(), reactions)
    }

    fn files_stream(&self) -> DataStream<File> {
        let files = self.dequeue_matching_events(|event| match event {
            SubscribeStreamEvent::Update(Update::File(file)) => Some(file.clone()),
            _ => None,
        });

        self.create_stream_in_list(self.file_streams.write(), files)
    }

    fn app_context_stream(&self) -> DataStream<AppContext> {
        let app_context = self.dequeue_matching_events(|event| match event {
            SubscribeStreamEvent::Update(Update::AppContext(app_context)) => {
                Some(app_context.clone())
            }
            _ => None,
        });

        self.create_stream_in_list(self.app_context_streams.write(), app_context)
    }

    fn presence_stream(&self) -> DataStream<Presence> {
        let presence = self.dequeue_matching_events(|event| match event {
            SubscribeStreamEvent::Update(Update::Presence(presence)) => Some(presence.clone()),
            _ => None,
        });

        self.create_stream_in_list(self.presence_streams.write(), presence)
    }

    fn stream(&self) -> DataStream<Update> {
        let updates = self.dequeue_matching_events(|event| match event {
            SubscribeStreamEvent::Update(update) => Some(update.clone()),
            _ => None,
        });

        self.create_stream_in_list(self.streams.write(), updates)
    }
}

#[cfg(test)]
mod it_should {
    use futures::StreamExt;
    use tokio::time::{timeout, Duration};

    use super::*;
    use crate::core::PubNubError;

    fn events() -> Vec<Update> {
        vec![
            Update::Message(Message {
                sender: Some("test-user-a".into()),
                timestamp: 0,
                channel: "test-channel".to_string(),
                subscription: "test-channel".to_string(),
                data: "Test message 1".to_string().into_bytes(),
                r#type: None,
                space_id: None,
                decryption_error: None,
            }),
            Update::Signal(Message {
                sender: Some("test-user-b".into()),
                timestamp: 0,
                channel: "test-channel".to_string(),
                subscription: "test-channel".to_string(),
                data: "Test signal 1".to_string().into_bytes(),
                r#type: None,
                space_id: None,
                decryption_error: None,
            }),
            Update::Presence(Presence::Join {
                timestamp: 0,
                uuid: "test-user-c".to_string(),
                channel: "test-channel".to_string(),
                subscription: "test-channel".to_string(),
                occupancy: 1,
                data: None,
                event_timestamp: 0,
            }),
            Update::Message(Message {
                sender: Some("test-user-c".into()),
                timestamp: 0,
                channel: "test-channel".to_string(),
                subscription: "test-channel".to_string(),
                data: "Test message 2".to_string().into_bytes(),
                r#type: None,
                space_id: None,
                decryption_error: None,
            }),
        ]
    }

    #[test]
    fn create_event_dispatcher() {
        let dispatcher = EventDispatcher::new();
        assert!(!*dispatcher.has_streams.read());
    }

    #[test]
    fn queue_events_when_there_no_listeners() {
        let dispatcher = EventDispatcher::new();
        let events = events();

        dispatcher.handle_status(ConnectionStatus::Connected);
        dispatcher.handle_events(events.clone());

        assert_eq!(dispatcher.updates.read().len(), events.len() + 1);
    }

    #[tokio::test]
    async fn dequeue_events_into_created_listener_streams() -> Result<(), PubNubError> {
        let dispatcher = EventDispatcher::new();
        let events = events();

        dispatcher.handle_status(ConnectionStatus::Connected);
        dispatcher.handle_events(events);

        let mut events_count = 0;
        let mut stream = dispatcher.messages_stream().take(10);
        loop {
            match timeout(Duration::from_millis(500), stream.next()).await {
                Ok(Some(_)) => events_count += 1,
                Err(_) => break,
                _ => {}
            }
        }
        assert_eq!(events_count, 2);

        let mut events_count = 0;
        let mut stream = dispatcher.signal_stream().take(10);
        loop {
            match timeout(Duration::from_millis(500), stream.next()).await {
                Ok(Some(_)) => events_count += 1,
                Err(_) => break,
                _ => {}
            }
        }
        assert_eq!(events_count, 1);

        let mut events_count = 0;
        let mut stream = dispatcher.presence_stream().take(10);
        loop {
            match timeout(Duration::from_millis(500), stream.next()).await {
                Ok(Some(_)) => events_count += 1,
                Err(_) => break,
                _ => {}
            }
        }
        assert_eq!(events_count, 1);

        let mut events_count = 0;
        let mut stream = dispatcher.status_stream().take(10);
        loop {
            match timeout(Duration::from_millis(500), stream.next()).await {
                Ok(Some(_)) => events_count += 1,
                Err(_) => break,
                _ => {}
            }
        }

        assert_eq!(events_count, 1);
        let mut events_count = 0;
        let mut stream = dispatcher.messages_stream().take(10);
        loop {
            match timeout(Duration::from_millis(500), stream.next()).await {
                Ok(Some(_)) => events_count += 1,
                Err(_) => break,
                _ => {}
            }
        }
        assert_eq!(events_count, 0);
        Ok(())
    }
}
