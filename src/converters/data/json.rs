use core::fmt;
use std::fmt::{Display, Formatter};
use bytes::Bytes;
use eframe::egui::{Color32, FontFamily, FontId, TextFormat};
use eframe::egui::text::LayoutJob;
use jaq_core::compile;
use jaq_core::load::{lex, File};
use jaq_json::{read, Val};
use crate::converters::data::{color_from_hex, fmt_seq, visible_indentation, FormatterFn, PpOpts, Settings};

#[derive(Clone, Copy)]
pub(crate) enum JqColor {
    Plain,
    Null,
    Boolean,
    Number,
    Key,
    String,
}

pub(crate) fn append_colored_value(job: &mut LayoutJob, value: &Val, settings: &Settings, level: usize) {
    match value {
        Val::Null => append_jq_text(job, "null", JqColor::Null),
        Val::Bool(value) => append_jq_text(job, &value.to_string(), JqColor::Boolean),
        Val::Num(value) => append_jq_text(job, &value.to_string(), JqColor::Number),
        Val::TStr(bytes) | Val::BStr(bytes) => {
            let text = json_string(bytes);
            append_jq_text(job, &text, JqColor::String);
        }
        Val::Arr(values) if values.is_empty() => append_jq_text(job, "[]", JqColor::Plain),
        Val::Arr(values) => {
            append_jq_text(job, "[", JqColor::Plain);
            append_jq_sequence_start(job, settings);

            let mut values = values.iter().peekable();

            while let Some(value) = values.next() {
                append_jq_indent(job, settings, level + 1);
                append_colored_value(job, value, settings, level + 1);

                if values.peek().is_some() {
                    append_jq_text(job, ",", JqColor::Plain);
                }

                append_jq_sequence_newline(job, settings);
            }

            append_jq_indent(job, settings, level);
            append_jq_text(job, "]", JqColor::Plain);
        }
        Val::Obj(values) if values.is_empty() => append_jq_text(job, "{}", JqColor::Plain),
        Val::Obj(values) => {
            append_jq_text(job, "{", JqColor::Plain);
            append_jq_sequence_start(job, settings);

            let mut values = values.iter().peekable();

            while let Some((key, value)) = values.next() {
                append_jq_indent(job, settings, level + 1);

                let key = json_key(key);
                append_jq_text(job, &key, JqColor::Key);
                append_jq_text(job, ":", JqColor::Plain);

                if !settings.compact {
                    append_jq_text(job, " ", JqColor::Plain);
                }

                append_colored_value(job, value, settings, level + 1);

                if values.peek().is_some() {
                    append_jq_text(job, ",", JqColor::Plain);
                }

                append_jq_sequence_newline(job, settings);
            }

            append_jq_indent(job, settings, level);
            append_jq_text(job, "}", JqColor::Plain);
        }
    }
}

// -----------------------------------------------------------------------------
// value formatter (plain text)
// -----------------------------------------------------------------------------

pub(crate) fn append_value(job: &mut LayoutJob, value: &Val, settings: &Settings) {
    if !job.text.is_empty() {
        append_jq_text(job, "\n", JqColor::Plain);
    }

    match value {
        Val::TStr(bytes) | Val::BStr(bytes) if settings.raw_output => {
            append_jq_text(
                job,
                String::from_utf8_lossy(bytes.as_ref()).as_ref(),
                JqColor::String,
            );
        }
        value => append_colored_value(job, value, settings, 0),
    }
}

pub(crate) fn json_slice(slice: &[u8]) -> impl Iterator<Item = Result<Val, String>> + '_ {
    read::parse_many(slice).map(|value| value.map_err(|error| error.to_string()))
}

fn append_jq_sequence_start(job: &mut LayoutJob, settings: &Settings) {
    if !settings.compact {
        append_jq_text(job, "\n", JqColor::Plain);
    }
}

fn append_jq_sequence_newline(job: &mut LayoutJob, settings: &Settings) {
    if !settings.compact {
        append_jq_text(job, "\n", JqColor::Plain);
    }
}

fn append_jq_indent(job: &mut LayoutJob, settings: &Settings, level: usize) {
    const WORD_JOINER: &str = "\u{2060}";

    if settings.compact || level == 0 {
        return;
    }

    // Prevent egui from removing or ignoring leading whitespace.
    // place an invisible WORD_JOINER at the beginning.
    append_jq_text(job, WORD_JOINER, JqColor::Plain);

    let indent = " ".repeat(settings.indent.saturating_mul(level));
    append_jq_text(job, &indent, JqColor::Plain);
}

fn append_jq_text(job: &mut LayoutJob, text: &str, color: JqColor) {
    if text.is_empty() {
        return;
    }

    let color = match color {
        JqColor::Plain => Color32::GRAY,
        JqColor::Null => color_from_hex("#a09a9a"),
        JqColor::Boolean => color_from_hex("#4892ef"),
        JqColor::Number => color_from_hex("#fbab23"),
        JqColor::Key => color_from_hex("#7d9bb5"),
        JqColor::String => color_from_hex("#dfdfdf"),
    };

    job.append(
        text,
        0.0,
        TextFormat {
            font_id: FontId::new(11.5, FontFamily::Monospace),
            color,
            ..Default::default()
        },
    );
}

pub(crate) fn json_key(value: &Val) -> String {
    match value {
        Val::TStr(bytes) | Val::BStr(bytes) => json_string(bytes),
        other => other.to_string(),
    }
}

pub(crate) fn json_string(bytes: &Bytes) -> String {
    let text = String::from_utf8_lossy(bytes.as_ref());
    let mut output = String::with_capacity(text.len().saturating_add(2));

    output.push('"');

    for character in text.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if (character as u32) < 0x20 => {
                output.push_str(&format!("\\u{:04x}", character as u32));
            }
            character => output.push(character),
        }
    }

    output.push('"');
    output
}

fn fmt_val(f: &mut Formatter, opts: &PpOpts, level: usize, v: &Val) -> fmt::Result {
    match v {
        Val::Null => span(f, "null", "null"),
        Val::Bool(b) => span(f, "boolean", b),
        Val::Num(n) => span(f, "number", n),
        Val::TStr(bytes) | Val::BStr(bytes) => span(
            f,
            "string",
            FormatterFn(|f: &mut Formatter| write_bytes_as_json_string(f, bytes)),
        ),
        Val::Arr(a) if a.is_empty() => write!(f, "[]"),
        Val::Arr(a) => {
            write!(f, "[")?;
            fmt_seq(f, opts, level, a.iter(), |f, value| {
                fmt_val(f, opts, level + 1, value)
            })?;
            write!(f, "]")
        }
        Val::Obj(o) if o.is_empty() => write!(f, "{{}}"),
        Val::Obj(o) => {
            write!(f, "{{")?;
            fmt_seq(f, opts, level, o.iter(), |f, (key, value)| {
                span(f, "key", FormatterFn(|f: &mut Formatter| fmt_plain(f, key)))?;
                write!(f, ":")?;

                if !opts.compact {
                    write!(f, " ")?;
                }

                fmt_val(f, opts, level + 1, value)
            })?;
            write!(f, "}}")
        }
    }
}

fn fmt_plain(f: &mut Formatter, v: &Val) -> fmt::Result {
    match v {
        Val::Null => write!(f, "null"),
        Val::Bool(b) => write!(f, "{b}"),
        Val::Num(n) => write!(f, "{n}"),
        Val::TStr(bytes) | Val::BStr(bytes) => write_bytes_as_json_string(f, bytes),
        // Since jaq-json 2.0 allows arbitrary values ​​for keys, it delegates rare cases to `Display`.
        other => write!(f, "{other}"),
    }
}

// -----------------------------------------------------------------------------
// bytes helpers (for jaq-json 2.0 TStr/BStr)
// -----------------------------------------------------------------------------

fn write_bytes_as_json_string(f: &mut Formatter, bytes: &Bytes) -> fmt::Result {
    let text = match std::str::from_utf8(bytes.as_ref()) {
        Ok(text) => std::borrow::Cow::Borrowed(text),
        Err(_) => String::from_utf8_lossy(bytes.as_ref()),
    };

    write!(f, "\"")?;

    for ch in text.chars() {
        match ch {
            '"' => write!(f, "\\\"")?,
            '\\' => write!(f, "\\\\")?,
            '\n' => write!(f, "\\n")?,
            '\r' => write!(f, "\\r")?,
            '\t' => write!(f, "\\t")?,
            c if (c as u32) < 0x20 => write!(f, "\\u{:04x}", c as u32)?,
            c => write!(f, "{c}")?,
        }
    }

    write!(f, "\"")
}

// -----------------------------------------------------------------------------
// colors for formatted jq diagnostics
// -----------------------------------------------------------------------------
#[derive(Debug)]
struct Report {
    message: String,
    labels: Vec<(core::ops::Range<usize>, StringColors, DiagnosticColor)>,
}

pub(crate) fn format_load_errors(errors: jaq_core::load::Errors<&str, ()>) -> LayoutJob {
    let reports = errors
        .into_iter()
        .map(|(file, error)| {
            let code = file.code;

            let reports = match error {
                jaq_core::load::Error::Io(errors) => errors
                    .into_iter()
                    .map(|error| report_io(code, error))
                    .collect(),
                jaq_core::load::Error::Lex(errors) => errors
                    .into_iter()
                    .map(|error| report_lex(code, error))
                    .collect(),
                jaq_core::load::Error::Parse(errors) => errors
                    .into_iter()
                    .map(|error| report_parse(code, error))
                    .collect(),
            };

            (file.map_code(str::to_owned), reports)
        })
        .collect();

    reports_layout(reports)
}

pub(crate) fn format_compile_errors(errors: compile::Errors<&str, ()>) -> LayoutJob {
    let reports = errors
        .into_iter()
        .map(|(file, errors)| {
            let code = file.code;
            let reports = errors
                .into_iter()
                .map(|error| report_compile(code, error))
                .collect();

            (file.map_code(str::to_owned), reports)
        })
        .collect();

    reports_layout(reports)
}

fn report_io(code: &str, (path, error): (&str, String)) -> Report {
    Report {
        message: format!("could not load file {path}: {error}"),
        labels: vec![(
            jaq_core::load::span(code, path),
            vec![(error, None)],
            DiagnosticColor::Red,
        )],
    }
}

fn reports_layout(file_reports: Vec<FileReports>) -> LayoutJob {
    let mut job = LayoutJob::default();

    for (file, reports) in file_reports {
        let index = codesnake::LineIndex::new(&file.code);

        for report in reports {
            append_diagnostic_text(
                &mut job,
                &format!("⚠️  Error: {}\n", report.message),
                Color32::GRAY,
            );

            let block = report.into_block(&index);

            append_diagnostic_text(
                &mut job,
                &format!("{}\n", block.prologue()),
                Color32::GRAY,
            );

            append_diagnostic_html_layout(&mut job, &block.to_string());

            append_diagnostic_text(
                &mut job,
                &format!("{}\n", block.epilogue()),
                Color32::GRAY,
            );
        }
    }

    job
}

fn append_diagnostic_text(job: &mut LayoutJob, text: &str, color: Color32) {
    let text = visible_indentation(text, 4);
    append_layout_text(job, &text, color);
}

fn append_layout_text(job: &mut LayoutJob, text: &str, color: Color32) {
    job.append(
        text,
        0.0,
        TextFormat {
            font_id: FontId::new(11.5, FontFamily::Monospace),
            color,
            ..Default::default()
        },
    );
}

fn append_diagnostic_html_layout(job: &mut LayoutJob, html: &str) {
    let mut at_line_start = true;
    let mut inserted_guard = false;
    let mut current_class: Option<String> = None;
    let mut class_stack: Vec<Option<String>> = Vec::new();
    let mut text = String::new();
    let mut characters = html.chars().peekable();

    while let Some(character) = characters.next() {
        if character != '<' {
            text.push(character);
            continue;
        }

        let mut tag = String::new();
        let mut closed = false;

        for character in characters.by_ref() {
            if character == '>' {
                closed = true;
                break;
            }

            tag.push(character);
        }

        if !closed {
            text.push('<');
            text.push_str(&tag);
            break;
        }

        let tag = tag.trim();

        if tag.starts_with('/') && tag[1..].trim_start().starts_with("span") {
            append_diagnostic_fragment(
                job,
                &text,
                current_class.as_deref(),
                &mut at_line_start,
                &mut inserted_guard,
            );
            text.clear();

            current_class = class_stack.pop().flatten();
            continue;
        }

        if tag.starts_with("span") {
            append_diagnostic_fragment(
                job,
                &text,
                current_class.as_deref(),
                &mut at_line_start,
                &mut inserted_guard,
            );
            text.clear();

            class_stack.push(current_class.clone());
            current_class = span_class(tag).or(current_class);
            continue;
        }

        text.push('<');
        text.push_str(tag);
        text.push('>');
    }

    append_diagnostic_fragment(
        job,
        &text,
        current_class.as_deref(),
        &mut at_line_start,
        &mut inserted_guard,
    );
}

fn span_class(tag: &str) -> Option<String> {
    let class = tag
        .strip_prefix("span")?
        .trim_start()
        .strip_prefix("class")?
        .trim_start()
        .strip_prefix('=')?
        .trim_start();

    let class = class
        .strip_prefix('"')
        .or_else(|| class.strip_prefix('\''))?;

    let end = class.find(['"', '\''])?;
    Some(class[..end].to_owned())
}

fn append_diagnostic_fragment(
    job: &mut LayoutJob,
    text: &str,
    class: Option<&str>,
    at_line_start: &mut bool,
    inserted_guard: &mut bool,
) {
    if text.is_empty() {
        return;
    }

    let text = visible_indentation_fragment(text, 4, at_line_start, inserted_guard);
    append_layout_text(job, &text, color_from_str(class));
}

fn visible_indentation_fragment(
    text: &str,
    tab_width: usize,
    at_line_start: &mut bool,
    inserted_guard: &mut bool,
) -> String {
    const WORD_JOINER: char = '\u{2060}';

    let mut rendered = String::with_capacity(text.len());

    for character in text.chars() {
        match character {
            '\n' => {
                rendered.push('\n');
                *at_line_start = true;
                *inserted_guard = false;
            }
            '\t' if *at_line_start => {
                if !*inserted_guard {
                    rendered.push(WORD_JOINER);
                    *inserted_guard = true;
                }

                rendered.extend(std::iter::repeat_n(' ', tab_width));
            }
            ' ' if *at_line_start => {
                if !*inserted_guard {
                    rendered.push(WORD_JOINER);
                    *inserted_guard = true;
                }

                rendered.push(' ');
            }
            character => {
                rendered.push(character);
                *at_line_start = false;
            }
        }
    }

    rendered
}

fn color_from_str(class: Option<&str>) -> Color32 {
    match class {
        Some("red") => color_from_hex("#ab7f1a"),
        Some("yellow") => color_from_hex("#d53130"),
        Some("null") => color_from_hex("#a09a9a"),
        Some("key") => color_from_hex("#7d9bb5"),
        Some("string") => color_from_hex("#dfdfdf"),
        Some("number") => color_from_hex("#fbab23"),
        Some("boolean") => color_from_hex("#4892ef"),
        _ => Color32::GRAY,
    }
}

fn report_lex(code: &str, (expected, found): lex::Error<&str>) -> Report {
    use jaq_core::load::span;
    // truncate found string to its first character
    let found = &found[..found.char_indices().nth(1).map_or(found.len(), |(i, _)| i)];

    let found_range = span(code, found);
    let found = match found {
        "" => [("unexpected end of input".to_string(), None)].into(),
        c => [("unexpected character ", None), (c, Some(DiagnosticColor::Red))]
            .map(|(s, c)| (s.into(), c))
            .into(),
    };
    let label = (found_range, found, DiagnosticColor::Red);

    let labels = match expected {
        lex::Expect::Delim(open) => {
            let text = [("unclosed delimiter ", None), (open, Some(DiagnosticColor::Yellow))]
                .map(|(s, c)| (s.into(), c));
            Vec::from([(span(code, open), text.into(), DiagnosticColor::Yellow), label])
        }
        _ => Vec::from([label]),
    };

    Report {
        message: format!("expected {}", expected.as_str()),
        labels,
    }
}

fn span(f: &mut Formatter, class: &str, value: impl Display) -> fmt::Result {
    write!(f, "<span class=\"{class}\">{value}</span>")
}


fn report_parse(code: &str, (expected, found): jaq_core::load::parse::Error<&str>) -> Report {
    let found_text = if found.is_empty() {
        "unexpected end of input"
    } else {
        "unexpected token"
    };

    Report {
        message: format!("expected {}", expected.as_str()),
        labels: vec![(
            jaq_core::load::span(code, found),
            vec![(found_text.to_owned(), None)],
            DiagnosticColor::Red,
        )],
    }
}

fn report_compile(code: &str, (found, undefined): compile::Error<&str>) -> Report {
    use compile::Undefined::Filter;

    let wrong_arity =
        |expected, actual| format!("wrong number of arguments (expected {expected}, found {actual})");

    let message = match (found, undefined) {
        ("reduce", Filter(arity)) => wrong_arity(2, arity),
        ("foreach", Filter(arity)) => wrong_arity("2 or 3".parse().unwrap(), arity),
        (_, undefined) => format!("undefined {}", undefined.as_str()),
    };

    Report {
        message: message.clone(),
        labels: vec![(
            jaq_core::load::span(code, found),
            vec![(message, None)],
            DiagnosticColor::Red,
        )],
    }
}

impl Report {
    // codesnake 0.2.1 is intentionally used here.
    // Its `Label::with_style` output is parsed as HTML spans and rendered as egui colors.
    // Newer versions changed the Block/style type and do not preserve the old colored snake output.
    fn into_block(self, index: &codesnake::LineIndex) -> CodeBlock {
        use codesnake::{Block, CodeWidth, Label};

        let color_maybe = |(text, color): (String, Option<DiagnosticColor>)| match color {
            None => text,
            Some(color) => color.apply(text).to_string(),
        };

        let labels = self.labels.into_iter().map(|(range, text, color)| {
            let text = text
                .into_iter()
                .map(color_maybe)
                .collect::<Vec<_>>()
                .join("");

            Label::new(range)
                .with_text(text)
                .with_style(move |text| color.apply(text).to_string())
        });

        Block::new(index, labels)
            .expect("diagnostic label range must be valid")
            .map_code(|code| {
                let code = code.replace('\t', "    ");
                let width = unicode_width::UnicodeWidthStr::width(code.as_str());

                CodeWidth::new(code, width.max(1))
            })
    }
}

// -----------------------------------------------------------------------------
// diagnostic formatting
// -----------------------------------------------------------------------------

type FileReports = (File<String, ()>, Vec<Report>);
type StringColors = Vec<(String, Option<DiagnosticColor>)>;
type CodeBlock = codesnake::Block<codesnake::CodeWidth<String>, String>;

#[derive(Clone, Debug)]
enum DiagnosticColor {
    Yellow,
    Red,
}

impl DiagnosticColor {
    fn apply(&self, value: impl Display) -> String {
        let class = match self {
            Self::Yellow => "yellow",
            Self::Red => "red",
        };

        format!("<span class=\"{class}\">{value}</span>")
    }
}
