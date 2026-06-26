//! Rust 后端集成测试（真实网络 + 本地 DB），评估 Command 层完成度。

use app_lib::adapters::{AkshareClient, JinshiClient};
use app_lib::testing::bootstrap_test_state;
use app_lib::config::Config;
use app_lib::db::Database;
use app_lib::engine::indicator;
use app_lib::engine::sectors;
use app_lib::engine::facts_from_dimension_summary;
use app_lib::models::{AnalysisReport, FollowupMessage, KLine};
use chrono::{Duration, Utc};
use reqwest::Client;

fn temp_db() -> Database {
    let path = std::env::temp_dir().join(format!(
        "thisismyquant-test-{}.db",
        uuid::Uuid::new_v4()
    ));
    let db = Database::open(&path).expect("open temp db");
    db.init_schema().expect("schema");
    db
}

#[tokio::test]
async fn liquidity_snapshot_roundtrip() {
    use app_lib::engine::liquidity;
    use app_lib::engine::sectors::LiquidityTier;
    use app_lib::models::LiquiditySnapshot;

    let db = temp_db();
    let snap = LiquiditySnapshot {
        symbol: "RB0".into(),
        volume_20d: 12000.0,
        turnover_20d: 800_000_000.0,
        score: 0.8,
        tier: "core".into(),
        scored_at: Utc::now().to_rfc3339(),
    };
    db.save_liquidity_snapshot(&snap).expect("save");
    let map = db.get_latest_liquidity_map().expect("load");
    assert_eq!(map.get("RB0").map(|s| s.tier.as_str()), Some("core"));

    let klines: Vec<KLine> = (0..20)
        .map(|i| KLine {
            symbol: "rb0".into(),
            interval: "1d".into(),
            open: 1.0,
            high: 1.0,
            low: 1.0,
            close: 1.0,
            volume: 20_000,
            turnover: 900_000_000.0,
            start_time: (Utc::now() - Duration::days(19 - i)).to_rfc3339(),
        })
        .collect();
    let cfg = app_lib::config::LiquidityConfig {
        min_volume_20d: 5000.0,
        min_turnover_20d: 500_000_000.0,
    };
    let built = liquidity::build_snapshot("RB0", &klines, LiquidityTier::Core, &cfg);
    assert_eq!(built.tier, "core");
}

#[tokio::test]
async fn product_catalog_core_filter() {
    use app_lib::engine::sectors;
    use std::collections::HashMap;

    let catalog = sectors::build_catalog("core", &HashMap::new());
    let count: usize = catalog.iter().map(|s| s.products.len()).sum();
    assert!(count >= 32, "core catalog should have 32+ products, got {count}");
    assert!(catalog.iter().any(|s| s.code == "shipping"));
}

#[tokio::test]
async fn unclassified_news_query() {
    use app_lib::models::{NewsRecord, news_content_hash};

    let db = temp_db();
    let hash = news_content_hash("未分类新闻", "测试");
    db.save_news(&NewsRecord {
        id: "news-uncls".into(),
        source: "jin10".into(),
        category_id: Some(52042),
        title: "未分类新闻".into(),
        summary: "测试".into(),
        url: String::new(),
        display_time: Utc::now().to_rfc3339(),
        content_hash: hash,
        ingested_at: Utc::now().to_rfc3339(),
    })
    .expect("save");
    let pending = db.get_unclassified_news(10).expect("query");
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].id, "news-uncls");
}

#[tokio::test]
async fn news_ingest_and_classify_roundtrip() {
    use app_lib::engine::news_classifier;
    use app_lib::models::{NewsRecord, news_content_hash};

    let db = temp_db();
    let hash = news_content_hash("螺纹钢库存下降", "社会库存环比减少");
    let record = NewsRecord {
        id: "news-test-1".into(),
        source: "jin10".into(),
        category_id: Some(52018),
        title: "螺纹钢库存下降".into(),
        summary: "社会库存环比减少，钢厂铁水产量回升".into(),
        url: String::new(),
        display_time: Utc::now().to_rfc3339(),
        content_hash: hash,
        ingested_at: Utc::now().to_rfc3339(),
    };
    db.save_news(&record).expect("save news");
    let labels = news_classifier::classify(&record);
    assert!(!labels.is_empty());
    db.save_classifications(&labels).expect("save cls");

    let items = db.get_news_for_symbol("RB0", None, 10).expect("query");
    assert!(!items.is_empty());
    assert!(items[0].classifications.iter().any(|c| c.dimension_code == "inventory"));
}

#[tokio::test]
async fn contracts_from_sectors() {
    let contracts = sectors::all_contracts();
    assert!(contracts.len() >= 30, "expected 30+ products, got {}", contracts.len());
    assert!(contracts.iter().any(|c| c.symbol == "RB0"));
}

#[tokio::test]
async fn db_klines_roundtrip() {
    let db = temp_db();
    let klines = vec![KLine {
        symbol: "rb0".into(),
        interval: "1d".into(),
        open: 3100.0,
        high: 3120.0,
        low: 3080.0,
        close: 3110.0,
        volume: 1000,
        turnover: 0.0,
        start_time: Utc::now().to_rfc3339(),
    }];
    db.save_klines(&klines).expect("save");
    let end = Utc::now();
    let start = end - Duration::days(1);
    let loaded = db.get_klines("rb0", "1d", start, end, 10).expect("load");
    assert_eq!(loaded.len(), 1);
    assert!((loaded[0].close - 3110.0).abs() < f64::EPSILON);
}

#[tokio::test]
async fn indicator_summary_from_klines() {
    let klines: Vec<KLine> = (0..30)
        .map(|i| KLine {
            symbol: "rb0".into(),
            interval: "1d".into(),
            open: 3000.0 + i as f64,
            high: 3010.0 + i as f64,
            low: 2990.0 + i as f64,
            close: 3005.0 + i as f64,
            volume: 1000,
            turnover: 0.0,
            start_time: (Utc::now() - Duration::days(29 - i)).to_rfc3339(),
        })
        .collect();
    let summary = indicator::summary(&klines);
    assert!(summary.contains_key("ma5"));
    assert!(summary.contains_key("macd_hist"));
}

#[tokio::test]
async fn akshare_daily_kline_live() {
    let http = Client::builder()
        .user_agent("ThisIsMyQuant-E2E/0.1")
        .build()
        .unwrap();
    let client = AkshareClient::new(http);
    let end = Utc::now();
    let start = end - Duration::days(120);
    let klines = client
        .get_history("rb2510", "1d", start, end)
        .await
        .expect("daily klines");
    assert!(!klines.is_empty(), "expected daily klines (incl. 60m supplement)");
    assert!(klines.last().unwrap().close > 0.0);
    if let Some(latest) = app_lib::engine::kline_agg::latest_bar_time(&klines) {
        assert!((end - latest).num_days() <= 14, "daily should include recent bars");
    }
}

#[tokio::test]
async fn akshare_minute_kline_live() {
    let http = Client::builder()
        .user_agent("ThisIsMyQuant-E2E/0.1")
        .build()
        .unwrap();
    let client = AkshareClient::new(http);
    let end = Utc::now();
    let start = end - Duration::days(3);
    let klines = client
        .get_history("rb2510", "1m", start, end)
        .await
        .expect("minute klines");
    assert!(!klines.is_empty(), "expected minute klines from Sina");
}

#[tokio::test]
async fn akshare_latest_tick_live() {
    let http = Client::builder()
        .user_agent("ThisIsMyQuant-E2E/0.1")
        .build()
        .unwrap();
    let client = AkshareClient::new(http);
    let tick = client
        .fetch_latest_tick("rb2510")
        .await
        .expect("tick fetch")
        .expect("tick data");
    assert!(tick.last_price > 0.0);
    assert!(!tick.timestamp.is_empty());
}

#[tokio::test]
async fn jinshi_news_live() {
    let mut config = Config::load();
    config.jinshi_enabled = true;
    let http = Client::builder()
        .user_agent("ThisIsMyQuant-E2E/0.1")
        .build()
        .unwrap();
    let mut client = JinshiClient::new(http, &config);
    client.connect().await.expect("jinshi connect");
    assert!(client.is_connected(), "jinshi should connect");
    let _news = client.fetch_for_symbol("rb2510", 3).await.expect("news");
}

#[tokio::test]
async fn jinshi_calendar_live() {
    use app_lib::adapters::{default_calendar_range, fetch_calendar_range, CalendarFetchOptions};

    let config = Config::load();
    if !config.jinshi_enabled {
        eprintln!("SKIP: JINSHI_ENABLED=false");
        return;
    }
    let http = Client::builder()
        .user_agent("ThisIsMyQuant-E2E/0.1")
        .build()
        .unwrap();
    let (start, end) = default_calendar_range(3);
    match fetch_calendar_range(
        &http,
        &config,
        CalendarFetchOptions {
            start,
            end,
            min_star: 3,
            country: None,
        },
    )
    .await
    {
        Ok(events) if !events.is_empty() => {
            assert!(events.iter().any(|e| e.star >= 3));
            assert!(
                events.iter().any(|e| !e.country.is_empty()),
                "MCP events should have country parsed from title"
            );
            eprintln!("calendar live: {} events, sample={}", events.len(), events[0].name);
        }
        Ok(_) => eprintln!("SKIP: calendar empty (rili API may be unavailable)"),
        Err(e) => eprintln!("SKIP: calendar fetch failed: {e}"),
    }
}

#[tokio::test]
async fn reports_db_crud() {
    let db = temp_db();
    let report = AnalysisReport {
        id: "test-id".into(),
        symbol: "rb0".into(),
        trigger: "manual".into(),
        provider: "doubao".into(),
        prompt_version: "v2".into(),
        context_summary: "summary".into(),
        content: "content body".into(),
        created_at: Utc::now().to_rfc3339(),
        tags: vec![],
        dimension_summary: Some(serde_json::json!({"demand": ["测试要点"]})),
        news_ids: vec!["news-1".into()],
        anomaly_reason: None,
    };
    db.save_report(&report).expect("save report");
    let loaded = db.get_report("test-id").expect("get").expect("found");
    assert_eq!(loaded.content, "content body");
    assert_eq!(loaded.news_ids, vec!["news-1"]);
    assert!(loaded.dimension_summary.is_some());
    let list = db.get_reports(Some("rb0"), None, 10).expect("list");
    assert_eq!(list.len(), 1);
}

#[tokio::test]
async fn dimension_facts_roundtrip() {
    let db = temp_db();
    let report = AnalysisReport {
        id: "fact-rpt".into(),
        symbol: "rb0".into(),
        trigger: "manual".into(),
        provider: "doubao".into(),
        prompt_version: "v2".into(),
        context_summary: "summary".into(),
        content: "content".into(),
        created_at: Utc::now().to_rfc3339(),
        tags: vec![],
        dimension_summary: Some(serde_json::json!({
            "demand": ["地产需求偏弱"],
            "inventory": ["社会库存下降"]
        })),
        news_ids: vec![],
        anomaly_reason: None,
    };
    db.save_report(&report).expect("save report");
    let facts = facts_from_dimension_summary(
        &report.symbol,
        &report.id,
        report.dimension_summary.as_ref().unwrap(),
        &report.created_at,
    );
    assert_eq!(facts.len(), 2);
    db.replace_report_facts(&report.id, &facts)
        .expect("replace facts");
    let loaded = db.get_dimension_facts("rb0", 10).expect("get facts");
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0].symbol, "RB0");
}

#[tokio::test]
async fn followup_messages_roundtrip() {
    let db = temp_db();
    let msg = FollowupMessage {
        id: "fu-test".into(),
        report_id: "fact-rpt".into(),
        symbol: "RB0".into(),
        question: "库存为何下降？".into(),
        answer: "需求回暖叠加供给收缩。".into(),
        provider: "doubao".into(),
        created_at: Utc::now().to_rfc3339(),
    };
    db.save_followup(&msg).expect("save followup");
    let list = db
        .get_followups(Some("fact-rpt"), None, 10)
        .expect("list followups");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].question, "库存为何下降？");
}

#[tokio::test]
async fn llm_health_if_configured() {
    let state = bootstrap_test_state().await;
    let llm = state.llm_snapshot();
    if llm.available_providers().is_empty() {
        eprintln!("SKIP: no LLM API key configured");
        return;
    }
    let health = llm.health().await;
    assert!(!health.is_empty());
}
