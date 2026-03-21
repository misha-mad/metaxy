import { highlightBlocks } from '$lib/highlight.server';
import type { PageServerLoad } from './$types';

const codeBlocks: Record<string, { code: string; lang: 'rust' | 'typescript' }> = {
	countdownRust: {
		lang: 'rust',
		code: `use metaxy::{rpc_stream, StreamSender};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct CountdownInput {
    pub from: u32,
    pub delay_ms: u64,
}

#[derive(Serialize)]
pub struct CountdownTick {
    pub remaining: u32,
    pub message: String,
}

#[rpc_stream(timeout = "30s")]
async fn countdown(input: CountdownInput, tx: StreamSender) {
    let from = input.from.min(10);
    let delay = Duration::from_millis(input.delay_ms.max(100));

    for i in (0..=from).rev() {
        let tick = CountdownTick {
            remaining: i,
            message: if i == 0 { "Done!".into() } else { format!("{i}...") },
        };
        if tx.send(tick).await.is_err() { break; }
        if i > 0 { tokio::time::sleep(delay).await; }
    }
}`
	},
	tokenStreamRust: {
		lang: 'rust',
		code: `use metaxy::{rpc_stream, StreamSender};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct TokenStreamInput {
    pub prompt: String,
}

#[derive(Serialize)]
pub struct Token {
    pub text: String,
    pub index: u32,
}

#[rpc_stream(timeout = "60s")]
async fn token_stream(input: TokenStreamInput, tx: StreamSender) {
    let words: Vec<&str> = input.prompt.split_whitespace().collect();

    for (i, word) in words.iter().enumerate() {
        let sep = if i == 0 { "" } else { " " };
        let token = Token { text: format!("{sep}{word}"), index: i as u32 };
        if tx.send(token).await.is_err() { break; }
        tokio::time::sleep(Duration::from_millis(50 + word.len() as u64 * 15)).await;
    }
}`
	},
	clientUsage: {
		lang: 'typescript',
		code: `// Direct call — async generator
for await (const chunk of rpc.stream('countdown', { from: 5, delay_ms: 500 })) {
  console.log(chunk.remaining, chunk.message);
}

// With call options
for await (const chunk of rpc.stream('token_stream', { prompt: "Hello world" }, {
  signal: controller.signal,
})) {
  process(chunk);
}`
	},
	svelteUsage: {
		lang: 'typescript',
		code: `// Reactive wrapper — manages chunks, state, and cleanup
const stream = createStream(rpc, 'countdown', () => ({
  from: countdownFrom,
  delay_ms: 500,
}));

// stream.chunks     — CountdownTick[]
// stream.isStreaming — boolean
// stream.isDone     — boolean
// stream.start()    — begin streaming
// stream.stop()     — abort`
	}
};

export const load: PageServerLoad = () => highlightBlocks(codeBlocks);
