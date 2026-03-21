use hyper::body::Bytes;
use serde::Serialize;
use tokio::sync::mpsc;

/// Error returned when the streaming channel is closed.
#[derive(Debug)]
pub struct SendError;

impl std::fmt::Display for SendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "stream channel closed")
    }
}

impl std::error::Error for SendError {}

/// A typed sender for streaming RPC responses.
///
/// Wraps an internal channel and serializes each value as an SSE `data:` event
/// before sending it to the client.
///
/// # Example
///
/// ```rust,ignore
/// use metaxy::{rpc_stream, StreamSender};
///
/// #[rpc_stream]
/// async fn chat(input: ChatInput, tx: StreamSender) {
///     for token in generate_tokens(&input.prompt) {
///         tx.send(token).await.ok();
///     }
/// }
/// ```
pub struct StreamSender {
    tx: mpsc::Sender<Result<Bytes, std::convert::Infallible>>,
}

impl StreamSender {
    /// Creates a new `StreamSender` wrapping the given channel.
    #[doc(hidden)]
    pub fn new(tx: mpsc::Sender<Result<Bytes, std::convert::Infallible>>) -> Self {
        Self { tx }
    }

    /// Sends a serializable value as an SSE `data:` event.
    ///
    /// The value is serialized to JSON and formatted as:
    /// ```text
    /// data: {"token":"Hello"}\n\n
    /// ```
    pub async fn send<T: Serialize>(&self, data: T) -> Result<(), SendError> {
        let json = serde_json::to_string(&data).map_err(|_| SendError)?;
        let event = format!("data: {json}\n\n");
        self.tx
            .send(Ok(Bytes::from(event)))
            .await
            .map_err(|_| SendError)
    }

    /// Sends a raw string as an SSE `data:` event.
    pub async fn send_raw(&self, data: impl Into<String>) -> Result<(), SendError> {
        let event = format!("data: {}\n\n", data.into());
        self.tx
            .send(Ok(Bytes::from(event)))
            .await
            .map_err(|_| SendError)
    }
}
