use super::parser;
use eframe::egui;

/// Highlight easymark, memoizing previous output to save CPU.
///
/// In practice, the highlighter is fast enough not to need any caching.
#[derive(Default)]
pub struct MemoizedHighlighter {
    style: egui::Style,
    code: String,
    output: egui::text::LayoutJob,
}

impl MemoizedHighlighter {
    pub fn highlight(&mut self, egui_style: &egui::Style, code: &str) -> egui::text::LayoutJob {
        if (&self.style, self.code.as_str()) != (egui_style, code) {
            self.style = egui_style.clone();
            code.clone_into(&mut self.code);
            self.output = highlight_easymark(egui_style, code);
        }
        self.output.clone()
    }
}

pub fn highlight_easymark(egui_style: &egui::Style, mut text: &str) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    let mut mark = parser::Style::default();
    let mut start_of_line = true;

    while !text.is_empty() {
        if start_of_line && text.starts_with("```") {
            mark = Default::default();
            mark.code = true;
            let end = text.find("\n```").map_or_else(|| text.len(), |i| i + 4);
            job.append(&text[..end], 0.0, format_from_style(egui_style, &mark));
            text = &text[end..];
            mark.code = false;
            continue;
        }

        if text.starts_with('`') {
            mark.code = true;
            let end = text[1..]
                .find(&['`', '\n'][..])
                .map_or_else(|| text.len(), |i| i + 2);
            job.append(&text[..end], 0.0, format_from_style(egui_style, &mark));
            text = &text[end..];
            mark.code = false;
            continue;
        }

        let mut skip;

        if text.starts_with('\\') && text.len() >= 2 {
            skip = 2;
        } else if start_of_line && text.starts_with(' ') {
            // we don't preview indentation, because it is confusing
            skip = 1;
        } else if start_of_line && text.starts_with("# ") {
            mark.heading = true;
            skip = 2;
        } else if start_of_line && text.starts_with("> ") {
            mark.quoted = true;
            skip = 2;
            // we don't preview indentation, because it is confusing
        } else if start_of_line && text.starts_with("- ") {
            skip = 2;
            // we don't preview indentation, because it is confusing
        } else if text.starts_with('*') {
            skip = 1;
            if mark.strong {
                // Include the character that is ending this style:
                job.append(&text[..skip], 0.0, format_from_style(egui_style, &mark));
                text = &text[skip..];
                skip = 0;
            }
            mark.strong ^= true;
        } else if text.starts_with('$') {
            skip = 1;
            if mark.small {
                // Include the character that is ending this style:
                job.append(&text[..skip], 0.0, format_from_style(egui_style, &mark));
                text = &text[skip..];
                skip = 0;
            }
            mark.small ^= true;
        } else if text.starts_with('^') {
            skip = 1;
            if mark.raised {
                // Include the character that is ending this style:
                job.append(&text[..skip], 0.0, format_from_style(egui_style, &mark));
                text = &text[skip..];
                skip = 0;
            }
            mark.raised ^= true;
        } else {
            skip = 0;
        }
        // Note: we don't preview underline, strikethrough and italics because it confuses things.

        // Swallow everything up to the next special character:
        let line_end = text[skip..]
            .find('\n')
            .map_or_else(|| text.len(), |i| skip + i + 1);
        let end = text[skip..]
            .find(&['*', '`', '~', '_', '/', '$', '^', '\\', '<', '['][..])
            .map_or_else(|| text.len(), |i| (skip + i).max(1));

        if line_end <= end {
            job.append(&text[..line_end], 0.0, format_from_style(egui_style, &mark));
            text = &text[line_end..];
            start_of_line = true;
            mark = Default::default();
        } else {
            job.append(&text[..end], 0.0, format_from_style(egui_style, &mark));
            text = &text[end..];
            start_of_line = false;
        }
    }

    job
}

fn format_from_style(style: &egui::Style, mark: &parser::Style) -> egui::text::TextFormat {
    use egui::{Align, Color32, Stroke, TextStyle};

    let color = if mark.strong || mark.heading {
        style.visuals.strong_text_color()
    } else if mark.quoted {
        style.visuals.weak_text_color()
    } else {
        style.visuals.text_color()
    };

    let text_style = if mark.heading {
        TextStyle::Heading
    } else if mark.code {
        TextStyle::Monospace
    } else if mark.small | mark.raised {
        TextStyle::Small
    } else {
        TextStyle::Body
    };

    let stroke = Stroke::new(1.0, color);
    let none = Stroke::NONE;

    egui::text::TextFormat {
        font_id: text_style.resolve(style),
        extra_letter_spacing: 0.0,
        line_height: None,
        color,
        background: if mark.code {
            style.visuals.code_bg_color
        } else {
            Color32::TRANSPARENT
        },
        expand_bg: 1.0,
        italics: mark.italics,
        underline: if mark.underline { stroke } else { none },
        strikethrough: if mark.strikethrough { stroke } else { none },
        valign: if mark.raised {
            Align::TOP
        } else {
            Align::BOTTOM
        },
    }
}
