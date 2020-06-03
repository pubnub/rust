use crate::data::message::{self, Message};
use crate::data::pubsub;

#[derive(Debug)]
pub(super) struct MessageDestinations<'a> {
    message: &'a Message,
    route: bool,
    channel: bool,
}

impl<'a> MessageDestinations<'a> {
    pub fn new(message: &'a Message) -> Self {
        Self {
            message,
            route: false,
            channel: false,
        }
    }

    fn route_destination(&self) -> Option<pubsub::SubscribeTo> {
        match self.message.route {
            Some(message::Route::ChannelGroup(ref val)) => {
                Some(pubsub::SubscribeTo::ChannelGroup(val.clone()))
            }
            Some(message::Route::ChannelWildcard(ref val)) => {
                Some(pubsub::SubscribeTo::ChannelWildcard(val.clone()))
            }
            None => None,
        }
    }
}

impl<'a> Iterator for MessageDestinations<'a> {
    type Item = pubsub::SubscribeTo;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.route {
            self.route = true;
            let destination = self.route_destination();
            if destination.is_some() {
                return destination;
            }
        }
        if !self.channel {
            self.channel = true;
            return Some(pubsub::SubscribeTo::Channel(self.message.channel.clone()));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::MessageDestinations;
    use crate::data::message::{self, Message};
    use crate::data::pubsub;

    fn route_channel_wildcard(s: &'static str) -> Option<message::Route> {
        let val = s.parse().unwrap();
        Some(message::Route::ChannelWildcard(val))
    }

    fn route_channel_group(s: &'static str) -> Option<message::Route> {
        let val = s.parse().unwrap();
        Some(message::Route::ChannelGroup(val))
    }

    fn message(route: Option<message::Route>, channel: &'static str) -> Message {
        let channel = channel.parse().unwrap();
        Message {
            route,
            channel,
            ..Message::default()
        }
    }

    fn assert_iter_eq(m: &Message, expected: &[pubsub::SubscribeTo]) {
        let iter = MessageDestinations::new(&m);
        let vec: Vec<_> = iter.collect();
        assert_eq!(vec, expected);
    }

    #[test]
    fn test() {
        assert_iter_eq(
            &message(None, "test"),
            &[pubsub::SubscribeTo::Channel("test".parse().unwrap())],
        );

        assert_iter_eq(
            &message(route_channel_wildcard("qwe.*"), "test"),
            &[
                pubsub::SubscribeTo::ChannelWildcard("qwe.*".parse().unwrap()),
                pubsub::SubscribeTo::Channel("test".parse().unwrap()),
            ],
        );

        assert_iter_eq(
            &message(route_channel_group("qwe"), "test"),
            &[
                pubsub::SubscribeTo::ChannelGroup("qwe".parse().unwrap()),
                pubsub::SubscribeTo::Channel("test".parse().unwrap()),
            ],
        );
    }
}
