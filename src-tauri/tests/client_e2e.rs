//! 客户端 Live E2E：启动真实 AppState，验证各业务模块与 LLM 明日/短期分析。

use std::sync::Arc;

use app_lib::testing::{bootstrap_test_state, run_client_e2e_suite};

fn require_llm() -> bool {
    std::env::var("E2E_SKIP_LLM")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

#[tokio::test]
async fn client_e2e_all_modules_and_llm_analysis() {
    let state: Arc<_> = bootstrap_test_state().await;
    let providers = state.llm_snapshot().available_providers();
    if providers.is_empty() {
        eprintln!("SKIP: no LLM provider — run bash scripts/sync-env.sh first");
        return;
    }

    let symbols = ["rb0", "au0", "m0", "sc0", "ec0"]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    let report = run_client_e2e_suite(&state, "rb0", &symbols).await;
    eprintln!("E2E modules:");
    for m in &report.modules {
        eprintln!(
            "  [{}] {} — {}",
            if m.ok { "OK" } else { "FAIL" },
            m.module,
            m.message
        );
    }
    for a in &report.analyses {
        eprintln!(
            "  analysis {} report={} len={}",
            a.trigger, a.report_id, a.content_len
        );
    }
    for s in &report.symbol_checks {
        eprintln!(
            "  symbol {} sector={} bars={} context_bars={}",
            s.symbol, s.sector, s.bars, s.context_bars
        );
    }

    for m in &report.modules {
        if m.module.starts_with("analysis_") && require_llm() {
            continue;
        }
        assert!(m.ok, "module {} failed: {}", m.module, m.message);
    }
    assert!(report.ok, "client e2e suite failed");
    assert_eq!(
        report.analyses.len(),
        2,
        "expected tomorrow + short_term analyses"
    );
    assert_eq!(
        report.symbol_checks.len(),
        5,
        "expected five sector symbol checks"
    );
    assert!(report.symbol_checks.iter().all(|s| s.ok));
}
