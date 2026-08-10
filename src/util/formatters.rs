use std::path::Path;
use crate::widgets::diff_lang::DiffLanguage;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormatDiagnostic {
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct FormattedInput {
    pub text: String,
    pub diagnostic: Option<FormatDiagnostic>,
}

pub struct FormattedInputs {
    pub left: FormattedInput,
    pub right: FormattedInput,
}

pub fn format_diff_input(language: DiffLanguage, source: &str) -> FormattedInput {
    let path = Path::new(language.virtual_path());

    let formatted = match language {
        DiffLanguage::Json => format_json(path, source),
        DiffLanguage::Xml => format_xml(source),
        DiffLanguage::JavaScript | DiffLanguage::TypeScript => {
            format_javascript_or_typescript(path, source)
        }
        DiffLanguage::Html => format_html(source),
        DiffLanguage::Css => format_css(source),
        DiffLanguage::Sql => Ok(format_sql(source)),
        DiffLanguage::Text
        | DiffLanguage::Yaml
        | DiffLanguage::Toml
        | DiffLanguage::Rust
        | DiffLanguage::Shell
        | DiffLanguage::Python
        | DiffLanguage::Go
        | DiffLanguage::Java
        | DiffLanguage::C
        | DiffLanguage::Cpp => Err("pretty print is not supported for this language".to_owned()),
    };

    match formatted {
        Ok(text) => FormattedInput {
            text,
            diagnostic: None,
        },
        Err(message) => FormattedInput {
            text: source.to_owned(),
            diagnostic: Some(parse_format_diagnostic(message)),
        },
    }
}


pub fn format_diff_inputs(
    language: DiffLanguage,
    left: &str,
    right: &str,
) -> FormattedInputs {
    FormattedInputs {
        left: format_diff_input(language, left),
        right: format_diff_input(language, right),
    }
}

fn parse_format_diagnostic(message: String) -> FormatDiagnostic {
    let expression = regex::Regex::new(
        r"(?:from|at) line (?<line>\d+), column (?<column>\d+)",
    )
        .expect("format diagnostic pattern must be valid");

    let location = expression.captures(&message).and_then(|captures| {
        Some((
            captures.name("line")?.as_str().parse::<usize>().ok()?,
            captures.name("column")?.as_str().parse::<usize>().ok()?,
        ))
    });

    FormatDiagnostic {
        message,
        line: location.map(|(line, _)| line),
        column: location.map(|(_, column)| column),
    }
}

fn format_sql(source: &str) -> String {
    sqlformat::format(
        source,
        &sqlformat::QueryParams::None,
        &sqlformat::FormatOptions::default(),
    )
}

fn format_json(path: &Path, source: &str) -> Result<String, String> {
    let config = dprint_plugin_json::configuration::ConfigurationBuilder::new()
        .indent_width(2)
        .build();

    dprint_plugin_json::format_text(path, source, &config)
        .map(|formatted| formatted.unwrap_or_else(|| source.to_owned()))
        .map_err(|error| error.to_string())
}

fn format_javascript_or_typescript(path: &Path, source: &str) -> Result<String, String> {
    let config = dprint_plugin_typescript::configuration::ConfigurationBuilder::new()
        .indent_width(2)
        .line_width(100)
        .build();

    dprint_plugin_typescript::format_text(
        dprint_plugin_typescript::FormatTextOptions {
            path,
            extension: None,
            text: source.into(),
            config: &config,
            external_formatter: None,
        },
    )
        .map(|formatted| formatted.unwrap_or_else(|| source.to_owned()))
        .map_err(|error| error.to_string())
}

fn format_xml(source: &str) -> Result<String, String> {
    format_markup(source, markup_fmt::Language::Xml)
}

fn format_html(source: &str) -> Result<String, String> {
    format_markup(source, markup_fmt::Language::Html)
}

fn format_markup(source: &str, language: markup_fmt::Language) -> Result<String, String> {
    markup_fmt::format_text(
        source,
        language,
        &markup_fmt::config::FormatOptions::default(),
        |code, _language| Ok(code.into()),
    )
        .map_err(|error| error.to_string())
}

fn format_css(source: &str) -> Result<String, String> {
    malva::format_text(
        source,
        malva::Syntax::Css,
        &malva::config::FormatOptions::default(),
    )
        .map_err(|error| error.to_string())
}