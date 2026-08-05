use jaq_fmts::write::{self, Writer};
use jaq_fmts::Format;
use jaq_json::write::Pp;
use jaq_json::Val;
use taplo::dom::node::DomNode;
use taplo::parser::parse;

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

fn is_json_value_stream(input: &str) -> bool {
    let mut value_count = 0;

    for value in serde_json::Deserializer::from_str(input).into_iter::<serde_json::Value>() {
        if value.is_err() {
            return false;
        }

        value_count += 1;
    }

    value_count > 0
}

pub fn parse_yaml(input: &str) -> Result<Val, String> {
    if is_json_value_stream(input) {
        return Err(
            "JSON or JSON Lines input is not accepted when the input format is YAML; select JSON instead."
                .to_owned(),
        );
    }

    serde_saphyr::from_str::<Val>(input).map_err(|error| error.to_string())
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

