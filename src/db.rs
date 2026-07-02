use std::sync::LazyLock;

use include_packed::include_packed;

static MODEL_PRICES_AND_CONTEXT_WINDOW: LazyLock<Vec<u8>> =
    LazyLock::new(|| include_packed!("assets/db/model_prices_and_context_window.json"));

const SOURCE: &str = "https://raw.githubusercontent.com/BerriAI/litellm/refs/heads/litellm_internal_staging/model_prices_and_context_window.json";

pub async fn get_module_prices_and_context_window_online() -> reqwest::Result<serde_json::Value> {
    reqwest::get(SOURCE).await?.json().await
}

pub async fn get_module_prices_and_context_window() -> anyhow::Result<serde_json::Value> {
    match get_module_prices_and_context_window_online().await {
        Ok(val) => Ok(val),
        Err(_) => serde_json::from_slice(&MODEL_PRICES_AND_CONTEXT_WINDOW).map_err(Into::into),
    }
}
