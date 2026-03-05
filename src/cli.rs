use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "spidermedic",
    about = "Crawl a website and validate HTTP responses",
    version
)]
pub struct Config {
    /// Starting URL to crawl
    #[arg(long, required = true)]
    pub url: String,

    /// URL path prefix to restrict crawling
    #[arg(long, default_value = "/")]
    pub path: String,

    /// Port to connect on (overrides port in URL; 0 = use scheme default)
    #[arg(long, default_value_t = 0)]
    pub port: u16,

    /// Milliseconds between requests (rate limiting)
    #[arg(long, default_value_t = 300)]
    pub interval: u64,

    /// Maximum number of concurrent in-flight requests
    #[arg(long, default_value_t = 10)]
    pub concurrency: usize,

    /// Maximum crawl depth from the seed URL (0 = unlimited)
    #[arg(long, default_value_t = 0)]
    pub max_depth: usize,

    /// Output format for results
    #[arg(long, value_enum, default_value_t = OutputFormat::Terminal)]
    pub output: OutputFormat,
}

#[derive(clap::ValueEnum, Debug, Clone, PartialEq)]
pub enum OutputFormat {
    Terminal,
    Json,
    Csv,
}
