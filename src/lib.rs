use std::num::NonZeroU32;
use std::string::ParseError;
use std::sync::Arc;

use async_trait::async_trait;
use ethers::providers::{JsonRpcClient, ProviderError};
use governor::clock::{QuantaClock, QuantaInstant};
use governor::middleware::NoOpMiddleware;
use governor::state::{InMemoryState, NotKeyed};
use governor::{Jitter, Quota, RateLimiter};
use serde::de::DeserializeOwned;
use serde::Serialize;

pub type Throttle =
    RateLimiter<NotKeyed, InMemoryState, QuantaClock, NoOpMiddleware<QuantaInstant>>;

#[derive(Clone, Debug)]
pub struct ThrottledJsonRpcClient<P: JsonRpcClient> {
    throttle: Arc<Throttle>,
    jitter: Option<Jitter>,
    inner: P,
}

impl<P: JsonRpcClient> ThrottledJsonRpcClient<P> {
    pub fn new(client: P, requests_per_second: u32, jitter: Option<Jitter>) -> Self {
        let throttle = Arc::new(RateLimiter::direct(Quota::per_second(
            NonZeroU32::new(requests_per_second).expect("Could not initialize NonZeroU32"),
        )));

        ThrottledJsonRpcClient {
            throttle,
            jitter,
            inner: client,
        }
    }
}

#[async_trait]
impl<P: JsonRpcClient> JsonRpcClient for ThrottledJsonRpcClient<P> {
    type Error = P::Error;

    /// Sends a request with the provided JSON-RPC and parameters serialized as JSON
    async fn request<T, R>(&self, method: &str, params: T) -> Result<R, Self::Error>
    where
        T: std::fmt::Debug + Serialize + Send + Sync,
        R: DeserializeOwned + Send,
    {
        if let Some(jitter) = self.jitter {
            self.throttle.until_ready_with_jitter(jitter).await;
        } else {
            self.throttle.until_ready().await;
        }

        self.inner.request(method, params).await
    }
}
