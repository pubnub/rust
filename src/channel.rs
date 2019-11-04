use std::collections::HashMap;

use crate::message::Message;
use crate::mvec::MVec;
use tokio::sync::mpsc;

pub(crate) type ChannelTx = mpsc::Sender<Message>;
pub(crate) type ChannelRx = mpsc::Receiver<Message>;
pub(crate) type ChannelMap = HashMap<String, MVec<ChannelTx>>;
