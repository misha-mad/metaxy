use metaxy::{StreamSender, rpc_stream};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct CountdownInput {
    /// Number to count down from (max 10).
    pub from: u32,
    /// Delay between ticks in milliseconds.
    pub delay_ms: u64,
}

#[derive(Serialize)]
pub struct CountdownTick {
    pub remaining: u32,
    pub message: String,
}

/// Counts down from a given number, streaming each tick to the client.
/// Demonstrates basic streaming with typed input and structured output chunks.
#[rpc_stream(timeout = "30s")]
async fn countdown(input: CountdownInput, tx: StreamSender) {
    let from = input.from.min(10);
    let delay = std::time::Duration::from_millis(input.delay_ms.max(100).min(2000));

    for i in (0..=from).rev() {
        let tick = CountdownTick {
            remaining: i,
            message: if i == 0 {
                "Done!".to_string()
            } else {
                format!("{i}...")
            },
        };
        if tx.send(tick).await.is_err() {
            break;
        }
        if i > 0 {
            tokio::time::sleep(delay).await;
        }
    }
}
