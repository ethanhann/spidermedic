use clap::Parser;
use spidermedic::cli::Config;
use spidermedic::{crawler, logger, reporter};
use std::process::ExitCode;
use url::Url;

#[tokio::main]
async fn main() -> ExitCode {
    let mut config = Config::parse();
    apply_port(&mut config);

    logger::info(&format!("Validating {}", config.url));

    let results = crawler::run(&config).await;

    reporter::report(&results, &config.output);

    if results.iter().any(|r| !r.success) {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn apply_port(config: &mut Config) {
    if config.port == 0 {
        return;
    }
    if let Ok(mut u) = Url::parse(&config.url) {
        u.set_port(Some(config.port)).ok();
        config.url = u.to_string();
    }
}
