build-release-wasm32:
    cargo build --release --bin ai-token-cost-estimator --target wasm32-unknown-unknown
    rm -rf out
    mkdir -p out
    wasm-bindgen --no-typescript --target web --out-dir ./out/ --out-name "ai_token_cost_estimator" ./target/wasm32-unknown-unknown/release/ai-token-cost-estimator.wasm
    wasm-opt --enable-bulk-memory --enable-threads --enable-nontrapping-float-to-int --enable-simd --enable-multivalue -O4 -ol 100 -s 100 -o out/output.wasm out/ai_token_cost_estimator_bg.wasm && mv out/output.wasm out/ai_token_cost_estimator_bg.wasm || echo "wasm-opt not found or failed, skipping optimization"
    cp index.html out/index.html
    cp _headers out/_headers
    cp favicon.png out/favicon.png

run-web-release:
    just build-release-wasm32
    static-web-server -p 3023 -d out
