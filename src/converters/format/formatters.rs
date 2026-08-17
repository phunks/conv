use std::path::Path;
use crate::app::result::ConvertResult;
use crate::converters::format::html_formatter;
use strum::{EnumMessage, VariantArray};
// use crate::converters::format::json_formatter::json_to_layout_job;

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


#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, VariantArray, EnumMessage)]
pub enum FormatLanguage {
    #[default]
    /// html
    #[strum(message = "HTML")]
    Html,
    // /// json
    // #[strum(message = "JSON")]
    // Json,
    // /// xml
    // #[strum(message = "XML")]
    // Xml,
    /// javascript
    #[strum(message = "JS")]
    JavaScript,
    /// typescript
    #[strum(message = "TS")]
    TypeScript,
    /// css
    #[strum(message = "CSS")]
    Css,
    /// sql
    #[strum(message = "SQL")]
    Sql,
}

impl FormatLanguage {
    pub const fn virtual_path(self) -> &'static str {
        match self {
            Self::Html => "input.html",
            // Self::Json => "input.json",
            // Self::Xml => "input.xml",
            Self::JavaScript => "input.js",
            Self::TypeScript => "input.ts",
            Self::Css => "input.css",
            Self::Sql => "input.sql",
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct FormatOptions {
    pub language: FormatLanguage,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            language: FormatLanguage::Html,
        }
    }
}

pub fn convert(input: &str, options: &FormatOptions) -> ConvertResult {
    let path = options.language.virtual_path();

    match format_path(path, input) {
        // Ok(formatted) if options.language == FormatLanguage::Json => {
        //     ConvertResult::RichText(json_to_layout_job(&formatted, true))
        // }
        Ok(formatted) => ConvertResult::Text(formatted),
        Err(error) => ConvertResult::Error(error),
    }
}

pub fn format_path(path: &str, source: &str) -> Result<String, String> {
    let path = Path::new(path);
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default();

    match extension {
        "html" | "htm" => Ok(html_formatter::lazy_format_html(source, "  ")),
        "json" => format_json(path, source),
        "xml" => format_xml(source),
        "js" | "mjs" | "cjs" | "ts" | "mts" | "cts" => {
            format_javascript_or_typescript(path, source)
        }
        "css" => format_css(source),
        "sql" => Ok(format_sql(source)),
        _ => Err(format!("no formatter is available for `.{extension}`")),
    }
}

