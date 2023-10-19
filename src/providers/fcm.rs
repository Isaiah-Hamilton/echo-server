use {
    crate::{
        blob::DecryptedPayloadBlob,
        error::Error,
        handlers::push_message::MessagePayload,
        providers::PushProvider,
    },
    async_trait::async_trait,
    fcm::{ErrorReason, FcmError, FcmResponse, MessageBuilder, NotificationBuilder, Priority},
    std::fmt::{Debug, Formatter},
    tracing::span,
};

pub struct FcmProvider {
    api_key: String,
    client: fcm::Client,
}

impl FcmProvider {
    pub fn new(api_key: String) -> Self {
        FcmProvider {
            api_key,
            client: fcm::Client::new(),
        }
    }
}

#[async_trait]
impl PushProvider for FcmProvider {
    async fn send_notification(
        &mut self,
        token: String,
        payload: MessagePayload,
    ) -> crate::error::Result<()> {
        let s = span!(tracing::Level::DEBUG, "send_fcm_notification");
        let _ = s.enter();

        let mut message_builder = MessageBuilder::new(self.api_key.as_str(), token.as_str());

        let result = if payload.is_encrypted() {
            message_builder.data(&payload)?;

            // Must set priority=high and content-available=true on data-only messages or
            // they don't show unless app is active
            // https://rnfirebase.io/messaging/usage#data-only-messages
            message_builder.priority(Priority::High);
            message_builder.content_available(true);

            let fcm_message = message_builder.finalize();

            self.client.send(fcm_message).await
        } else {
            let blob = DecryptedPayloadBlob::from_base64_encoded(payload.clone().blob)?;

            let mut notification_builder = NotificationBuilder::new();
            notification_builder.title(blob.title.as_str());
            notification_builder.body(blob.body.as_str());
            let notification = notification_builder.finalize();

            message_builder.notification(notification);
            message_builder.data(&payload)?;

            let fcm_message = message_builder.finalize();

            self.client.send(fcm_message).await
        };

        match result {
            Ok(val) => {
                let FcmResponse { error, .. } = val;
                if let Some(error) = error {
                    match error {
                        ErrorReason::MissingRegistration => Err(Error::BadDeviceToken(
                            "Missing registration for token".into(),
                        )),
                        ErrorReason::InvalidRegistration => {
                            Err(Error::BadDeviceToken("Invalid token registration".into()))
                        }
                        ErrorReason::NotRegistered => {
                            Err(Error::BadDeviceToken("Token is not registered".into()))
                        }
                        ErrorReason::InvalidApnsCredential => Err(Error::BadApnsCredentials),
                        e => Err(Error::FcmResponse(e)),
                    }
                } else {
                    Ok(())
                }
            }
            Err(e) => match e {
                FcmError::Unauthorized => Err(Error::BadFcmApiKey),
                e => Err(Error::Fcm(e)),
            },
        }
    }
}

// Manual Impl Because `fcm::Client` does not derive anything and doesn't need
// to be accounted for

impl Clone for FcmProvider {
    fn clone(&self) -> Self {
        FcmProvider {
            api_key: self.api_key.clone(),
            client: fcm::Client::new(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.api_key = source.api_key.clone();
        self.client = fcm::Client::new();
    }
}

impl PartialEq for FcmProvider {
    fn eq(&self, other: &Self) -> bool {
        self.api_key == other.api_key
    }
}

impl Debug for FcmProvider {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[FcmProvider] api_key = {}", self.api_key)
    }
}
