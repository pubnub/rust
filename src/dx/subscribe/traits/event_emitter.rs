//! # Event emitter module.
//!
//! This module contains the [`EventEmitter`] trait, which is used to implement
//! a real-time event emitter.

use crate::{
    core::DataStream,
    subscribe::{AppContext, File, Message, MessageAction, Presence, Update},
};

/// Events emitter trait.
///
/// Types that implement this trait provide various streams, which are dedicated
/// to specific events. to specific events.
pub trait EventEmitter {
    /// Stream used to notify regular messages.
    fn messages_stream(&self) -> DataStream<Message>;

    /// Stream used to notify signals.
    fn signal_stream(&self) -> DataStream<Message>;

    /// Stream used to notify message action updates.
    fn message_actions_stream(&self) -> DataStream<MessageAction>;

    /// Stream used to notify about file receive.
    fn files_stream(&self) -> DataStream<File>;

    /// Stream used to notify about App Context (Channel and User) updates.
    fn app_context_stream(&self) -> DataStream<AppContext>;

    /// Stream used to notify about subscribers' presence updates.
    fn presence_stream(&self) -> DataStream<Presence>;

    /// Generic stream used to notify all updates mentioned above.
    fn stream(&self) -> DataStream<Update>;
}
