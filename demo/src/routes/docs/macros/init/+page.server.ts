import { highlightBlocks } from '$lib/highlight.server';
import type { PageServerLoad } from './$types';

const codeBlocks: Record<string, { code: string; lang: 'rust' }> = {
	sideEffect: {
		lang: 'rust',
		code: `// Side-effects only — logger, dotenv, tracing
async fn setup_logger() {
    env_logger::init();
}

#[rpc_query(init = "setup_logger")]
async fn get_data() -> Data { ... }

#[rpc_mutation(init = "setup_logger")]
async fn create_item(input: ItemInput) -> Item { ... }`
	},
	sharedState: {
		lang: 'rust',
		code: `// Return shared state — DB pool, HTTP client, config
async fn setup_db() -> PgPool {
    PgPool::connect(&env::var("DATABASE_URL").unwrap()).await.unwrap()
}

// The return type is injected as &T parameter
#[rpc_query(init = "setup_db")]
async fn get_users(pool: &PgPool) -> Vec<User> {
    sqlx::query_as("SELECT * FROM users")
        .fetch_all(pool).await.unwrap()
}

#[rpc_mutation(init = "setup_db")]
async fn create_user(pool: &PgPool, input: UserInput) -> User {
    sqlx::query_as("INSERT INTO users ...")
        .fetch_one(pool).await.unwrap()
}`
	},
	httpClient: {
		lang: 'rust',
		code: `// HTTP client for external API calls
async fn setup_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap()
}

#[rpc_query(init = "setup_client")]
async fn get_weather(client: &reqwest::Client, city: String) -> Weather {
    client.get(&format!("https://api.weather.com/{city}"))
        .send().await.unwrap()
        .json().await.unwrap()
}`
	},
	combined: {
		lang: 'rust',
		code: `// Combine with other attributes
#[rpc_query(init = "setup_db", cache = "5m", timeout = "10s")]
async fn get_stats(pool: &PgPool) -> Stats { ... }`
	},
	initDemoRust: {
		lang: 'rust',
		code: `pub struct AppState {
    pub cold_start_at: u64,
    pub init_duration_ms: u64,
    pub request_count: AtomicU64,
}

async fn setup() -> AppState {
    let start = Instant::now();
    let cold_start_at = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap().as_millis() as u64;

    // Simulate real init work
    tokio::time::sleep(Duration::from_millis(5)).await;

    let init_duration_ms = start.elapsed().as_millis() as u64;
    AppState { cold_start_at, init_duration_ms, request_count: AtomicU64::new(0) }
}

#[rpc_query(init = "setup")]
async fn init_demo(state: &AppState) -> InitDemoResponse {
    let count = state.request_count.fetch_add(1, Ordering::Relaxed) + 1;
    InitDemoResponse {
        cold_start_at: state.cold_start_at,
        init_duration_ms: state.init_duration_ms,
        request_count: count,
        now: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap().as_millis() as u64,
    }
}`
	}
};

export const load: PageServerLoad = () => highlightBlocks(codeBlocks);
