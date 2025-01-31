pub mod apns;
pub mod fcm;
pub mod noop;

use crate::handlers::push_message::MessagePayload;
use crate::providers::apns::ApnsProvider;
#[cfg(any(debug_assertions, test))]
use crate::providers::noop::NoopProvider;
use crate::{error, providers::fcm::FcmProvider};
use async_trait::async_trait;
use tracing::span;

#[async_trait]
pub trait PushProvider {
    async fn send_notification(
        &mut self,
        token: String,
        payload: MessagePayload,
    ) -> error::Result<()>;
}

const PROVIDER_APNS: &str = "apns";
const PROVIDER_FCM: &str = "fcm";
#[cfg(any(debug_assertions, test))]
const PROVIDER_NOOP: &str = "noop";

#[derive(Debug, Copy, Clone, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "provider")]
#[sqlx(rename_all = "lowercase")]
pub enum ProviderKind {
    Apns,
    Fcm,
    #[cfg(any(debug_assertions, test))]
    Noop,
}

impl ProviderKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Apns => PROVIDER_APNS,
            Self::Fcm => PROVIDER_FCM,
            #[cfg(any(debug_assertions, test))]
            Self::Noop => PROVIDER_NOOP,
        }
    }
}

impl Into<String> for ProviderKind {
    fn into(self) -> String {
        self.as_str().to_string()
    }
}

impl Into<String> for &ProviderKind {
    fn into(self) -> String {
        self.as_str().to_string()
    }
}

impl From<ProviderKind> for &str {
    fn from(val: ProviderKind) -> Self {
        val.as_str()
    }
}

impl TryFrom<&str> for ProviderKind {
    type Error = error::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            PROVIDER_APNS => Ok(Self::Apns),
            PROVIDER_FCM => Ok(Self::Fcm),
            #[cfg(any(debug_assertions, test))]
            PROVIDER_NOOP => Ok(Self::Noop),
            _ => Err(error::Error::ProviderNotFound(value.to_owned())),
        }
    }
}

#[allow(clippy::large_enum_variant)]
pub enum Provider {
    Fcm(FcmProvider),
    Apns(ApnsProvider),
    #[cfg(any(debug_assertions, test))]
    Noop(NoopProvider),
}

#[async_trait]
impl PushProvider for Provider {
    async fn send_notification(
        &mut self,
        token: String,
        payload: MessagePayload,
    ) -> error::Result<()> {
        let s = span!(tracing::Level::INFO, "send_notification");
        let _ = s.enter();
        match self {
            Provider::Fcm(p) => p.send_notification(token, payload).await,
            Provider::Apns(p) => p.send_notification(token, payload).await,
            #[cfg(any(debug_assertions, test))]
            Provider::Noop(p) => p.send_notification(token, payload).await,
        }
    }
}
