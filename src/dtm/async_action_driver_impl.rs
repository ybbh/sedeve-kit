use std::sync::Arc;

use async_trait::async_trait;
use scupt_util::error_type::ET;
use scupt_util::message::Message;
use scupt_util::node_id::NID;
use scupt_util::res::Res;
use scupt_util::serde_json_string::SerdeJsonString;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot::{channel, Sender};
use tracing::trace;
use uuid::Uuid;

use crate::action::action_json::ActionJson;
use crate::action::action_type::{ActionBeginEnd, ActionType};
use crate::dtm::async_action_driver::AsyncActionDriver;
use crate::dtm::msg_ctrl::MessageControl;

pub struct AsyncActionDriverImplInner {
    node_id: NID,
    server_id: NID,
    sender: UnboundedSender<(Message<MessageControl>, Sender<Message<MessageControl>>)>,
}

pub struct AsyncActionDriverImpl {
    inner: Arc<AsyncActionDriverImplInner>,
}


impl AsyncActionDriverImpl {
    pub fn new(node_id: NID, server_node_id: NID, sender: UnboundedSender<(Message<MessageControl>, Sender<Message<MessageControl>>)>) -> Self {
        Self {
            inner: Arc::new(AsyncActionDriverImplInner::new(node_id, server_node_id, sender)),
        }
    }
}

#[async_trait]
impl AsyncActionDriver for AsyncActionDriverImpl {
    async fn action(
        &self,
        action_type: ActionType,
        action_begin_end: ActionBeginEnd,
        source: NID, dest: NID, payload_json: String) -> Res<()> {
        let action = ActionJson::from_json(action_type, source, dest, payload_json)?.to_serde_json_string();
        match action_begin_end {
            ActionBeginEnd::Begin => {
                self.inner.async_begin_action(action).await
            }
            ActionBeginEnd::End => {
                self.inner.async_end_action(action).await
            }
        }
    }
}

impl AsyncActionDriverImplInner {
    pub fn new(node_id: NID, server_id: NID, sender: UnboundedSender<(Message<MessageControl>, Sender<Message<MessageControl>>)>) -> Self {
        Self {
            node_id,
            server_id,
            sender,
        }
    }

    async fn async_begin_action(&self, action: SerdeJsonString) -> Res<()> {
        self.async_send_action(action, true).await
    }

    async fn async_end_action(&self, action: SerdeJsonString) -> Res<()> {
        self.async_send_action(action, false).await
    }

    async fn async_send_action(&self, action: SerdeJsonString, begin_action: bool) -> Res<()> {
        let uuid = Uuid::new_v4();
        let m = MessageControl::ActionReq {
            id: uuid.to_string(),
            action,
            begin: begin_action,
        };
        let req = Message::new(m, self.node_id, self.server_id);

        // send request
        let (req_sender, resp_receiver) = channel();
        match self.sender.send((req, req_sender)) {
            Ok(()) => {}
            Err(e) => { return Err(ET::SerdeError(e.to_string())); }
        };

        // receive response
        let response: MessageControl = match resp_receiver.await {
            Ok(m) => { m.payload() }
            Err(e) => { return Err(ET::RecvError(e.to_string())); }
        };

        trace!("receive response {:?}", response);
        match response {
            MessageControl::ActionACK { .. } => {}
            _ => { panic!("not possible") }
        }
        Ok(())
    }
}

