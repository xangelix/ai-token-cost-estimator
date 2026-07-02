//! All UI rendering for the tokenizer playground.

use std::fmt::Write as _;

use egui::{
    Align, Color32, FontFamily, Frame, Layout, Margin, RichText, ScrollArea, Sense, Stroke, Ui,
    Vec2,
};

use crate::app::{Tab, TokenizerApp};
use crate::colors;
use crate::models::ModelEntry;
use crate::tokenizer::{ChatMessage, ChatRole, TokenSegment};

// ---------------------------------------------------------------------------
// Top-level render
// ---------------------------------------------------------------------------

pub fn render(app: &mut TokenizerApp, ui: &mut Ui) {
    // ── Top bar ──────────────────────────────────────────────────────
    egui::Panel::top("top_bar").show(ui, |ui| {
        render_top_bar(app, ui);
    });

    // ── Footer ───────────────────────────────────────────────────────
    egui::Panel::bottom("footer").show(ui, |ui| {
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("AI Token & Cost Estimator — load any HuggingFace tokenizer.json from a URL or local path")
                    .small(),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(RichText::new("Powered by egui + tokenizers + egui-async + reqwest").small());
            });
        });
        ui.add_space(2.0);
    });

    // ── Right sidebar (stats / model info) ───────────────────────────
    egui::Panel::right("stats_panel")
        .resizable(true)
        .default_size(340.0)
        .min_size(260.0)
        .show(ui, |ui| match app.active_tab {
            Tab::Prompt => render_stats_panel(app, ui),
            Tab::Chat => render_chat_stats(app, ui),
        });

    // ── Central content ──────────────────────────────────────────────
    egui::CentralPanel::default().show(ui, |ui| match app.active_tab {
        Tab::Prompt => render_prompt_view(app, ui),
        Tab::Chat => render_chat_view(app, ui),
    });
}

// ---------------------------------------------------------------------------
// Top bar
// ---------------------------------------------------------------------------

fn render_top_bar(app: &mut TokenizerApp, ui: &mut Ui) {
    ui.add_space(4.0);

    // Row 1: title + model selector + URL loader
    ui.horizontal(|ui| {
        ui.heading("AI Token & Cost Estimator");
        ui.separator();

        // Model selector
        ui.label("Model:");
        let selected_text = app
            .model_registry
            .get(app.selected_model)
            .map_or("Select…", |m| m.name);

        egui::ComboBox::from_id_salt("model_selector")
            .selected_text(selected_text)
            .show_ui(ui, |ui| {
                for (i, model) in app.model_registry.iter().enumerate() {
                    ui.selectable_value(&mut app.selected_model, i, model.name);
                }
            });

        if ui.button("↻ Load").clicked() {
            app.load_selected_model(ui.ctx());
        }

        ui.separator();

        // Custom URL
        ui.label("URL / path:");
        ui.add(
            egui::TextEdit::singleline(&mut app.custom_url)
                .hint_text("https://huggingface.co/.../resolve/main/tokenizer.json")
                .desired_width(300.0),
        );
        if ui.button("Load URL").clicked() {
            app.load_custom_url(ui.ctx());
        }
    });

    // Row 2: tabs + controls + status
    ui.add_space(2.0);
    ui.horizontal(|ui| {
        ui.selectable_value(&mut app.active_tab, Tab::Prompt, "📝  Single prompt");
        ui.selectable_value(&mut app.active_tab, Tab::Chat, "💬  Chat conversation");

        ui.separator();
        ui.checkbox(&mut app.show_token_ids, "Show token IDs");

        ui.separator();
        let theme_label = if app.theme.is_dark() {
            "☀ Light"
        } else {
            "🌙 Dark"
        };
        if ui.button(theme_label).clicked() {
            app.theme.toggle();
        }

        // Status indicator
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let (status_text, status_color) = if app.is_loading() {
                ("Loading…", Color32::from_rgb(251, 191, 36))
            } else if app.is_tokenizer_ready() {
                ("Ready", Color32::from_rgb(52, 211, 153))
            } else {
                ("Offline", Color32::from_rgb(248, 113, 113))
            };

            ui.label(RichText::new(status_text).small().color(status_color));

            // Draw status dot
            let (rect, _) = ui.allocate_exact_size(Vec2::new(14.0, 14.0), Sense::hover());
            ui.painter().circle_filled(rect.center(), 5.0, status_color);
        });
    });

    // Error message
    if let Some(error) = app.get_error() {
        ui.add_space(2.0);
        Frame::new()
            .fill(Color32::from_rgba_premultiplied(120, 30, 30, 30))
            .corner_radius(6.0)
            .inner_margin(Margin::same(6))
            .show(ui, |ui| {
                ui.label(
                    RichText::new(format!("⚠ {error}"))
                        .small()
                        .color(Color32::from_rgb(248, 113, 113)),
                );
            });
    }

    ui.add_space(4.0);
}

// ---------------------------------------------------------------------------
// Prompt view
// ---------------------------------------------------------------------------

fn render_prompt_view(app: &mut TokenizerApp, ui: &mut Ui) {
    // ── Top panel: text editor ───────────────────────────────────
    egui::Panel::top("prompt_input_panel")
        .resizable(true)
        .default_size(200.0)
        .min_size(80.0)
        .frame(
            Frame::new()
                .inner_margin(Margin::symmetric(0, 6))
                .fill(ui.style().visuals.panel_fill),
        )
        .show(ui, |ui| {
            ui.label(RichText::new("Input text").strong());
            ui.add_space(2.0);

            ScrollArea::vertical().show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut app.prompt)
                        .desired_width(ui.available_width())
                        .desired_rows(6)
                        .font(egui::TextStyle::Monospace)
                        .hint_text("Type or paste text to tokenize…"),
                );
            });
        });

    // ── Remaining area: tokenized view ───────────────────────────
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.label(RichText::new("Tokenized view").strong());
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if app.is_tokenizer_ready()
                && !app.last_segments.is_empty()
                && ui.button("📋 Copy IDs").clicked()
            {
                let ids: Vec<String> = app.last_segments.iter().map(|s| s.id.to_string()).collect();
                ui.ctx().copy_text(ids.join(", "));
            }
            ui.label(RichText::new(format!("{} tokens", app.last_segments.len())).small());
        });
    });
    ui.add_space(2.0);

    if app.is_loading() {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label("Loading tokenizer…");
        });
    } else if !app.is_tokenizer_ready() {
        ui.label("No tokenizer loaded. Select a model or enter a URL above.");
    } else if app.last_segments.is_empty() {
        ui.label("Type some text to see tokens…");
    } else {
        render_token_view(
            ui,
            &app.prompt,
            &app.last_segments,
            app.show_token_ids,
            &mut app.hovered_token,
        );
    }
}

// ---------------------------------------------------------------------------
// Chat view
// ---------------------------------------------------------------------------

fn render_chat_view(app: &mut TokenizerApp, ui: &mut Ui) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.heading("Chat prompt builder");
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button("+ Add message").clicked() {
                    app.chat_messages.push(ChatMessage {
                        role: ChatRole::User,
                        content: String::new(),
                    });
                }
            });
        });
        ui.add_space(6.0);

        // Extract tokenizer to avoid borrowing `app` inside the loop
        let is_ready = app.is_tokenizer_ready();
        let token_counts = &app.chat_message_token_counts;

        // Message cards
        let mut to_remove = None;
        for (i, msg) in app.chat_messages.iter_mut().enumerate() {
            Frame::new()
                .stroke(ui.style().visuals.widgets.noninteractive.bg_stroke)
                .corner_radius(8.0)
                .inner_margin(Margin::same(8))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Role:");
                        egui::ComboBox::from_id_salt(format!("role_{i}"))
                            .selected_text(msg.role.as_str())
                            .show_ui(ui, |ui| {
                                for role in ChatRole::variants() {
                                    ui.selectable_value(&mut msg.role, role, role.as_str());
                                }
                            });

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if is_ready {
                                let count = token_counts.get(i).copied().unwrap_or(0);
                                ui.label(RichText::new(format!("{count} tokens")).small());
                            }
                            if ui.button("🗑 Remove").clicked() {
                                to_remove = Some(i);
                            }
                        });
                    });

                    ui.add_space(2.0);
                    ui.add(
                        egui::TextEdit::multiline(&mut msg.content)
                            .desired_width(ui.available_width()) // Forces wrapping
                            .desired_rows(2)
                            .font(egui::TextStyle::Monospace)
                            .hint_text(format!("Compose a {} message…", msg.role.as_str())),
                    );
                });

            ui.add_space(4.0);
        }

        if let Some(idx) = to_remove
            && app.chat_messages.len() > 1
        {
            app.chat_messages.remove(idx);
        }

        // Tokenized conversation preview
        ui.add_space(6.0);
        ui.label(RichText::new("Tokenized conversation preview").strong());
        ui.add_space(2.0);

        Frame::new()
            .stroke(ui.style().visuals.widgets.noninteractive.bg_stroke)
            .corner_radius(8.0)
            .inner_margin(Margin::same(8))
            .show(ui, |ui| {
                if app.is_loading() {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Loading tokenizer…");
                    });
                } else if !app.is_tokenizer_ready() {
                    ui.label("No tokenizer loaded.");
                } else if app.last_chat_segments.is_empty() {
                    ui.label("Add at least one message to see tokens.");
                } else {
                    let mut chat_text = String::new();
                    for msg in &app.chat_messages {
                        if !msg.content.trim().is_empty() {
                            let _ = write!(
                                chat_text,
                                "<|im_start|>{}\n{}<|im_end|>\n",
                                msg.role.as_str(),
                                msg.content
                            );
                        }
                    }
                    render_token_view(
                        ui,
                        &chat_text,
                        &app.last_chat_segments,
                        app.show_token_ids,
                        &mut app.hovered_token,
                    );
                }
            });
    });
}

// ---------------------------------------------------------------------------
// Token visualization
// ---------------------------------------------------------------------------

fn render_token_view(
    ui: &mut Ui,
    prompt: &str,
    segments: &[TokenSegment],
    show_ids: bool,
    hovered: &mut Option<usize>,
) {
    *hovered = None;

    // ── Gather layout metrics ────────────────────────────────────────
    let font_id = egui::FontId::new(12.0, FontFamily::Monospace);
    let button_padding = ui.spacing().button_padding; // default (4.0, 1.0)
    let stroke_width = 1.0_f32; // matches the Stroke::new(1.0, …) below
    let item_spacing = Vec2::new(3.0, 3.0);

    // Chip total margin around the text (inner_margin + stroke on each side)
    let chip_margin_x = (button_padding.x + stroke_width) * 2.0;
    let chip_margin_y = (button_padding.y + stroke_width) * 2.0;

    let font_row_height = ui.painter().fonts_mut(|f| f.row_height(&font_id));
    let row_height_sans_spacing = font_row_height + chip_margin_y;

    let available_width = ui.available_width();

    // ── Pre-compute chip display texts + widths ──────────────────────
    struct Chip {
        orig_idx: usize,
        segment_idx: usize,
        display_text: String,
        width: f32,
    }

    let mut chip_infos: Vec<Chip> = Vec::with_capacity(segments.len());
    for (i, segment) in segments.iter().enumerate() {
        let raw = if show_ids {
            segment.id.to_string()
        } else {
            sanitize_token_text(&segment.text)
        };
        let display_text = if raw.is_empty() {
            "·".to_string()
        } else {
            raw
        };

        let text_width = ui
            .painter()
            .layout_no_wrap(display_text.clone(), font_id.clone(), Color32::PLACEHOLDER)
            .size()
            .x;

        chip_infos.push(Chip {
            orig_idx: i,
            segment_idx: i,
            display_text,
            width: text_width + chip_margin_x,
        });
    }

    // ── Group segments by source line ────────────────────────────────
    let mut line_starts = vec![0usize];
    for (idx, ch) in prompt.char_indices() {
        if ch == '\n' {
            line_starts.push(idx + 1);
        }
    }

    let num_lines = line_starts.len();
    let mut source_rows: Vec<Vec<usize>> = vec![Vec::new(); num_lines]; // indices into chip_infos
    for (i, segment) in segments.iter().enumerate() {
        let line_idx = match line_starts.binary_search(&segment.start) {
            Ok(idx) => idx,
            Err(idx) => idx.saturating_sub(1),
        };
        if line_idx < num_lines {
            source_rows[line_idx].push(i);
        }
    }

    // ── Pack chips into visual rows (our own wrapping) ───────────────
    // Each visual row is guaranteed to be exactly one line tall.
    let mut visual_rows: Vec<Vec<usize>> = Vec::new(); // each entry is chip_infos indices

    for source_row in &source_rows {
        if source_row.is_empty() {
            // Empty source line → one blank visual row
            visual_rows.push(Vec::new());
            continue;
        }

        let mut current_row: Vec<usize> = Vec::new();
        let mut current_x = 0.0_f32;

        for &chip_idx in source_row {
            let chip_w = chip_infos[chip_idx].width;
            let needed = if current_row.is_empty() {
                chip_w
            } else {
                item_spacing.x + chip_w
            };

            if !current_row.is_empty() && current_x + needed > available_width {
                // Wrap: push current row, start a new one
                visual_rows.push(std::mem::take(&mut current_row));
                current_x = chip_w;
            } else {
                current_x += needed;
            }
            current_row.push(chip_idx);
        }

        if !current_row.is_empty() {
            visual_rows.push(current_row);
        }
    }

    // ── Virtualized scroll ───────────────────────────────────────────
    let total_visual_rows = visual_rows.len();

    ScrollArea::vertical().auto_shrink(false).show_rows(
        ui,
        row_height_sans_spacing,
        total_visual_rows,
        |ui, row_range| {
            ui.style_mut().spacing.item_spacing = item_spacing;

            for row_idx in row_range {
                let vis_row = &visual_rows[row_idx];

                if vis_row.is_empty() {
                    // Blank line — keep uniform height
                    ui.label("");
                } else {
                    // Use ui.horizontal so egui doesn't try to re-wrap
                    ui.horizontal(|ui| {
                        for &chip_idx in vis_row {
                            let chip = &chip_infos[chip_idx];
                            let segment = &segments[chip.segment_idx];
                            let color = colors::color_for_token(segment.id);

                            let button = egui::Button::new(
                                RichText::new(&chip.display_text)
                                    .family(FontFamily::Monospace)
                                    .size(12.0)
                                    .color(color.text),
                            )
                            .fill(color.bg)
                            .stroke(Stroke::new(stroke_width, color.border))
                            .corner_radius(4.0)
                            .sense(Sense::hover());

                            let response = ui.add(button);

                            if response.hovered() {
                                *hovered = Some(chip.orig_idx);
                            }

                            response.on_hover_ui(|ui| {
                                ui.style_mut().override_font_id =
                                    Some(egui::FontId::new(12.0, FontFamily::Monospace));
                                ui.label(RichText::new("Token details").strong());
                                ui.separator();
                                ui.label(format!("ID:       {}", segment.id));
                                ui.label(format!("Token:    {:?}", segment.token));
                                ui.label(format!("Text:     {:?}", segment.text));
                                ui.label(format!("Offset:   {}–{}", segment.start, segment.end));
                                ui.label(format!(
                                    "Length:   {} chars",
                                    segment.end - segment.start
                                ));
                            });
                        }
                    });
                }
            }
        },
    );
}

fn sanitize_token_text(text: &str) -> String {
    text.replace('\n', "↵").replace('\r', "").replace('\t', "→")
}

// ---------------------------------------------------------------------------
// Stats panel (prompt mode)
// ---------------------------------------------------------------------------

fn render_stats_panel(app: &TokenizerApp, ui: &mut Ui) {
    ScrollArea::vertical().show(ui, |ui| {
        let token_count = app.last_segments.len();
        let char_count = app.prompt.chars().count();
        let byte_count = app.prompt.len();
        let tokens_per_100 = if char_count > 0 {
            token_count as f64 / char_count as f64 * 100.0
        } else {
            0.0
        };

        ui.heading("Statistics");
        ui.add_space(6.0);

        stat_card(
            ui,
            "Tokens",
            &token_count.to_string(),
            &format!(
                "{tokens_per_100:.2} tokens / 100 chars\n{char_count} chars · {byte_count} bytes"
            ),
        );

        // Context window usage
        let model = if app.is_custom {
            None
        } else {
            app.model_registry.get(app.selected_model)
        };

        let (ctx_pct, ctx_detail) = model.map_or_else(
            || ("—".into(), "Custom tokenizer".into()),
            |m| {
                m.context_window.map_or_else(
                    || ("—".into(), "Context window unknown".into()),
                    |ctx| {
                        let pct = if ctx > 0 {
                            token_count as f64 / ctx as f64 * 100.0
                        } else {
                            0.0
                        };
                        (
                            format!("{pct:.2}%"),
                            format!("{token_count} / {ctx} tokens"),
                        )
                    },
                )
            },
        );

        stat_card(ui, "Context used", &ctx_pct, &ctx_detail);

        // Cost
        if let Some(m) = model
            && let Some(pricing) = &m.pricing
        {
            let cost = token_count as f64 * pricing.input_per / 1_000_000.0;
            stat_card(
                ui,
                "Est. input cost",
                &format!("${cost:.6}"),
                &format!("${:.2} / 1M tokens", pricing.input_per),
            );

            if let Some(max_out) = m.max_output_tokens {
                let out_cost = max_out as f64 * pricing.output_per / 1_000_000.0;
                stat_card(
                    ui,
                    "Max output cost",
                    &format!("${out_cost:.6}"),
                    &format!("${:.2} / 1M tokens", pricing.output_per),
                );
            }

            if let Some(cached) = pricing.cached_input_per {
                let cached_cost = token_count as f64 * cached / 1_000_000.0;
                stat_card(
                    ui,
                    "Cached input cost",
                    &format!("${cached_cost:.6}"),
                    &format!("${cached:.2} / 1M tokens"),
                );
            }
        }

        ui.add_space(12.0);

        // Model info
        ui.heading("Model info");
        ui.add_space(6.0);

        if let Some(m) = model {
            model_info(ui, m);
        } else if let Some(name) = app.tokenizer_manager.current_name() {
            info_row(ui, "Source", name);
            info_row(ui, "Type", "Custom (URL or file)");
            ui.label(
                RichText::new("Metadata and pricing not available for custom tokenizers.").small(),
            );
        }

        ui.add_space(12.0);

        // Educational primer
        ui.heading("What is tokenization?");
        ui.add_space(4.0);
        ui.label(
            RichText::new(
                "Tokenization breaks text into small chunks called tokens based on how commonly the pattern occurs in training data. \
                 A token might be a whole word, part of a word, or punctuation.\n\n\
                 Language models process tokens in sequence to understand and \
                 predict text. Pricing and context limits are measured in tokens, \
                 so shorter prompts mean cheaper and faster inference.",
            )
            .small(),
        );
    });
}

// ---------------------------------------------------------------------------
// Stats panel (chat mode)
// ---------------------------------------------------------------------------

fn render_chat_stats(app: &mut TokenizerApp, ui: &mut Ui) {
    ScrollArea::vertical().show(ui, |ui| {
        let total_tokens = app.last_chat_segments.len();
        let msg_count = app
            .chat_messages
            .iter()
            .filter(|m| !m.content.trim().is_empty())
            .count();

        ui.heading("Chat statistics");
        ui.add_space(6.0);

        stat_card(
            ui,
            "Chat tokens",
            &total_tokens.to_string(),
            &format!("Across {msg_count} messages"),
        );

        if let Some(m) = app.model_registry.get(app.selected_model)
            && let Some(pricing) = &m.pricing
        {
            let cost = total_tokens as f64 * pricing.input_per / 1_000_000.0;
            stat_card(
                ui,
                "Est. input cost",
                &format!("${cost:.6}"),
                &format!("${:.2} / 1M tokens", pricing.input_per),
            );
        }

        ui.add_space(12.0);
        ui.heading("Per-message tokens");
        ui.add_space(4.0);

        if app.is_tokenizer_ready() {
            let mut running = 0usize;
            for (i, msg) in app.chat_messages.iter().enumerate() {
                if msg.content.trim().is_empty() {
                    continue;
                }
                let count = app.chat_message_token_counts.get(i).copied().unwrap_or(0);
                running += count;
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("{}:", msg.role.as_str())).small());
                    ui.label(RichText::new(format!("{count} tokens")).small().strong());
                    ui.label(
                        RichText::new(format!("(running: {running})"))
                            .small()
                            .color(ui.style().visuals.noninteractive().fg_stroke.color),
                    );
                });
            }
        }

        ui.add_space(12.0);

        // Model info
        ui.heading("Model info");
        ui.add_space(6.0);

        if let Some(m) = app.model_registry.get(app.selected_model) {
            if !app.is_custom {
                model_info(ui, m);
            } else if let Some(name) = app.tokenizer_manager.current_name() {
                info_row(ui, "Source", name);
                info_row(ui, "Type", "Custom (URL or file)");
            }
        } else if let Some(name) = app.tokenizer_manager.current_name() {
            info_row(ui, "Source", name);
            info_row(ui, "Type", "Custom (URL or file)");
        }
    });
}

// ---------------------------------------------------------------------------
// UI helpers
// ---------------------------------------------------------------------------

fn stat_card(ui: &mut Ui, label: &str, value: &str, sublabel: &str) {
    Frame::new()
        .stroke(ui.style().visuals.widgets.noninteractive.bg_stroke)
        .corner_radius(8.0)
        .inner_margin(Margin::same(10))
        .show(ui, |ui| {
            ui.label(
                RichText::new(label)
                    .small()
                    .color(ui.style().visuals.noninteractive().fg_stroke.color),
            );
            ui.add_space(2.0);
            ui.label(RichText::new(value).strong().size(22.0));
            ui.add_space(2.0);
            ui.label(
                RichText::new(sublabel)
                    .small()
                    .color(ui.style().visuals.noninteractive().fg_stroke.color),
            );
        });
    ui.add_space(4.0);
}

fn info_row(ui: &mut Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(label)
                .small()
                .color(ui.style().visuals.noninteractive().fg_stroke.color),
        );
        ui.label(RichText::new(value).small().strong());
    });
}

fn model_info(ui: &mut Ui, m: &ModelEntry) {
    info_row(ui, "Name", m.name);
    info_row(ui, "Description", m.description);
    if let Some(ctx) = m.context_window {
        info_row(ui, "Context window", &format!("{ctx} tokens"));
    }
    if let Some(max_out) = m.max_output_tokens {
        info_row(ui, "Max output", &format!("{max_out} tokens"));
    }
    if !m.modalities.is_empty() {
        info_row(ui, "Modalities", &m.modalities.join(", "));
    }
    if let Some(p) = &m.pricing {
        info_row(
            ui,
            "Pricing",
            &format!("${:.2}/1M in · ${:.2}/1M out", p.input_per, p.output_per),
        );
    }
}
