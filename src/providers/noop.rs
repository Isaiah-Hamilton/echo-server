use crate::handlers::push_message::MessagePayload;
#[cfg(any(debug_assertions, test))]
use crate::providers::PushProvider;
#[cfg(any(debug_assertions, test))]
use async_trait::async_trait;
use std::collections::HashMap;
use tracing::span;

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct NoopProvider {
    // token -> [MessagePayload{..}, MessagePayload{..}, MessagePayload{..}]
    notifications: HashMap<String, Vec<MessagePayload>>,
}

#[cfg(any(debug_assertions, test))]
impl NoopProvider {
    pub fn new() -> Self {
        Default::default()
    }
}

#[async_trait]
#[cfg(any(debug_assertions, test))]
impl PushProvider for NoopProvider {
    async fn send_notification(
        &mut self,
        token: String,
        payload: MessagePayload,
    ) -> crate::error::Result<()> {
        let s = span!(tracing::Level::DEBUG, "send_noop_notification");
        let _ = s.enter();

        self.bootstrap(token.clone());

        let notifications = self.notifications.get_mut(&token).unwrap();
        notifications.append(&mut vec![payload]);

        Ok(())
    }
}

// Utils
#[cfg(any(debug_assertions, test))]
impl NoopProvider {
    /// Insert empty notifications for a new token
    fn bootstrap(&mut self, token: String) {
        self.notifications.entry(token).or_insert_with(Vec::new);
    }
}
