use quick_xml::events::{BytesCData, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer, XmlVersion};
use serde_json::{Map, Number, Value};
use indexmap::IndexMap;

const ATTRIBUTE_PREFIX: &str = "@";
const TEXT_KEY: &str = "#text";
const CDATA_KEY: &str = "#cdata";

#[derive(Default)]
struct Element {
    name: String,
    attributes: IndexMap<String, String>,
    children: IndexMap<String, Vec<Value>>,
    text: String,
    cdata: Vec<String>,
}

pub fn xml_to_json(input: &str) -> Result<Value, String> {
    validate_utf8_declaration(input)?;

    let mut reader = Reader::from_str(input);
    reader.config_mut().trim_text(false);

    let mut xml_version = XmlVersion::Explicit1_0;
    let mut stack = Vec::<Element>::new();
    let mut root = None;

    loop {
        match reader.read_event().map_err(|error| error.to_string())? {
            Event::Decl(declaration) => {
                xml_version = declaration
                    .xml_version()
                    .map_err(|error| error.to_string())?;
            }
            Event::Start(start) => {
                stack.push(element_from_start(&reader, &start, xml_version)?);
            }
            Event::Empty(start) => {
                let element = element_from_start(&reader, &start, xml_version)?;
                let (name, value) = element_to_json(element);

                if let Some(parent) = stack.last_mut() {
                    parent.children.entry(name).or_default().push(value);
                } else if root.replace((name, value)).is_some() {
                    return Err("XML input contains more than one root element".to_owned());
                }
            }
            Event::Text(text) => {
                if let Some(element) = stack.last_mut() {
                    element.text.push_str(
                        &text
                            .xml_content(xml_version)
                            .map_err(|error| error.to_string())?,
                    );
                }
            }
            Event::CData(cdata) => {
                if let Some(element) = stack.last_mut() {
                    element.cdata.push(
                        cdata
                            .decode()
                            .map_err(|error| error.to_string())?
                            .into_owned(),
                    );
                }
            }
            Event::End(_) => {
                let element = stack
                    .pop()
                    .ok_or_else(|| "XML input has an unexpected closing tag".to_owned())?;
                let (name, value) = element_to_json(element);

                if let Some(parent) = stack.last_mut() {
                    parent.children.entry(name).or_default().push(value);
                } else if root.replace((name, value)).is_some() {
                    return Err("XML input contains more than one root element".to_owned());
                }
            }
            Event::Comment(_) | Event::PI(_) | Event::DocType(_) => {}
            Event::Eof => break,
            _ => {}
        }
    }

    if !stack.is_empty() {
        return Err("XML input has unclosed elements".to_owned());
    }

    let (name, value) = root.ok_or_else(|| "XML input has no root element".to_owned())?;
    Ok(Value::Object(Map::from_iter([(name, value)])))
}

pub fn json_to_xml(value: &Value) -> Result<String, String> {
    let Value::Object(root) = value else {
        return Err("JSON-to-XML conversion requires an object with one root element".to_owned());
    };

    if root.len() != 1 {
        return Err("JSON-to-XML conversion requires exactly one root element".to_owned());
    }

    let (name, value) = root
        .iter()
        .next()
        .expect("a one-entry object always has its entry");

    validate_xml_name(name)?;

    let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);
    write_element(&mut writer, name, value)?;

    String::from_utf8(writer.into_inner())
        .map_err(|error| format!("XML output is not UTF-8: {error}"))
}

fn element_from_start(
    reader: &Reader<&[u8]>,
    start: &BytesStart<'_>,
    xml_version: XmlVersion
) -> Result<Element, String> {
    let name = std::str::from_utf8(start.name().as_ref())
        .map_err(|error| error.to_string())?
        .to_owned();

    let mut attributes = IndexMap::new();

    for attribute in start.attributes().with_checks(false) {
        let attribute = attribute.map_err(|error| error.to_string())?;
        let name = std::str::from_utf8(attribute.key.as_ref())
            .map_err(|error| error.to_string())?
            .to_owned();
        let value = attribute
            .decoded_and_normalized_value(xml_version, reader.decoder())
            .map_err(|error| error.to_string())?
            .into_owned();

        attributes.insert(name, value);
    }

    Ok(Element {
        name,
        attributes,
        ..Default::default()
    })
}

fn element_to_json(mut element: Element) -> (String, Value) {
    if !element.children.is_empty() && element.text.trim().is_empty() {
        element.text.clear();
    }

    if element.attributes.is_empty()
        && element.children.is_empty()
        && element.cdata.is_empty()
    {
        return (element.name, Value::String(element.text));
    }

    let mut object = Map::new();

    for (name, value) in element.attributes {
        object.insert(format!("{ATTRIBUTE_PREFIX}{name}"), Value::String(value));
    }

    if !element.text.is_empty() {
        object.insert(TEXT_KEY.to_owned(), Value::String(element.text));
    }

    match element.cdata.len() {
        0 => {}
        1 => {
            object.insert(
                CDATA_KEY.to_owned(),
                Value::String(element.cdata.into_iter().next().unwrap_or_default()),
            );
        }
        _ => {
            object.insert(
                CDATA_KEY.to_owned(),
                Value::Array(element.cdata.into_iter().map(Value::String).collect()),
            );
        }
    }

    for (name, mut values) in element.children {
        let value = if values.len() == 1 {
            values.pop().unwrap_or(Value::Null)
        } else {
            Value::Array(values)
        };

        object.insert(name, value);
    }

    (element.name, Value::Object(object))
}

fn write_element(
    writer: &mut Writer<Vec<u8>>,
    name: &str,
    value: &Value,
) -> Result<(), String> {
    validate_xml_name(name)?;

    match value {
        Value::Null => {
            writer
                .write_event(Event::Empty(BytesStart::new(name)))
                .map_err(|error| error.to_string())?;
        }
        Value::Bool(_) | Value::Number(_) | Value::String(_) => {
            writer
                .write_event(Event::Start(BytesStart::new(name)))
                .map_err(|error| error.to_string())?;
            write_text(writer, &scalar_to_string(value))?;
            writer
                .write_event(Event::End(BytesEnd::new(name)))
                .map_err(|error| error.to_string())?;
        }
        Value::Array(_) => {
            return Err(format!(
                "root element `{name}` cannot be an array; wrap it in an object"
            ));
        }
        Value::Object(object) => write_object_element(writer, name, object)?,
    }

    Ok(())
}

fn write_object_element(
    writer: &mut Writer<Vec<u8>>,
    name: &str,
    object: &Map<String, Value>,
) -> Result<(), String> {
    let mut start = BytesStart::new(name);

    for (key, value) in object {
        if let Some(attribute_name) = key.strip_prefix(ATTRIBUTE_PREFIX) {
            validate_xml_name(attribute_name)?;
            start.push_attribute((attribute_name, scalar_to_string(value).as_str()));
        }
    }

    writer
        .write_event(Event::Start(start))
        .map_err(|error| error.to_string())?;

    if let Some(text) = object.get(TEXT_KEY) {
        write_text(writer, &scalar_to_string(text))?;
    }

    if let Some(cdata) = object.get(CDATA_KEY) {
        match cdata {
            Value::Array(values) => {
                for value in values {
                    write_cdata(writer, &scalar_to_string(value))?;
                }
            }
            value => write_cdata(writer, &scalar_to_string(value))?,
        }
    }

    for (key, value) in object {
        if key.starts_with(ATTRIBUTE_PREFIX) || key == TEXT_KEY || key == CDATA_KEY {
            continue;
        }

        match value {
            Value::Array(values) => {
                for value in values {
                    write_element(writer, key, value)?;
                }
            }
            value => write_element(writer, key, value)?,
        }
    }

    writer
        .write_event(Event::End(BytesEnd::new(name)))
        .map_err(|error| error.to_string())?;

    Ok(())
}

fn write_text(writer: &mut Writer<Vec<u8>>, text: &str) -> Result<(), String> {
    writer
        .write_event(Event::Text(BytesText::new(text)))
        .map_err(|error| error.to_string())
}

fn write_cdata(writer: &mut Writer<Vec<u8>>, text: &str) -> Result<(), String> {
    for section in text.split("]]>") {
        writer
            .write_event(Event::CData(BytesCData::new(section)))
            .map_err(|error| error.to_string())?;

        if !section.is_empty() && text.ends_with(section) {
            break;
        }

        if !section.is_empty() || text.contains("]]>") {
            writer
                .write_event(Event::CData(BytesCData::new("]]")))
                .map_err(|error| error.to_string())?;
            writer
                .write_event(Event::CData(BytesCData::new(">")))
                .map_err(|error| error.to_string())?;
        }
    }

    Ok(())
}

fn scalar_to_string(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(value) => value.to_string(),
        Value::Number(Number { .. }) => value.to_string(),
        Value::String(value) => value.clone(),
        Value::Array(_) | Value::Object(_) => value.to_string(),
    }
}

fn validate_utf8_declaration(input: &str) -> Result<(), String> {
    let trimmed = input.trim_start_matches('\u{feff}').trim_start();

    let Some(declaration) = trimmed
        .strip_prefix("<?xml")
        .and_then(|rest| rest.split_once("?>"))
        .map(|(declaration, _)| declaration)
    else {
        return Ok(());
    };

    let lower = declaration.to_ascii_lowercase();

    if let Some(index) = lower.find("encoding") {
        let encoding = declaration[index + "encoding".len()..]
            .trim_start()
            .strip_prefix('=')
            .map(str::trim_start)
            .and_then(|value| {
                value
                    .strip_prefix('"')
                    .and_then(|value| value.split_once('"').map(|(value, _)| value))
                    .or_else(|| {
                        value
                            .strip_prefix('\'')
                            .and_then(|value| value.split_once('\'').map(|(value, _)| value))
                    })
            })
            .ok_or_else(|| "XML declaration has an invalid encoding attribute".to_owned())?;

        if !encoding.eq_ignore_ascii_case("utf-8") && !encoding.eq_ignore_ascii_case("utf8") {
            return Err(format!(
                "XML encoding `{encoding}` is not supported; use UTF-8 input"
            ));
        }
    }

    Ok(())
}

fn validate_xml_name(name: &str) -> Result<(), String> {
    if name.is_empty()
        || name.starts_with(|character: char| character.is_ascii_digit())
        || name.contains(|character: char| {
        character.is_ascii_whitespace()
            || matches!(character, '<' | '>' | '"' | '\'' | '&' | '/' | '=')
    })
    {
        return Err(format!("`{name}` is not a valid XML element or attribute name"));
    }

    Ok(())
}