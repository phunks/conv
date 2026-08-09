use crate::app::result::ConvertResult;
use crate::app::state::AppState;
use crate::converters::Converter;
use core::fmt::{self, Debug, Formatter};
use eframe::egui::{FontFamily, FontId, TextFormat, Ui};
use eframe::egui::text::LayoutJob;
use jaq_core::load::{Arena, File, Loader};
use jaq_core::{compile, Ctx, Native, Vars};
use jaq_json::Val;
use std::rc::Rc;
use bytes::Bytes;
use jaq_core::data::JustLut;
use crate::converters::data::csv::CsvOutput;
use crate::converters::data::json::{append_value, format_compile_errors, format_load_errors, json_key, json_slice, json_string};
use crate::widgets::menu::{InputFormat, OutputFormat};
use crate::app::colors::{TEXT_MUTED, WARNING};

pub(crate) mod format;
pub(crate) mod csv;
pub(crate) mod json;
pub(crate) mod toml;

#[derive(Clone, PartialEq)]
pub struct DataOptions {
    pub filter: String,
    pub input_format: InputFormat,
    pub output_format: OutputFormat,
    /// combine parsed input values into one array
    pub slurp: bool,
    /// compact JSON output (`-c`)
    pub compact: bool,
}

impl Default for DataOptions {
    fn default() -> Self {
        Self {
            filter: ".".to_string(),
            input_format: InputFormat::Json,
            output_format: OutputFormat::Json,
            slurp: false,
            compact: false,
        }
    }
}

type Filter = jaq_core::compile::Filter<Native<JustLut<Val>>>;

#[derive(Default)]
pub struct DataFormatConverter {
    settings: Settings,
}

impl DataFormatConverter {
    pub fn render(&mut self, ui: &mut Ui, state: &AppState) {
        if state.options.data.filter.is_empty() {
            return;
        }

        let settings = self.settings_for(&state.options.data);

        let mut output = LayoutJob::default();
        let mut format_error = None;
        let mut csv_output = CsvOutput::default();

        match run(&state.options.data.filter, &state.input, &settings, |value| {
            match state.options.data.output_format {
                OutputFormat::Json => append_value(&mut output, &value, &settings),
                OutputFormat::Toml => match format::toml(&value) {
                    Ok(text) => append_data_text(&mut output, &text),
                    Err(error) => format_error = Some(error),
                },
                OutputFormat::Yaml => match format::yaml(&value) {
                    Ok(text) => append_data_text(&mut output, &text),
                    Err(error) => format_error = Some(error),
                },
                OutputFormat::Csv => match csv_output.write(&value) {
                    Ok(text) => append_data_text(&mut output, &text),
                    Err(error) => format_error = Some(error),
                },
            }
        }) {
            Ok(()) if let Some(error) = format_error => {
                ui.colored_label(WARNING, format!("warn: TOML output: {error}"));
            }
            Ok(()) => {
                ui.label(output);
            }
            Err(RunError::Compile(job)) => {
                ui.label(job);
            }
            Err(RunError::Parse(message)) => {
                ui.colored_label(WARNING, format!("warn: parse error: {message}"));
            }
            Err(RunError::Jaq(error)) => {
                ui.colored_label(WARNING, format!("warn: {error}"));
            }
        }
    }

    pub fn copy_output_text(&self, state: &AppState) -> Option<String> {
        if state.options.data.filter.is_empty() {
            return None;
        }

        let settings = self.settings_for(&state.options.data);
        let mut output = String::new();
        let mut format_error = None;
        let mut csv_output = CsvOutput::default();

        run(&state.options.data.filter, &state.input, &settings, |value| {
            match state.options.data.output_format {
                OutputFormat::Json => append_plain_value(&mut output, &value, &settings),
                OutputFormat::Toml => match format::toml(&value) {
                    Ok(text) => output.push_str(&text),
                    Err(error) => format_error = Some(error),
                },
                OutputFormat::Yaml => match format::yaml(&value) {
                    Ok(text) => output.push_str(&text),
                    Err(error) => format_error = Some(error),
                },
                OutputFormat::Csv => match csv_output.write(&value) {
                    Ok(text) => output.push_str(&text),
                    Err(error) => format_error = Some(error),
                },
            }
        })
            .ok()?;

        format_error.is_none().then_some(output).filter(|output| !output.is_empty())
    }

    fn settings_for(&self, options: &DataOptions) -> Settings {
        Settings {
            input_format: options.input_format,
            slurp: options.slurp,
            compact: options.compact,
            ..self.settings.clone()
        }
    }
}

impl Converter for DataFormatConverter {
    fn convert(&mut self, state: &AppState) -> ConvertResult {
        if state.options.data.filter.is_empty() {
            return ConvertResult::Empty;
        }

        let settings = self.settings_for(&state.options.data);
        let mut output = LayoutJob::default();

        match run(&state.options.data.filter, &state.input, &settings, |value| {
            append_value(&mut output, &value, &settings);
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

#[derive(Debug, Clone)]
pub(crate) struct Settings {
    input_format: InputFormat,
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
            input_format: InputFormat::Json,
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
    match settings.input_format {
        InputFormat::Json => {
            if settings.raw_input {
                Box::new(raw_input(settings.slurp, input).map(|text| Ok(text_val(text))))
            } else {
                collect_if(settings.slurp, json_slice(input.as_bytes()))
            }
        }
        InputFormat::Toml => {
            Box::new(core::iter::once(format::parse_toml(input)))
        }
        InputFormat::Yaml => {
            Box::new(core::iter::once(format::parse_yaml(input)))
        }
        InputFormat::Csv => {
            let rows = csv::parse_csv(input).into_iter();
            collect_if(settings.slurp, rows)
        }
    }
}

fn raw_input(slurp: bool, input: &str) -> Box<dyn Iterator<Item = &str> + '_> {
    if slurp {
        Box::new(core::iter::once(input))
    } else {
        Box::new(input.lines())
    }
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


fn append_plain_value(output: &mut String, value: &Val, settings: &Settings) {
    if !output.is_empty() {
        output.push('\n');
    }

    match value {
        Val::TStr(bytes) | Val::BStr(bytes) if settings.raw_output => {
            output.push_str(&String::from_utf8_lossy(bytes.as_ref()));
        }
        value => append_plain_formatted_value(output, value, settings, 0),
    }
}

fn append_plain_formatted_value(
    output: &mut String,
    value: &Val,
    settings: &Settings,
    level: usize,
) {
    match value {
        Val::Null => output.push_str("null"),
        Val::Bool(value) => output.push_str(&value.to_string()),
        Val::Num(value) => output.push_str(&value.to_string()),
        Val::TStr(bytes) | Val::BStr(bytes) => output.push_str(&json_string(bytes)),
        Val::Arr(values) if values.is_empty() => output.push_str("[]"),
        Val::Arr(values) => {
            output.push('[');

            if !settings.compact {
                output.push('\n');
            }

            let mut values = values.iter().peekable();

            while let Some(value) = values.next() {
                append_plain_indent(output, settings, level + 1);
                append_plain_formatted_value(output, value, settings, level + 1);

                if values.peek().is_some() {
                    output.push(',');
                }

                if !settings.compact {
                    output.push('\n');
                }
            }

            append_plain_indent(output, settings, level);
            output.push(']');
        }
        Val::Obj(values) if values.is_empty() => output.push_str("{}"),
        Val::Obj(values) => {
            output.push('{');

            if !settings.compact {
                output.push('\n');
            }

            let mut values = values.iter().peekable();

            while let Some((key, value)) = values.next() {
                append_plain_indent(output, settings, level + 1);
                output.push_str(&json_key(key));
                output.push(':');

                if !settings.compact {
                    output.push(' ');
                }

                append_plain_formatted_value(output, value, settings, level + 1);

                if values.peek().is_some() {
                    output.push(',');
                }

                if !settings.compact {
                    output.push('\n');
                }
            }

            append_plain_indent(output, settings, level);
            output.push('}');
        }
    }
}

fn append_plain_indent(output: &mut String, settings: &Settings, level: usize) {
    if !settings.compact && level > 0 {
        output.push_str(&" ".repeat(settings.indent.saturating_mul(level)));
    }
}

fn append_data_text(job: &mut LayoutJob, text: &str) {
    if text.is_empty() {
        return;
    }

    job.append(
        text,
        0.0,
        TextFormat {
            font_id: FontId::new(11.5, FontFamily::Monospace),
            color: TEXT_MUTED,
            ..Default::default()
        },
    );
}

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

fn visible_indentation(text: &str, tab_width: usize) -> String {
    let mut rendered = String::with_capacity(text.len());
    let mut at_line_start = true;

    for character in text.chars() {
        match character {
            '\n' => {
                rendered.push('\n');
                at_line_start = true;
            }
            '\t' if at_line_start => {
                rendered.extend(std::iter::repeat_n(' ', tab_width));
            }
            character => {
                rendered.push(character);
                at_line_start = false;
            }
        }
    }

    rendered
}
