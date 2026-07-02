//! Predefined model registry with metadata and tokenizer URLs.

pub struct PricingPerM {
    pub input_per: f64,
    pub output_per: f64,
    pub cached_input_per: Option<f64>,
}

pub struct ModelEntry {
    pub name: &'static str,
    pub url: &'static str,
    pub context_window: Option<usize>,
    pub max_output_tokens: Option<usize>,
    pub description: &'static str,
    pub pricing: Option<PricingPerM>,
    pub modalities: &'static [&'static str],
}

pub struct ModelRegistry {
    models: Vec<ModelEntry>,
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self {
            models: vec![
                // ── OpenAI ────────────────────────────────────────────────
                ModelEntry {
                    name: "gpt-4o",
                    url: "https://huggingface.co/Xenova/gpt-4o/resolve/main/tokenizer.json",
                    context_window: Some(128_000),
                    max_output_tokens: Some(16_384),
                    description: "OpenAI GPT-4o multimodal flagship",
                    pricing: Some(PricingPerM {
                        input_per: 2.50,
                        output_per: 10.00,
                        cached_input_per: Some(1.25),
                    }),
                    modalities: &["text", "image", "audio"],
                },
                ModelEntry {
                    name: "gpt-4",
                    url: "https://huggingface.co/Xenova/gpt-4/resolve/main/tokenizer.json",
                    context_window: Some(8_192),
                    max_output_tokens: Some(4_096),
                    description: "OpenAI GPT-4 (cl100k_base)",
                    pricing: Some(PricingPerM {
                        input_per: 30.00,
                        output_per: 60.00,
                        cached_input_per: None,
                    }),
                    modalities: &["text", "image"],
                },
                ModelEntry {
                    name: "gpt-3.5-turbo",
                    url: "https://huggingface.co/Xenova/gpt-3.5-turbo/resolve/main/tokenizer.json",
                    context_window: Some(16_385),
                    max_output_tokens: Some(4_096),
                    description: "OpenAI GPT-3.5 Turbo (cl100k_base)",
                    pricing: Some(PricingPerM {
                        input_per: 0.50,
                        output_per: 1.50,
                        cached_input_per: Some(0.25),
                    }),
                    modalities: &["text"],
                },
                ModelEntry {
                    name: "gpt-2",
                    url: "https://huggingface.co/openai-community/gpt2/resolve/main/tokenizer.json",
                    context_window: Some(1_024),
                    max_output_tokens: Some(1_024),
                    description: "OpenAI GPT-2 (original BPE)",
                    pricing: None,
                    modalities: &["text"],
                },
                ModelEntry {
                    name: "o200k_base",
                    url: "https://huggingface.co/Xenova/o200k_base/resolve/main/tokenizer.json",
                    context_window: Some(128_000),
                    max_output_tokens: None,
                    description: "Base tokenizer for GPT-4o / o-series",
                    pricing: None,
                    modalities: &["text"],
                },
                ModelEntry {
                    name: "cl100k_base",
                    url: "https://huggingface.co/Xenova/cl100k_base/resolve/main/tokenizer.json",
                    context_window: Some(8_192),
                    max_output_tokens: None,
                    description: "Base tokenizer for GPT-3.5 / GPT-4",
                    pricing: None,
                    modalities: &["text"],
                },
                // ── Meta Llama ────────────────────────────────────────────
                ModelEntry {
                    name: "llama-3.1-8b",
                    url: "https://huggingface.co/NousResearch/Meta-Llama-3.1-8B/resolve/main/tokenizer.json",
                    context_window: Some(128_000),
                    max_output_tokens: Some(8_192),
                    description: "Meta Llama 3.1 8B Instruct",
                    pricing: None,
                    modalities: &["text"],
                },
                ModelEntry {
                    name: "llama-3.2-1b",
                    url: "https://huggingface.co/NousResearch/Llama-3.2-1B/resolve/main/tokenizer.json",
                    context_window: Some(128_000),
                    max_output_tokens: Some(8_192),
                    description: "Meta Llama 3.2 1B (edge)",
                    pricing: None,
                    modalities: &["text"],
                },
                // ── Mistral ───────────────────────────────────────────────
                ModelEntry {
                    name: "mistral-7b-v0.3",
                    url: "https://huggingface.co/mistralai/Mistral-7B-v0.3/resolve/main/tokenizer.json",
                    context_window: Some(32_768),
                    max_output_tokens: None,
                    description: "Mistral 7B v0.3 base model",
                    pricing: None,
                    modalities: &["text"],
                },
                ModelEntry {
                    name: "mistral-nemo",
                    url: "https://huggingface.co/mistralai/Mistral-Nemo-Base-2407/resolve/main/tokenizer.json",
                    context_window: Some(128_000),
                    max_output_tokens: None,
                    description: "Mistral NeMo 12B (Tekken tokenizer)",
                    pricing: None,
                    modalities: &["text"],
                },
                // ── Google Gemma ──────────────────────────────────────────
                ModelEntry {
                    name: "gemma-2-2b",
                    url: "https://huggingface.co/google/gemma-2-2b/resolve/main/tokenizer.json",
                    context_window: Some(8_192),
                    max_output_tokens: None,
                    description: "Google Gemma 2 2B",
                    pricing: None,
                    modalities: &["text"],
                },
                ModelEntry {
                    name: "gemma-2-9b",
                    url: "https://huggingface.co/google/gemma-2-9b/resolve/main/tokenizer.json",
                    context_window: Some(8_192),
                    max_output_tokens: None,
                    description: "Google Gemma 2 9B",
                    pricing: None,
                    modalities: &["text"],
                },
                // ── Microsoft Phi ─────────────────────────────────────────
                ModelEntry {
                    name: "phi-3-mini",
                    url: "https://huggingface.co/microsoft/Phi-3-mini-4k-instruct/resolve/main/tokenizer.json",
                    context_window: Some(4_096),
                    max_output_tokens: None,
                    description: "Microsoft Phi-3 Mini 4K Instruct",
                    pricing: None,
                    modalities: &["text"],
                },
                ModelEntry {
                    name: "phi-3.5-mini",
                    url: "https://huggingface.co/microsoft/Phi-3.5-mini-instruct/resolve/main/tokenizer.json",
                    context_window: Some(128_000),
                    max_output_tokens: None,
                    description: "Microsoft Phi-3.5 Mini Instruct",
                    pricing: None,
                    modalities: &["text"],
                },
                // ── Alibaba Qwen ──────────────────────────────────────────
                ModelEntry {
                    name: "qwen2.5-7b",
                    url: "https://huggingface.co/Qwen/Qwen2.5-7B/resolve/main/tokenizer.json",
                    context_window: Some(131_072),
                    max_output_tokens: Some(8_192),
                    description: "Qwen 2.5 7B",
                    pricing: None,
                    modalities: &["text"],
                },
                ModelEntry {
                    name: "qwen2.5-coder-7b",
                    url: "https://huggingface.co/Qwen/Qwen2.5-Coder-7B/resolve/main/tokenizer.json",
                    context_window: Some(131_072),
                    max_output_tokens: Some(8_192),
                    description: "Qwen 2.5 Coder 7B",
                    pricing: None,
                    modalities: &["text"],
                },
                // ── Code models ───────────────────────────────────────────
                ModelEntry {
                    name: "starcoder2-3b",
                    url: "https://huggingface.co/bigcode/starcoder2-3b/resolve/main/tokenizer.json",
                    context_window: Some(16_384),
                    max_output_tokens: None,
                    description: "BigCode StarCoder2 3B",
                    pricing: None,
                    modalities: &["text"],
                },
                ModelEntry {
                    name: "deepseek-coder-6.7b",
                    url: "https://huggingface.co/deepseek-ai/deepseek-coder-6.7b-base/resolve/main/tokenizer.json",
                    context_window: Some(16_000),
                    max_output_tokens: None,
                    description: "DeepSeek Coder 6.7B Base",
                    pricing: None,
                    modalities: &["text"],
                },
                // ── Cohere ────────────────────────────────────────────────
                ModelEntry {
                    name: "command-r-plus",
                    url: "https://huggingface.co/CohereForAI/c4ai-command-r-plus/resolve/main/tokenizer.json",
                    context_window: Some(128_000),
                    max_output_tokens: None,
                    description: "Cohere Command R+",
                    pricing: None,
                    modalities: &["text"],
                },
                // ── Classic NLP ───────────────────────────────────────────
                ModelEntry {
                    name: "roberta-base",
                    url: "https://huggingface.co/FacebookAI/roberta-base/resolve/main/tokenizer.json",
                    context_window: Some(512),
                    max_output_tokens: None,
                    description: "RoBERTa base (BERT-style)",
                    pricing: None,
                    modalities: &["text"],
                },
                ModelEntry {
                    name: "t5-base",
                    url: "https://huggingface.co/google-t5/t5-base/resolve/main/tokenizer.json",
                    context_window: Some(512),
                    max_output_tokens: None,
                    description: "Google T5 base (SentencePiece)",
                    pricing: None,
                    modalities: &["text"],
                },
            ],
        }
    }
}

impl ModelRegistry {
    pub fn get(&self, index: usize) -> Option<&ModelEntry> {
        self.models.get(index)
    }

    pub fn iter(&self) -> impl Iterator<Item = &ModelEntry> {
        self.models.iter()
    }
}
