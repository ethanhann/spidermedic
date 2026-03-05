use spidermedic::cli::{Config, OutputFormat};
use spidermedic::crawler::run;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn make_config(url: String) -> Config {
    Config {
        url,
        path: "/".to_string(),
        port: 80,
        interval: 0,
        concurrency: 4,
        max_depth: 0,
        output: OutputFormat::Terminal,
    }
}

#[tokio::test]
async fn test_crawl_all_200() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"<a href="/about">About</a>"#))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/about"))
        .respond_with(ResponseTemplate::new(200).set_body_string("<h1>About</h1>"))
        .mount(&mock_server)
        .await;

    let results = run(&make_config(mock_server.uri())).await;
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|r| r.success));
}

#[tokio::test]
async fn test_crawl_detects_404() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(r#"<a href="/missing">Missing</a>"#),
        )
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/missing"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    let results = run(&make_config(mock_server.uri())).await;
    let failed: Vec<_> = results.iter().filter(|r| !r.success).collect();
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0].status, 404);
}

#[tokio::test]
async fn test_no_duplicate_visits() {
    let mock_server = MockServer::start().await;

    // Both / and /page1 link to /shared
    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"<a href="/page1">P1</a><a href="/shared">Shared</a>"#),
        )
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/page1"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"<a href="/shared">Shared</a>"#))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/shared"))
        .respond_with(ResponseTemplate::new(200).set_body_string("<h1>Shared</h1>"))
        .mount(&mock_server)
        .await;

    let results = run(&make_config(mock_server.uri())).await;
    let shared: Vec<_> = results
        .iter()
        .filter(|r| r.url.contains("/shared"))
        .collect();
    assert_eq!(
        shared.len(),
        1,
        "shared page should be visited exactly once"
    );
}

#[tokio::test]
async fn test_max_depth_limits_crawl() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"<a href="/depth1">D1</a>"#))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/depth1"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"<a href="/depth2">D2</a>"#))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/depth2"))
        .respond_with(ResponseTemplate::new(200).set_body_string("<h1>D2</h1>"))
        .mount(&mock_server)
        .await;

    let mut config = make_config(mock_server.uri());
    config.max_depth = 1;

    let results = run(&config).await;
    // Should visit / (depth 0) and /depth1 (depth 1), but not /depth2 (depth 2)
    assert_eq!(results.len(), 2);
    assert!(!results.iter().any(|r| r.url.contains("/depth2")));
}
