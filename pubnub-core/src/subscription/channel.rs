use crate::message::Message;
use tokio::sync::mpsc;

pub(crate) type Tx = mpsc::Sender<Message>;
pub(crate) type Rx = mpsc::Receiver<Message>;
