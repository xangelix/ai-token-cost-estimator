//! Main application state and `eframe::App` implementation.

use std::time::Duration;

use egui::Context;
use tokenizers::Tokenizer;
use tokio_with_wasm::alias as tokio;

use crate::models::ModelRegistry;
use crate::theme::Theme;
use crate::tokenizer::{ChatMessage, ChatRole, TokenSegment, TokenizerManager};

const DEFAULT_PROMPT: &str = "Tokenization turns text into small pieces called tokens. \
Models read those tokens to understand and predict what comes next.\n\n\
Try editing this text, switching models, or loading any tokenizer.json from a URL!";

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Tab {
    Prompt,
    Chat,
}

pub struct TokenizerApp {
    pub theme: Theme,
    pub tokenizer_manager: TokenizerManager,
    pub model_registry: ModelRegistry,
    pub selected_model: usize,
    pub last_selected_model: usize,
    pub prompt: String,
    pub chat_messages: Vec<ChatMessage>,
    pub show_token_ids: bool,
    pub active_tab: Tab,
    pub custom_url: String,
    pub hovered_token: Option<usize>,
    pub last_segments: Vec<TokenSegment>,
    pub last_chat_segments: Vec<TokenSegment>,
    pub is_custom: bool,
    pub chat_message_token_counts: Vec<usize>,
    pub prompt_tok_bind: egui_async::Bind<Vec<TokenSegment>, ()>,
    pub chat_tok_bind: egui_async::Bind<(Vec<TokenSegment>, Vec<usize>), ()>,
    cached_prompt_tokenizer: Option<std::sync::Arc<Tokenizer>>,
    cached_prompt_text: Option<String>,
    cached_chat_tokenizer: Option<std::sync::Arc<Tokenizer>>,
    cached_chat_messages: Option<Vec<ChatMessage>>,
}

impl TokenizerApp {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        let mut app = Self {
            theme: Theme::default(),
            tokenizer_manager: TokenizerManager::default(),
            model_registry: ModelRegistry::default(),
            selected_model: 0,
            last_selected_model: 0,
            prompt: DEFAULT_PROMPT.to_string(),
            chat_messages: vec![
                ChatMessage {
                    role: ChatRole::System,
                    content: "You are an enthusiastic educator that loves to visualise how tokenization works.".to_string(),
                },
                ChatMessage {
                    role: ChatRole::User,
                    content: "Explain how tokenizers estimate token costs in two concise bullet points.".to_string(),
                },
            ],
            show_token_ids: false,
            active_tab: Tab::Prompt,
            custom_url: String::new(),
            hovered_token: None,
            last_segments: Vec::new(),
            last_chat_segments: Vec::new(),
            is_custom: false,
            chat_message_token_counts: Vec::new(),
            prompt_tok_bind: egui_async::Bind::new(true),
            chat_tok_bind: egui_async::Bind::new(true),
            cached_prompt_tokenizer: None,
            cached_prompt_text: None,
            cached_chat_tokenizer: None,
            cached_chat_messages: None,
        };
        // Auto-load the first model
        app.load_selected_model(&app_first_ctx());
        app
    }

    pub fn load_selected_model(&mut self, ctx: &Context) {
        if let Some(model) = self.model_registry.get(self.selected_model) {
            self.tokenizer_manager.load(model.url, model.name, ctx);
            self.is_custom = false;
        }
    }

    pub fn load_custom_url(&mut self, ctx: &Context) {
        let url = self.custom_url.trim();
        if !url.is_empty() {
            let name = if url.starts_with("http") {
                url.split('/').nth(3).unwrap_or("Custom")
            } else {
                "Local file"
            };
            self.tokenizer_manager.load(url, name, ctx);
            self.is_custom = true;
        }
    }

    pub const fn is_tokenizer_ready(&self) -> bool {
        self.tokenizer_manager.is_loaded()
    }

    pub const fn is_loading(&self) -> bool {
        self.tokenizer_manager.is_loading()
    }

    pub const fn get_tokenizer(&self) -> Option<&std::sync::Arc<Tokenizer>> {
        self.tokenizer_manager.get_tokenizer()
    }

    pub fn get_error(&self) -> Option<&str> {
        self.tokenizer_manager.get_error()
    }

    fn update_tokenization(&mut self) {
        // First, check for prompt tokenization async results
        self.prompt_tok_bind.poll();
        if let Some(res) = self.prompt_tok_bind.ok_ref() {
            self.last_segments = res.clone();
        }

        // Check for chat tokenization async results
        self.chat_tok_bind.poll();
        if let Some((segments, counts)) = self.chat_tok_bind.ok_ref() {
            self.last_chat_segments = segments.clone();
            self.chat_message_token_counts = counts.clone();
        }

        if let Some(tokenizer) = self.get_tokenizer().cloned() {
            match self.active_tab {
                Tab::Prompt => {
                    if self.prompt.is_empty() {
                        self.last_segments.clear();
                        self.prompt_tok_bind.clear();
                        self.cached_prompt_text = Some(self.prompt.clone());
                        self.cached_prompt_tokenizer = Some(tokenizer.clone());
                    } else {
                        let needs_retokenize =
                            match (&self.cached_prompt_tokenizer, &self.cached_prompt_text) {
                                (Some(cached_tok), Some(cached_text)) => {
                                    !std::sync::Arc::ptr_eq(cached_tok, &tokenizer)
                                        || cached_text != &self.prompt
                                }
                                _ => true,
                            };
                        if needs_retokenize {
                            self.cached_prompt_tokenizer = Some(tokenizer.clone());
                            self.cached_prompt_text = Some(self.prompt.clone());

                            let tokenizer = tokenizer.clone();
                            let prompt = self.prompt.clone();
                            self.prompt_tok_bind.request(async move {
                                let gg = super::db::get_module_prices_and_context_window()
                                    .await
                                    .unwrap();
                                println!("{:#?}", gg["sample_spec"]);
                                let res = tokio::task::spawn_blocking(move || {
                                    crate::tokenizer::tokenize(&tokenizer, &prompt)
                                })
                                .await
                                .unwrap_or_default();
                                Ok(res)
                            });
                        }
                    }
                }
                Tab::Chat => {
                    let needs_retokenize =
                        match (&self.cached_chat_tokenizer, &self.cached_chat_messages) {
                            (Some(cached_tok), Some(cached_msg)) => {
                                !std::sync::Arc::ptr_eq(cached_tok, &tokenizer)
                                    || cached_msg != &self.chat_messages
                            }
                            _ => true,
                        };
                    if needs_retokenize {
                        let messages: Vec<ChatMessage> = self
                            .chat_messages
                            .iter()
                            .filter(|m| !m.content.trim().is_empty())
                            .map(|m| ChatMessage {
                                role: m.role,
                                content: m.content.clone(),
                            })
                            .collect();

                        self.cached_chat_tokenizer = Some(tokenizer.clone());
                        self.cached_chat_messages = Some(self.chat_messages.clone());

                        if messages.is_empty() {
                            self.last_chat_segments.clear();
                            self.chat_message_token_counts =
                                self.chat_messages.iter().map(|_| 0).collect();
                            self.chat_tok_bind.clear();
                        } else {
                            let tokenizer = tokenizer.clone();
                            let messages_clone = messages.clone();
                            let all_chat_messages = self.chat_messages.clone();

                            self.chat_tok_bind.request(async move {
                                let res = tokio::task::spawn_blocking(move || {
                                    let last_chat_segments = crate::tokenizer::tokenize_chat(
                                        &tokenizer,
                                        &messages_clone,
                                    );
                                    let chat_message_token_counts = all_chat_messages
                                        .iter()
                                        .map(|m| {
                                            if m.content.trim().is_empty() {
                                                0
                                            } else {
                                                crate::tokenizer::count_tokens(
                                                    &tokenizer, &m.content,
                                                )
                                            }
                                        })
                                        .collect();
                                    (last_chat_segments, chat_message_token_counts)
                                })
                                .await
                                .unwrap_or_else(|_| (Vec::new(), Vec::new()));
                                Ok(res)
                            });
                        }
                    }
                }
            }
        } else {
            if self.cached_prompt_tokenizer.is_some() || self.cached_prompt_text.is_some() {
                self.last_segments.clear();
                self.prompt_tok_bind.clear();
                self.cached_prompt_tokenizer = None;
                self.cached_prompt_text = None;
            }
            if self.cached_chat_tokenizer.is_some() || self.cached_chat_messages.is_some() {
                self.last_chat_segments.clear();
                self.chat_message_token_counts.clear();
                self.chat_tok_bind.clear();
                self.cached_chat_tokenizer = None;
                self.cached_chat_messages = None;
            }
        }
    }
}

/// Helper to get a context for the initial load.
fn app_first_ctx() -> Context {
    egui::Context::default()
}

impl eframe::App for TokenizerApp {
    fn logic(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        ctx.plugin_or_default::<egui_async::EguiAsyncPlugin>();

        self.theme.apply(ctx);
        self.tokenizer_manager.poll();

        // Trigger repaint while loading for spinner animation
        if self.is_loading() {
            ctx.request_repaint_after(Duration::from_millis(100));
        }

        // Auto-reload when model selection changes
        if self.last_selected_model != self.selected_model {
            self.last_selected_model = self.selected_model;
            self.load_selected_model(ctx);
        }

        // Re-tokenize
        self.update_tokenization();
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        crate::ui::render(self, ui);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokenizers::Tokenizer;

    fn advance_frame() {
        static FRAME_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let frame = FRAME_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed) as f64;
        egui_async::bind::CURR_FRAME.store(frame, std::sync::atomic::Ordering::Relaxed);
    }

    fn make_test_tokenizer() -> Arc<Tokenizer> {
        let json = r#"{
  "version": "1.0",
  "truncation": null,
  "padding": null,
  "added_tokens": [],
  "normalizer": null,
  "pre_tokenizer": null,
  "post_processor": null,
  "decoder": null,
  "model": {
    "type": "WordLevel",
    "vocab": {
      "[UNK]": 0,
      "hello": 1,
      "world": 2
    },
    "unk_token": "[UNK]"
  }
}"#;
        Arc::new(Tokenizer::from_bytes(json.as_bytes()).unwrap())
    }

    #[tokio::test]
    async fn test_prompt_tokenization_caching() {
        let tokenizer1 = make_test_tokenizer();
        let tokenizer2 = make_test_tokenizer();

        let mut app = TokenizerApp {
            theme: Theme::default(),
            tokenizer_manager: TokenizerManager::default(),
            model_registry: ModelRegistry::default(),
            selected_model: 0,
            last_selected_model: 0,
            prompt: "hello world".to_string(),
            chat_messages: vec![],
            show_token_ids: false,
            active_tab: Tab::Prompt,
            custom_url: String::new(),
            hovered_token: None,
            last_segments: Vec::new(),
            last_chat_segments: Vec::new(),
            is_custom: false,
            chat_message_token_counts: Vec::new(),
            prompt_tok_bind: egui_async::Bind::new(true),
            chat_tok_bind: egui_async::Bind::new(true),
            cached_prompt_tokenizer: None,
            cached_prompt_text: None,
            cached_chat_tokenizer: None,
            cached_chat_messages: None,
        };

        // Initially no tokenizer is loaded, so update_tokenization should clear segments and not start tasks
        advance_frame();
        app.update_tokenization();
        assert!(app.last_segments.is_empty());
        assert_eq!(app.prompt_tok_bind.count_executed(), 0);

        // Set tokenizer1
        app.tokenizer_manager.state = crate::tokenizer::TokenizerState::Loaded(tokenizer1.clone());

        // First run: should trigger tokenization
        advance_frame();
        app.update_tokenization();
        assert_eq!(app.prompt_tok_bind.count_executed(), 1);

        while {
            advance_frame();
            app.prompt_tok_bind.is_pending()
        } {
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
        advance_frame();
        app.update_tokenization();
        assert!(!app.last_segments.is_empty());

        // Second run: should hit cache and NOT trigger a new request
        advance_frame();
        app.update_tokenization();
        assert_eq!(
            app.prompt_tok_bind.count_executed(),
            1,
            "Cache missed unexpectedly!"
        );

        // Third run: change prompt text. Should trigger tokenization
        app.prompt = "hello".to_string();
        advance_frame();
        app.update_tokenization();
        assert_eq!(
            app.prompt_tok_bind.count_executed(),
            2,
            "Cache hit when prompt changed!"
        );

        while {
            advance_frame();
            app.prompt_tok_bind.is_pending()
        } {
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
        advance_frame();
        app.update_tokenization();
        assert!(!app.last_segments.is_empty());

        // Fourth run: change tokenizer. Should trigger tokenization
        app.tokenizer_manager.state = crate::tokenizer::TokenizerState::Loaded(tokenizer2.clone());
        advance_frame();
        app.update_tokenization();
        assert_eq!(
            app.prompt_tok_bind.count_executed(),
            3,
            "Cache hit when tokenizer changed!"
        );
    }

    #[tokio::test]
    async fn test_chat_tokenization_caching() {
        let tokenizer = make_test_tokenizer();

        let mut app = TokenizerApp {
            theme: Theme::default(),
            tokenizer_manager: TokenizerManager::default(),
            model_registry: ModelRegistry::default(),
            selected_model: 0,
            last_selected_model: 0,
            prompt: String::new(),
            chat_messages: vec![ChatMessage {
                role: ChatRole::User,
                content: "hello".to_string(),
            }],
            show_token_ids: false,
            active_tab: Tab::Chat,
            custom_url: String::new(),
            hovered_token: None,
            last_segments: Vec::new(),
            last_chat_segments: Vec::new(),
            is_custom: false,
            chat_message_token_counts: Vec::new(),
            prompt_tok_bind: egui_async::Bind::new(true),
            chat_tok_bind: egui_async::Bind::new(true),
            cached_prompt_tokenizer: None,
            cached_prompt_text: None,
            cached_chat_tokenizer: None,
            cached_chat_messages: None,
        };

        // Set tokenizer
        app.tokenizer_manager.state = crate::tokenizer::TokenizerState::Loaded(tokenizer.clone());

        // First run: should trigger tokenization
        advance_frame();
        app.update_tokenization();
        assert_eq!(app.chat_tok_bind.count_executed(), 1);

        while {
            advance_frame();
            app.chat_tok_bind.is_pending()
        } {
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
        advance_frame();
        app.update_tokenization();
        assert!(!app.last_chat_segments.is_empty());
        assert_eq!(app.chat_message_token_counts.len(), 1);
        assert_eq!(app.chat_message_token_counts[0], 1);

        // Second run: should hit cache
        advance_frame();
        app.update_tokenization();
        assert_eq!(
            app.chat_tok_bind.count_executed(),
            1,
            "Chat cache missed unexpectedly!"
        );

        // Third run: change chat message content
        app.chat_messages[0].content = "world".to_string();
        advance_frame();
        app.update_tokenization();
        assert_eq!(
            app.chat_tok_bind.count_executed(),
            2,
            "Chat cache hit when messages changed!"
        );

        while {
            advance_frame();
            app.chat_tok_bind.is_pending()
        } {
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
        advance_frame();
        app.update_tokenization();
        assert!(!app.last_chat_segments.is_empty());
        assert_eq!(app.chat_message_token_counts[0], 1);
    }
}
