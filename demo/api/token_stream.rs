use metaxy::{StreamSender, rpc_stream};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct TokenStreamInput {
    /// The prompt to "generate" tokens for.
    pub prompt: String,
}

#[derive(Serialize)]
pub struct Token {
    /// The generated token text.
    pub text: String,
    /// Token index in the sequence (0-based).
    pub index: u32,
}

/// Simulates LLM-style token streaming by splitting the prompt into words
/// and streaming each word back as a token with a small delay.
///
/// This demonstrates the typical pattern for AI/LLM integrations where
/// the response is generated incrementally.
#[rpc_stream(timeout = "60s")]
async fn token_stream(input: TokenStreamInput, tx: StreamSender) {
    let words: Vec<&str> = input.prompt.split_whitespace().collect();

    for (i, word) in words.iter().enumerate() {
        let separator = if i == 0 { "" } else { " " };
        let token = Token {
            text: format!("{separator}{word}"),
            index: i as u32,
        };
        if tx.send(token).await.is_err() {
            break;
        }
        // Simulate generation latency (50-150ms per token)
        let delay = 50 + (word.len() as u64 * 15);
        tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
    }
}
