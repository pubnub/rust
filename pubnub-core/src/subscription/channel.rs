use crate::message::Message;
use futures_channel::mpsc;

pub(crate) type Tx = mpsc::Sender<Message>;
pub(crate) type Rx = mpsc::Receiver<Message>;
