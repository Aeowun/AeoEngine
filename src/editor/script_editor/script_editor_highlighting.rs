use crate::scripting::token::{Token, TokenKind};
use egui::Color32;
use egui::text::{LayoutJob, TextFormat};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HighlightKind {
    Keyword,
    Type,
    Boolean,
    Number,
    String,
    Function,
    Property,
    Operator,
    Comment,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct HighlightSpan {
    pub start: usize,
    pub end: usize,
    pub kind: HighlightKind,
}

pub(crate) fn build_highlight_spans(source: &str, tokens: &[Token]) -> Vec<HighlightSpan> {
    let mut spans = Vec::with_capacity(tokens.len());

    for token in tokens {
        let start = token.span.start as usize;
        let end = token.span.end as usize;

        if start >= end || end > source.len() {
            continue;
        }

        let Some(kind) = highlight_kind(source, start, end, &token.kind) else {
            continue;
        };

        spans.push(HighlightSpan { start, end, kind });
    }

    for (start, end) in find_comment_spans(source) {
        spans.push(HighlightSpan {
            start,
            end,
            kind: HighlightKind::Comment,
        });
    }

    spans.sort_by_key(|span| (span.start, span.end));
    spans
}

fn highlight_kind(
    source: &str,
    start: usize,
    end: usize,
    token: &TokenKind,
) -> Option<HighlightKind> {
    match token {
        TokenKind::Entity
        | TokenKind::Fn
        | TokenKind::Return
        | TokenKind::If
        | TokenKind::Else
        | TokenKind::While
        | TokenKind::For
        | TokenKind::In
        | TokenKind::Const
        | TokenKind::Import
        | TokenKind::On => Some(HighlightKind::Keyword),

        TokenKind::True | TokenKind::False | TokenKind::Nil => Some(HighlightKind::Boolean),

        TokenKind::Number(_) => Some(HighlightKind::Number),

        TokenKind::String(_) => Some(HighlightKind::String),

        TokenKind::Identifier(name) if matches!(name.as_str(), "number" | "string" | "bool") => {
            Some(HighlightKind::Type)
        }

        TokenKind::Identifier(_) => {
            if previous_non_whitespace(source, start) == Some('.') {
                Some(HighlightKind::Property)
            } else if next_non_whitespace(source, end) == Some('(') {
                Some(HighlightKind::Function)
            } else {
                None
            }
        }

        TokenKind::Plus
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::Slash
        | TokenKind::Percent
        | TokenKind::Equal
        | TokenKind::EqualEqual
        | TokenKind::Bang
        | TokenKind::BangEqual
        | TokenKind::Less
        | TokenKind::LessEqual
        | TokenKind::Greater
        | TokenKind::GreaterEqual
        | TokenKind::AndAnd
        | TokenKind::OrOr
        | TokenKind::PlusEqual
        | TokenKind::MinusEqual
        | TokenKind::StarEqual
        | TokenKind::SlashEqual => Some(HighlightKind::Operator),

        _ => None,
    }
}

fn highlight_color(kind: HighlightKind, _base: Color32) -> Color32 {
    match kind {
        HighlightKind::Keyword => Color32::from_rgb(198, 120, 221),

        HighlightKind::Type => Color32::from_rgb(86, 182, 194),

        HighlightKind::Boolean => Color32::from_rgb(86, 156, 214),

        HighlightKind::Number => Color32::from_rgb(209, 154, 102),

        HighlightKind::String => Color32::from_rgb(152, 195, 121),

        HighlightKind::Function => Color32::from_rgb(97, 175, 239),

        HighlightKind::Property => Color32::from_rgb(220, 220, 170),

        HighlightKind::Operator => Color32::from_rgb(180, 180, 180),

        HighlightKind::Comment => Color32::from_rgb(106, 153, 85),
    }
}

pub(crate) fn build_layout_job(
    ui: &egui::Ui,
    source: &str,
    spans: &[HighlightSpan],
    wrap_width: f32,
) -> LayoutJob {
    let font_id = egui::TextStyle::Monospace.resolve(ui.style());
    let base_color = ui.visuals().text_color();

    let mut job = LayoutJob::default();

    job.break_on_newline = true;
    job.wrap.max_width = wrap_width;

    let mut cursor = 0;

    for span in spans {
        if span.start > source.len() || span.end > source.len() || span.start < cursor {
            continue;
        }

        if span.start > cursor {
            job.append(
                &source[cursor..span.start],
                0.0,
                TextFormat::simple(font_id.clone(), base_color),
            );
        }

        let mut format =
            TextFormat::simple(font_id.clone(), highlight_color(span.kind, base_color));

        if span.kind == HighlightKind::Comment {
            format.italics = true;
        }

        job.append(&source[span.start..span.end], 0.0, format);

        cursor = span.end;
    }

    if cursor < source.len() {
        job.append(
            &source[cursor..],
            0.0,
            TextFormat::simple(font_id, base_color),
        );
    }

    job
}

pub(crate) fn find_comment_spans(source: &str) -> Vec<(usize, usize)> {
    let bytes = source.as_bytes();
    let mut spans = Vec::new();

    let mut index = 0;
    let mut in_string = false;
    let mut escaped = false;

    while index < bytes.len() {
        let byte = bytes[index];

        if in_string {
            if escaped {
                escaped = false;
                index += 1;
                continue;
            }

            if byte == b'\\' {
                escaped = true;
                index += 1;
                continue;
            }

            if byte == b'"' {
                in_string = false;
            }

            index += 1;
            continue;
        }

        if byte == b'"' {
            in_string = true;
            index += 1;
            continue;
        }

        if byte == b'/' && index + 1 < bytes.len() && bytes[index + 1] == b'/' {
            let start = index;

            index += 2;

            while index < bytes.len() && bytes[index] != b'\n' {
                index += 1;
            }

            spans.push((start, index));
            continue;
        }

        index += 1;
    }

    spans
}

pub(crate) fn previous_non_whitespace(source: &str, offset: usize) -> Option<char> {
    source.get(..offset).and_then(|prefix| {
        prefix
            .chars()
            .rev()
            .find(|character| !character.is_whitespace())
    })
}

pub(crate) fn next_non_whitespace(source: &str, offset: usize) -> Option<char> {
    source
        .get(offset..)
        .and_then(|suffix| suffix.chars().find(|character| !character.is_whitespace()))
}

pub(crate) fn byte_offset_to_char_index(source: &str, byte_offset: usize) -> usize {
    source
        .get(..byte_offset.min(source.len()))
        .map(|prefix| prefix.chars().count())
        .unwrap_or_else(|| source.chars().count())
}
