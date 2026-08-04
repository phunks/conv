use crate::app::result::ConvertResult;
use crate::app::state::AppState;
use crate::converters::Converter;
use core::fmt::{self, Debug, Display, Formatter};

use eframe::egui::{Color32, FontFamily, FontId, TextFormat, Ui};
use eframe::egui::text::LayoutJob;
use jaq_core::load::{Arena, File, Loader};
use jaq_core::{compile, Ctx, Native, Vars};
use jaq_json::{read, Val};
use std::rc::Rc;
use bytes::Bytes;
use jaq_core::data::JustLut;

#[derive(Clone, PartialEq)]
pub struct JqOptions {
    pub filter: String,
}

impl Default for JqOptions {
    fn default() -> Self {
        Self {
            filter: ".[]".to_string(),
        }
    }
}

type Filter = jaq_core::compile::Filter<Native<JustLut<Val>>>;

#[derive(Default)]
pub struct JqConverter {
    settings: Settings,
}

impl JqConverter {
    pub fn render(&mut self, ui: &mut Ui, state: &AppState) {
        if state.options.jq.filter.is_empty() {
            return;
        }

        let mut output = LayoutJob::default();

        match run(&state.options.jq.filter, &state.input, &self.settings, |value| {
            append_value(&mut output, &value, &self.settings);
        }) {
            Ok(()) => {
                ui.label(output);
            }
            Err(RunError::Compile(job)) => {
                ui.label(job);
            }
            Err(RunError::Parse(message)) => {
                ui.colored_label(Color32::ORANGE, format!("warn: parse error: {message}"));
            }
            Err(RunError::Jaq(error)) => {
                ui.colored_label(Color32::ORANGE, format!("warn: {error}"));
            }
        }
    }
}

impl Converter for JqConverter {
    fn convert(&mut self, state: &AppState) -> ConvertResult {
        if state.options.jq.filter.is_empty() {
            return ConvertResult::Empty;
        }

        let mut output = LayoutJob::default();

        match run(&state.options.jq.filter, &state.input, &self.settings, |value| {
            append_value(&mut output, &value, &self.settings);
        }) {
            Ok(()) => ConvertResult::RichText(output),
            Err(RunError::Compile(job)) => ConvertResult::RichText(job),
            Err(RunError::Parse(msg)) => ConvertResult::Error(format!("warn: parse error: {msg}")),
            Err(RunError::Jaq(error)) => ConvertResult::Error(format!("warn: {error}")),
        }
    }
}

// -----------------------------------------------------------------------------
// settings
// -----------------------------------------------------------------------------

#[derive(Debug)]
struct Settings {
    raw_input: bool,
    slurp: bool,
    null_input: bool,
    raw_output: bool,
    compact: bool,
    indent: usize,
    #[allow(unused)]
    tab: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            raw_input: false,
            slurp: false,
            null_input: false,
            raw_output: true,
            compact: false,
            indent: 4,
            tab: true,
        }
    }
}

// -----------------------------------------------------------------------------
// pipeline: compile + run
// -----------------------------------------------------------------------------

enum RunError {
    Compile(LayoutJob),
    Parse(String),
    Jaq(jaq_core::Error<Val>),
}

/// Compile and run in one go.
///
/// We can't easily name the concrete `Filter<F>` type here,
/// so we keep the whole pipeline inside a single function
/// and let the compiler infer `F` locally.
fn run(
    source: &str,
    input: &str,
    settings: &Settings,
    mut on_value: impl FnMut(Val),
) -> Result<(), RunError> {
    // ---- compile ----
    let arena = Arena::default();
    let loader = Loader::new(
        jaq_core::defs()
            .chain(jaq_std::defs())
            .chain(jaq_json::defs()),
    );

    let modules = loader
        .load(
            &arena,
            File {
                path: (),
                code: source,
            },
        )
        .map_err(|errors| RunError::Compile(format_load_errors(errors)))?;

    // We do NOT name `F`. `Compiler::default()` picks it, and we consume
    // the filter right away so nothing escapes.
    let filter: Filter = compile::Compiler::default()
        .with_funs(
            jaq_core::funs::<JustLut<Val>>()
                .chain(jaq_std::funs::<JustLut<Val>>())
                .chain(jaq_json::funs::<JustLut<Val>>()),
        )
        .compile(modules)
        .map_err(|errors| RunError::Compile(format_compile_errors(errors)))?;

    // ---- inputs ----
    let inputs: Box<dyn Iterator<Item = Result<Val, String>> + '_> = if settings.null_input {
        Box::new(core::iter::once(Ok(Val::Null)))
    } else {
        read_inputs(settings, input)
    };

    // ---- run ----
    for x in inputs {
        let x = x.map_err(RunError::Parse)?;

        let ctx = Ctx::<JustLut<Val>>::new(
            &filter.lut,
            Vars::<Val>::new([]),
        );

        for y in filter.id.run((ctx, x)).take(100_000) {
            let value = jaq_core::unwrap_valr(y).map_err(RunError::Jaq)?;
            on_value(value);
        }
    }

    Ok(())
}

// -----------------------------------------------------------------------------
// inputs
// -----------------------------------------------------------------------------

fn read_inputs<'a>(
    settings: &Settings,
    input: &'a str,
) -> Box<dyn Iterator<Item = Result<Val, String>> + 'a> {
    if settings.raw_input {
        Box::new(raw_input(settings.slurp, input).map(|s| Ok(text_val(s))))
    } else {
        collect_if(settings.slurp, json_slice(input.as_bytes()))
    }
}

fn raw_input(slurp: bool, input: &str) -> Box<dyn Iterator<Item = &str> + '_> {
    if slurp {
        Box::new(core::iter::once(input))
    } else {
        Box::new(input.lines())
    }
}

fn json_slice(slice: &[u8]) -> impl Iterator<Item = Result<Val, String>> + '_ {
    read::parse_many(slice).map(|value| value.map_err(|error| error.to_string()))
}

fn collect_if<'a>(
    slurp: bool,
    iter: impl Iterator<Item = Result<Val, String>> + 'a,
) -> Box<dyn Iterator<Item = Result<Val, String>> + 'a> {
    if slurp {
        // Terminate at the first Err (to prevent `collect` from hanging due to an infinite stream of Errs).
        let mut failed = false;
        let iter = iter.take_while(move |item| {
            if failed {
                return false;
            }
            failed = item.is_err();
            true
        });

        let collected: Result<Vec<Val>, String> = iter.collect();

        Box::new(core::iter::once(
            collected.map(|values| Val::Arr(Rc::new(values))),
        ))
    } else {
        Box::new(iter)
    }
}

fn text_val(s: &str) -> Val {
    // jaq-json 2.0: strings are `TStr(Box<Bytes>)`.
    Val::TStr(Box::new(Bytes::from(s.as_bytes().to_vec())))
}

// -----------------------------------------------------------------------------
// value formatter (plain text)
// -----------------------------------------------------------------------------

fn append_value(job: &mut LayoutJob, value: &Val, settings: &Settings) {
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

#[derive(Clone, Copy)]
enum JqColor {
    Plain,
    Null,
    Boolean,
    Number,
    Key,
    String,
}

fn append_colored_value(job: &mut LayoutJob, value: &Val, settings: &Settings, level: usize) {
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

fn json_key(value: &Val) -> String {
    match value {
        Val::TStr(bytes) | Val::BStr(bytes) => json_string(bytes),
        other => other.to_string(),
    }
}

fn json_string(bytes: &Bytes) -> String {
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

#[allow(unused)]
struct PpOpts {
    compact: bool,
    indent: String,
}

impl PpOpts {
    #[allow(unused)]
    fn indent(&self, f: &mut Formatter, level: usize) -> fmt::Result {
        if !self.compact {
            write!(f, "{}", self.indent.repeat(level))?;
        }
        Ok(())
    }

    #[allow(unused)]
    fn newline(&self, f: &mut Formatter) -> fmt::Result {
        if !self.compact {
            writeln!(f)?;
        }
        Ok(())
    }
}

#[allow(unused)]
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

#[allow(unused)]
fn fmt_seq<T, I, F>(
    f: &mut Formatter,
    opts: &PpOpts,
    level: usize,
    values: I,
    mut render: F,
) -> fmt::Result
where
    I: IntoIterator<Item = T>,
    F: FnMut(&mut Formatter, T) -> fmt::Result,
{
    opts.newline(f)?;

    let mut values = values.into_iter().peekable();

    while let Some(value) = values.next() {
        opts.indent(f, level + 1)?;
        render(f, value)?;

        if values.peek().is_some() {
            write!(f, ",")?;
        }

        opts.newline(f)?;
    }

    opts.indent(f, level)
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

fn span(f: &mut Formatter, class: &str, value: impl Display) -> fmt::Result {
    write!(f, "<span class=\"{class}\">{value}</span>")
}


fn color_from_hex(hex: &str) -> Color32 {
    let Some(hex) = hex.strip_prefix('#') else {
        return Color32::GRAY;
    };

    if hex.len() != 6 {
        return Color32::GRAY;
    }

    let parse_component = |range| u8::from_str_radix(&hex[range], 16).ok();

    match (
        parse_component(0..2),
        parse_component(2..4),
        parse_component(4..6),
    ) {
        (Some(red), Some(green), Some(blue)) => Color32::from_rgb(red, green, blue),
        _ => Color32::GRAY,
    }
}

fn visible_indentation(text: &str, tab_width: usize) -> String {
    const WORD_JOINER: char = '\u{2060}';

    let mut rendered = String::with_capacity(text.len());
    let mut at_line_start = true;
    let mut inserted_guard = false;

    for character in text.chars() {
        match character {
            '\n' => {
                rendered.push('\n');
                at_line_start = true;
                inserted_guard = false;
            }
            '\t' if at_line_start => {
                if !inserted_guard {
                    rendered.push(WORD_JOINER);
                    inserted_guard = true;
                }
                rendered.extend(std::iter::repeat_n(' ', tab_width));
            }
            ' ' if at_line_start => {
                if !inserted_guard {
                    rendered.push(WORD_JOINER);
                    inserted_guard = true;
                }
                rendered.push(' ');
            }
            character => {
                rendered.push(character);
                at_line_start = false;
            }
        }
    }

    rendered
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

fn append_diagnostic_text(job: &mut LayoutJob, text: &str, color: Color32) {
    let text = visible_indentation(text, 4);
    append_layout_text(job, &text, color);
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

#[derive(Debug)]
struct Report {
    message: String,
    labels: Vec<(core::ops::Range<usize>, StringColors, DiagnosticColor)>,
}

fn format_load_errors(errors: jaq_core::load::Errors<&str, ()>) -> LayoutJob {
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

fn format_compile_errors(errors: compile::Errors<&str, ()>) -> LayoutJob {
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

use jaq_core::load::lex;

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
// misc
// -----------------------------------------------------------------------------

struct FormatterFn<F>(F);

impl<F> Display for FormatterFn<F>
where
    F: Fn(&mut Formatter) -> fmt::Result,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0(f)
    }
}

impl<F> Debug for FormatterFn<F>
where
    F: Fn(&mut Formatter) -> fmt::Result,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0(f)
    }
}

// fn fmt_plain_val(f: &mut Formatter, opts: &PpOpts, level: usize, value: &Val) -> fmt::Result {
//     match value {
//         Val::Null => write!(f, "null"),
//         Val::Bool(value) => write!(f, "{value}"),
//         Val::Num(value) => write!(f, "{value}"),
//         Val::TStr(bytes) | Val::BStr(bytes) => write_bytes_as_json_string(f, bytes),
//         Val::Arr(values) if values.is_empty() => write!(f, "[]"),
//         Val::Arr(values) => {
//             write!(f, "[")?;
//             fmt_seq(f, opts, level, values.iter(), |f, value| {
//                 fmt_plain_val(f, opts, level + 1, value)
//             })?;
//             write!(f, "]")
//         }
//         Val::Obj(values) if values.is_empty() => write!(f, "{{}}"),
//         Val::Obj(values) => {
//             write!(f, "{{")?;
//             fmt_seq(f, opts, level, values.iter(), |f, (key, value)| {
//                 fmt_plain(f, key)?;
//                 write!(f, ":")?;
//
//                 if !opts.compact {
//                     write!(f, " ")?;
//                 }
//
//                 fmt_plain_val(f, opts, level + 1, value)
//             })?;
//             write!(f, "}}")
//         }
//     }
// }