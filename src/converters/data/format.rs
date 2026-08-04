use fancy_regex::Input;
use jaq_fmts::write::{self, Writer};
use jaq_fmts::Format;
use jaq_json::write::Pp;
use jaq_json::Val;
use strum::{EnumMessage, VariantArray};
use taplo::dom::node::DomNode;
use taplo::parser::parse;

// #[derive(Default, Copy, Clone, Debug, PartialEq, Eq, VariantArray, EnumMessage)]
// pub enum JqInputFormat {
//     #[default]
//     #[strum(message = "Auto")]
//     Auto,
//     #[strum(message = "JSON")]
//     Json,
//     #[strum(message = "YAML")]
//     Yaml,
//     #[strum(message = "TOML")]
//     Toml,
//     #[strum(message = "XML")]
//     Xml,
//     #[strum(message = "CSV")]
//     Csv,
//     #[strum(message = "TSV")]
//     Tsv,
//     #[strum(message = "CBOR Base64")]
//     CborBase64,
// }
//
// #[derive(Default, Copy, Clone, Debug, PartialEq, Eq, VariantArray, EnumMessage)]
// pub enum JqOutputFormat {
//     #[default]
//     #[strum(message = "JSON")]
//     Json,
//     #[strum(message = "YAML")]
//     Yaml,
//     #[strum(message = "TOML")]
//     Toml,
//     #[strum(message = "XML")]
//     Xml,
//     #[strum(message = "CSV")]
//     Csv,
//     #[strum(message = "TSV")]
//     Tsv,
//     #[strum(message = "CBOR Base64")]
//     CborBase64,
// }

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, VariantArray, EnumMessage)]
pub enum InputFormat {
    #[default]
    #[strum(message = "JSON")]
    Json,
    #[strum(message = "TOML")]
    Toml,
    #[strum(message = "YAML")]
    Yaml,
    #[strum(message = "CSV")]
    Csv,
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, VariantArray, EnumMessage)]
pub enum OutputFormat {
    #[default]
    #[strum(message = "JSON")]
    Json,
    #[strum(message = "TOML")]
    Toml,
    #[strum(message = "YAML")]
    Yaml,
    #[strum(message = "CSV")]
    Csv,
}

pub fn parse_toml(input: &str) -> Result<Val, String> {
    let parsed = parse(input);

    if !parsed.errors.is_empty() {
        let errors = parsed
            .errors
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n");

        return Err(errors);
    }

    let dom_root = parsed.into_dom();
    let dom_errors = dom_root.errors().read();

    if !dom_errors.is_empty() {
        let errors = dom_errors
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n");

        return Err(errors);
    }

    let json_value: serde_json::Value = serde_json::to_value(&dom_root)
        .map_err(|error| error.to_string())?;

    serde_json::from_value(json_value).map_err(|error| error.to_string())
}

pub fn parse_yaml(input: &str) -> Result<Val, String> {
    let json_value: serde_json::Value =
        rust_yaml::from_str(input).map_err(|error| error.to_string())?;

    serde_json::from_value(json_value).map_err(|error| error.to_string())
}



pub fn toml(value: &Val) -> Result<String, String> {
    let writer = Writer {
        format: Format::Toml,
        pp: Pp::default(),
        join: false,
    };
    let mut output = Vec::new();

    write::write(&mut output, &writer, value).map_err(|error| error.to_string())?;

    String::from_utf8(output).map_err(|error| error.to_string())
}

pub fn yaml(value: &Val) -> Result<String, String> {
    let writer = Writer {
        format: Format::Yaml,
        pp: Pp {
            indent: Some("  ".to_owned()),
            sep_space: true,
            ..Default::default()
        },
        // Omit YAML's `---` / `...` document markers.
        join: true,
    };
    let mut output = Vec::new();

    write::write(&mut output, &writer, value).map_err(|error| error.to_string())?;

    String::from_utf8(output).map_err(|error| error.to_string())
}

pub fn csv(value: &Val) -> Result<String, String> {
    let writer = Writer {
        format: Format::Csv,
        pp: Pp::default(),
        join: false,
    };
    let mut output = Vec::new();

    write::write(&mut output, &writer, value).map_err(|error| error.to_string())?;

    String::from_utf8(output).map_err(|error| error.to_string())
}
