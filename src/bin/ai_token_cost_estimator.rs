use ai_token_cost_estimator::app;

#[cfg(not(target_family = "wasm"))]
fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1320.0, 880.0])
            .with_min_inner_size([900.0, 600.0])
            .with_title("AI Token & Cost Estimator"),
        ..Default::default()
    };

    eframe::run_native(
        "AI Token & Cost Estimator",
        options,
        Box::new(|cc| Ok(Box::new(app::TokenizerApp::new(cc)))),
    )
}

#[cfg(target_family = "wasm")]
pub fn main() -> Result<(), wasm_bindgen::JsValue> {
    use wasm_bindgen::JsCast as _;

    // Redirect tracing/log to console.log
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .and_then(|win| win.document())
            .expect("Failed to get window/document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .or_else(|| document.query_selector("canvas").ok().flatten())
            .expect("Failed to find canvas element")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("Failed to cast element to HtmlCanvasElement");

        eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(app::TokenizerApp::new(cc)))),
            )
            .await
            .expect("failed to start eframe");
    });

    Ok(())
}
