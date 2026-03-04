use crate::cli::{Config, OutputFormat};
use crate::extractor::extract_links;
use crate::logger;
use serde::Serialize;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, Semaphore};
use url::Url;

#[derive(Debug, Clone, Serialize)]
pub struct CrawlResult {
    pub url: String,
    pub status: u16,
    pub bytes: usize,
    pub depth: usize,
    pub success: bool,
    pub error: Option<String>,
}

pub async fn run(config: &Config) -> Vec<CrawlResult> {
    let base = match Url::parse(&config.url) {
        Ok(u) => u,
        Err(e) => {
            logger::error(&format!("Invalid URL '{}': {}", config.url, e));
            return vec![];
        }
    };

    let semaphore = Arc::new(Semaphore::new(config.concurrency));
    let visited: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));
    let results: Arc<Mutex<Vec<CrawlResult>>> = Arc::new(Mutex::new(Vec::new()));
    let active = Arc::new(AtomicUsize::new(0));

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<(String, usize)>();

    // Seed
    let seed = normalize_url(&config.url);
    visited.lock().await.insert(seed.clone());
    tx.send((seed, 0)).unwrap();

    let mut local_queue: VecDeque<(String, usize)> = VecDeque::new();

    loop {
        // Drain the channel into the local queue
        loop {
            match rx.try_recv() {
                Ok(item) => local_queue.push_back(item),
                Err(_) => break,
            }
        }

        if local_queue.is_empty() {
            if active.load(Ordering::SeqCst) == 0 {
                break;
            }
            // Yield and wait for tasks to push new items
            tokio::time::sleep(Duration::from_millis(10)).await;
            continue;
        }

        let (url, depth) = local_queue.pop_front().unwrap();

        // Depth gate
        if config.max_depth > 0 && depth > config.max_depth {
            continue;
        }

        let permit = semaphore.clone().acquire_owned().await.unwrap();

        let tx2 = tx.clone();
        let visited2 = visited.clone();
        let results2 = results.clone();
        let active2 = active.clone();
        let base2 = base.clone();
        let path = config.path.clone();
        let interval = config.interval;
        let max_depth = config.max_depth;
        let output_format = config.output.clone();

        active.fetch_add(1, Ordering::SeqCst);

        tokio::spawn(async move {
            if interval > 0 {
                tokio::time::sleep(Duration::from_millis(interval)).await;
            }

            let result = fetch_url(&url, depth).await;

            // Log streaming output for terminal mode
            if output_format == OutputFormat::Terminal {
                if result.success {
                    logger::success(&format!(
                        "✔ {} ({} bytes) {}",
                        result.status, result.bytes, result.url
                    ));
                } else {
                    let msg = match &result.error {
                        Some(e) => format!("✘ {} {}", e, result.url),
                        None => format!("✘ {} {}", result.status, result.url),
                    };
                    logger::error(&msg);
                }
            }

            // Extract links from successful HTML responses
            if result.success {
                if let Some(html) = result.body.as_deref() {
                    let new_links = extract_links(html, &base2, &path);
                    for link in new_links {
                        let normalized = normalize_url(&link);
                        let mut v = visited2.lock().await;
                        if v.insert(normalized.clone()) {
                            drop(v);
                            // Depth gate before queuing
                            if max_depth == 0 || depth + 1 <= max_depth {
                                tx2.send((normalized, depth + 1)).ok();
                            }
                        }
                    }
                }
            }

            results2.lock().await.push(CrawlResult {
                url: result.url,
                status: result.status,
                bytes: result.bytes,
                depth: result.depth,
                success: result.success,
                error: result.error,
            });

            active2.fetch_sub(1, Ordering::SeqCst);
            drop(permit);
        });
    }

    let final_results = Arc::try_unwrap(results)
        .expect("results arc still held")
        .into_inner();
    final_results
}

struct FetchOutcome {
    url: String,
    status: u16,
    bytes: usize,
    depth: usize,
    success: bool,
    error: Option<String>,
    body: Option<String>,
}

async fn fetch_url(url: &str, depth: usize) -> FetchOutcome {
    let client = build_client();
    match client.get(url).send().await {
        Ok(response) => {
            let status = response.status().as_u16();
            let success = status < 400;
            match response.text().await {
                Ok(body) => {
                    let bytes = body.len();
                    FetchOutcome {
                        url: url.to_string(),
                        status,
                        bytes,
                        depth,
                        success,
                        error: None,
                        body: if success { Some(body) } else { None },
                    }
                }
                Err(e) => FetchOutcome {
                    url: url.to_string(),
                    status,
                    bytes: 0,
                    depth,
                    success: false,
                    error: Some(e.to_string()),
                    body: None,
                },
            }
        }
        Err(e) => FetchOutcome {
            url: url.to_string(),
            status: 0,
            bytes: 0,
            depth,
            success: false,
            error: Some(e.to_string()),
            body: None,
        },
    }
}

fn build_client() -> reqwest::Client {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(10))
        .timeout(Duration::from_secs(30))
        .user_agent("spidermedic/0.2.0")
        .build()
        .expect("failed to build HTTP client")
}

fn normalize_url(url: &str) -> String {
    match Url::parse(url) {
        Ok(mut u) => {
            u.set_fragment(None);
            u.to_string()
        }
        Err(_) => url.to_string(),
    }
}
