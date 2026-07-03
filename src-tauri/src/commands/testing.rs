use std::sync::Arc;

use tauri::State;

use crate::models::ApiResponse;
use crate::state::AppState;

#[tauri::command]
pub async fn run_client_e2e(
    state: State<'_, Arc<AppState>>,
    symbol: Option<String>,
) -> Result<ApiResponse<crate::testing::E2eSuiteReport>, String> {
    #[cfg(not(debug_assertions))]
    {
        let _ = (state, symbol);
        return Ok(ApiResponse::err("client e2e 仅 debug 构建可用"));
    }
    #[cfg(debug_assertions)]
    {
        let sym = symbol.unwrap_or_else(|| "rb0".into());
        let symbols = vec![
            "rb0".into(),
            "au0".into(),
            "m0".into(),
            "sc0".into(),
            "ec0".into(),
        ];
        Ok(ApiResponse::ok(
            crate::testing::run_client_e2e_suite(state.inner(), &sym, &symbols).await,
        ))
    }
}
