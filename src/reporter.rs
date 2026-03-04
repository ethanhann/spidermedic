use crate::cli::OutputFormat;
use crate::crawler::CrawlResult;
use colored::Colorize;

pub fn report(results: &[CrawlResult], format: &OutputFormat) {
    match format {
        OutputFormat::Terminal => report_terminal(results),
        OutputFormat::Json => report_json(results),
        OutputFormat::Csv => report_csv(results),
    }
}

fn report_terminal(results: &[CrawlResult]) {
    let total = results.len();
    let failed = results.iter().filter(|r| !r.success).count();
    eprintln!(
        "{}",
        format!(
            "All queued items have been processed ({total} total, {failed} failed)"
        )
        .blue()
    );
}

fn report_json(results: &[CrawlResult]) {
    let json = serde_json::to_string_pretty(results).expect("serialization failed");
    println!("{json}");
}

fn report_csv(results: &[CrawlResult]) {
    let mut wtr = csv::Writer::from_writer(std::io::stdout());
    wtr.write_record(["url", "status", "bytes", "depth", "success", "error"])
        .unwrap();
    for r in results {
        wtr.write_record(&[
            r.url.as_str(),
            &r.status.to_string(),
            &r.bytes.to_string(),
            &r.depth.to_string(),
            &r.success.to_string(),
            r.error.as_deref().unwrap_or(""),
        ])
        .unwrap();
    }
    wtr.flush().unwrap();
}
