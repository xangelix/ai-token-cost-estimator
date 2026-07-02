//! Tokenizer loading (from URL or local path) and tokenization logic.

use std::{fmt::Write as _, sync::Arc};

use tokenizers::Tokenizer;

// ---------------------------------------------------------------------------
// Chat types
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum ChatRole {
    System,
    User,
    Assistant,
}

impl ChatRole {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::User => "user",
            Self::Assistant => "assistant",
        }
    }

    pub const fn variants() -> [Self; 3] {
        [Self::System, Self::User, Self::Assistant]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

// ---------------------------------------------------------------------------
// Token segment
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct TokenSegment {
    pub id: u32,
    pub text: String,
    pub token: String,
    pub start: usize,
    pub end: usize,
}

// ---------------------------------------------------------------------------
// Tokenizer manager – handles async loading
// ---------------------------------------------------------------------------

#[derive(Clone, Default)]
pub enum TokenizerState {
    #[default]
    Idle,
    Loading,
    Loaded(Arc<Tokenizer>),
    Error(String),
}

pub struct TokenizerManager {
    pub(crate) state: TokenizerState,
    current_name: Option<String>,
    pub loader: egui_async::Bind<Arc<Tokenizer>, String>,
}

impl Default for TokenizerManager {
    fn default() -> Self {
        Self {
            state: TokenizerState::Idle,
            current_name: None,
            loader: egui_async::Bind::new(true),
        }
    }
}

impl TokenizerManager {
    /// Load a tokenizer from a URL or local file path.
    pub fn load(&mut self, source: &str, name: &str, ctx: &egui::Context) {
        self.state = TokenizerState::Loading;
        self.current_name = Some(name.to_string());

        if source.starts_with("http://") || source.starts_with("https://") {
            self.load_from_url(source, ctx);
        } else {
            self.load_from_file(source);
        }
    }

    fn load_from_url(&mut self, url: &str, _ctx: &egui::Context) {
        let url = url.to_string();
        self.loader.request(async move {
            let response = reqwest::get(&url).await.map_err(|e| e.to_string())?;
            let status = response.status();
            let bytes = response.bytes().await.map_err(|e| e.to_string())?;
            if status.is_success() {
                let tokenizer = Tokenizer::from_bytes(&bytes)
                    .map_err(|e| format!("Failed to parse tokenizer JSON: {e}"))?;
                Ok(Arc::new(tokenizer))
            } else {
                Err(format!(
                    "HTTP {}: {}",
                    status.as_u16(),
                    String::from_utf8_lossy(&bytes)
                ))
            }
        });
    }

    fn load_from_file(&mut self, path: &str) {
        match std::fs::read(path) {
            Ok(bytes) => match Tokenizer::from_bytes(&bytes) {
                Ok(tokenizer) => {
                    let arc_tokenizer = Arc::new(tokenizer);
                    self.loader.fill(Ok(arc_tokenizer.clone()));
                    self.state = TokenizerState::Loaded(arc_tokenizer);
                }
                Err(e) => {
                    let err_msg = format!("Failed to parse tokenizer JSON: {e}");
                    self.loader.fill(Err(err_msg.clone()));
                    self.state = TokenizerState::Error(err_msg);
                }
            },
            Err(e) => {
                let err_msg = format!("Failed to read file: {e}");
                self.loader.fill(Err(err_msg.clone()));
                self.state = TokenizerState::Error(err_msg);
            }
        }
    }

    /// Called every frame on the main thread to check for completed loads.
    pub fn poll(&mut self) {
        self.loader.poll();
        match self.loader.state() {
            egui_async::StateWithData::Idle => {
                if !matches!(
                    self.state,
                    TokenizerState::Loaded(_) | TokenizerState::Error(_)
                ) {
                    self.state = TokenizerState::Idle;
                }
            }
            egui_async::StateWithData::Pending => {
                self.state = TokenizerState::Loading;
            }
            egui_async::StateWithData::Finished(t) => {
                self.state = TokenizerState::Loaded(t.clone());
            }
            egui_async::StateWithData::Failed(e) => {
                self.state = TokenizerState::Error(e.clone());
            }
        }
    }

    pub const fn is_loaded(&self) -> bool {
        matches!(self.state, TokenizerState::Loaded(_))
    }

    pub const fn is_loading(&self) -> bool {
        matches!(self.state, TokenizerState::Loading)
    }

    pub const fn get_tokenizer(&self) -> Option<&Arc<Tokenizer>> {
        match &self.state {
            TokenizerState::Loaded(t) => Some(t),
            _ => None,
        }
    }

    pub fn get_error(&self) -> Option<&str> {
        match &self.state {
            TokenizerState::Error(e) => Some(e),
            _ => None,
        }
    }

    pub fn current_name(&self) -> Option<&str> {
        self.current_name.as_deref()
    }
}

// ---------------------------------------------------------------------------
// Tokenization helpers
// ---------------------------------------------------------------------------

pub fn tokenize(tokenizer: &Tokenizer, text: &str) -> Vec<TokenSegment> {
    match tokenizer.encode(text, true) {
        Ok(encoding) => {
            let ids = encoding.get_ids();
            let offsets = encoding.get_offsets();
            let tokens = encoding.get_tokens();

            ids.iter()
                .zip(offsets.iter())
                .zip(tokens.iter())
                .map(|((id, (start, end)), token)| {
                    let display_text =
                        if *start <= text.len() && *end <= text.len() && *start <= *end {
                            text[*start..*end].to_string()
                        } else {
                            token.clone()
                        };
                    TokenSegment {
                        id: *id,
                        text: display_text,
                        token: token.clone(),
                        start: *start,
                        end: *end,
                    }
                })
                .collect()
        }
        Err(e) => {
            eprintln!("Tokenization error: {e}");
            Vec::new()
        }
    }
}

/// Tokenize a chat conversation using a simple ChatML-style format.
/// This is approximate — different models use different chat templates.
pub fn tokenize_chat(tokenizer: &Tokenizer, messages: &[ChatMessage]) -> Vec<TokenSegment> {
    let mut chat_text = String::new();
    for msg in messages {
        let _ = write!(
            chat_text,
            "<|im_start|>{}\n{}<|im_end|>\n",
            msg.role.as_str(),
            msg.content
        );
    }
    tokenize(tokenizer, &chat_text)
}

pub fn count_tokens(tokenizer: &Tokenizer, text: &str) -> usize {
    tokenizer
        .encode(text, true)
        .map_or(0, |encoding| encoding.get_ids().len())
}
