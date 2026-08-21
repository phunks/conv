use std::collections::HashSet;
use std::rc::Rc;
use jaq_fmts::{write, Format};
use jaq_fmts::write::Writer;
use jaq_json::Val;
use jaq_json::write::Pp;

#[derive(Default)]
pub struct CsvOutput {
    header: Option<Vec<Val>>,
    header_written: bool,
}

impl CsvOutput {
    pub fn write(&mut self, value: &Val) -> Result<String, String> {
        match value {
            Val::Arr(rows) => {
                let mut output = String::new();

                for (index, row) in rows.iter().enumerate() {
                    let text = self
                        .write(row)
                        .map_err(|error| format!("CSV row {}: {error}", index + 1))?;
                    output.push_str(&text);
                }

                Ok(output)
            }
            Val::Obj(fields) => {
                let header = match &self.header {
                    Some(header) => {
                        if fields.len() != header.len()
                            || header.iter().any(|key| {
                            !fields.iter().any(|(candidate, _)| candidate == key)
                        })
                        {
                            return Err("CSV output rows have different columns".to_owned());
                        }

                        header.clone()
                    }
                    None => {
                        let header = fields
                            .iter()
                            .map(|(key, _)| key.clone())
                            .collect::<Vec<_>>();
                        self.header = Some(header.clone());
                        header
                    }
                };

                let row = header
                    .iter()
                    .map(|key| {
                        fields
                            .iter()
                            .find_map(|(candidate, value)| (candidate == key).then(|| value.clone()))
                            .expect("validated CSV header key must be present in every row")
                    })
                    .collect::<Vec<_>>();

                let mut output = String::new();

                if !self.header_written {
                    output.push_str(&csv_row(&header)?);
                    self.header_written = true;
                }

                output.push_str(&csv_row(&row)?);
                Ok(output)
            }
            value => Err(format!("CSV output expects an object row or an array of rows, found: {value}")),
        }
    }
}

pub(crate) fn csv_row(values: &[Val]) -> Result<String, String> {
    let writer = Writer {
        format: Format::Csv,
        pp: Pp::default(),
        join: false,
    };
    let mut output = Vec::new();
    let value = Val::Arr(Rc::new(values.to_vec()));

    write::write(&mut output, &writer, &value).map_err(|error| error.to_string())?;

    String::from_utf8(output).map_err(|error| error.to_string())
}

pub fn parse_csv(input: &str) -> Vec<Result<Val, String>> {
    let rows = match parse_flexible_csv(input) {
        Ok(rows) => rows,
        Err(error) => return vec![Err(error)],
    };

    let Some(header) = rows.first() else {
        return Vec::new();
    };

    let header = match csv_header(header) {
        Ok(header) => header,
        Err(error) => return vec![Err(error)],
    };

    rows
        .into_iter()
        .skip(1)
        .map(|row| {
            let values = row.into_iter().map(Val::from).collect::<Vec<_>>();
            csv_record(&header, Val::Arr(Rc::new(values)))
        })
        .collect()
}

pub fn parse_flexible_csv(input: &str) -> Result<Vec<Vec<String>>, String> {
    let mut records = Vec::new();
    let mut row = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut quote_closed = false;
    let mut record_started = false;

    for character in input.chars() {
        if quoted {
            match character {
                '"' => {
                    quoted = false;
                    quote_closed = true;
                }
                character => field.push(character),
            }

            continue;
        }

        if quote_closed {
            match character {
                '"' => {
                    field.push('"');
                    quoted = true;
                    quote_closed = false;
                }
                ',' => {
                    row.push(std::mem::take(&mut field));
                    quote_closed = false;
                    record_started = true;
                }
                '\r' => {}
                '\n' => {
                    row.push(std::mem::take(&mut field));
                    records.push(std::mem::take(&mut row));
                    quote_closed = false;
                    record_started = false;
                }
                _ => {
                    return Err(
                        "unexpected character after closing quote in CSV field".to_owned(),
                    );
                }
            }

            continue;
        }

        match character {
            '"' if field.is_empty() => {
                quoted = true;
                record_started = true;
            }
            '"' => return Err("unexpected quote in unquoted CSV field".to_owned()),
            ',' => {
                row.push(std::mem::take(&mut field));
                record_started = true;
            }
            '\r' => {}
            '\n' => {
                row.push(std::mem::take(&mut field));
                records.push(std::mem::take(&mut row));
                record_started = false;
            }
            character => {
                field.push(character);
                record_started = true;
            }
        }
    }

    if quoted {
        return Err("CSV field has an unclosed quote".to_owned());
    }

    if record_started || !field.is_empty() || !row.is_empty() {
        row.push(field);
        records.push(row);
    }

    Ok(records)
}

fn csv_header(columns: &[String]) -> Result<Vec<String>, String> {
    let mut names = Vec::with_capacity(columns.len());
    let mut seen = HashSet::with_capacity(columns.len());

    for name in columns {
        if !seen.insert(name.clone()) {
            return Err(format!("CSV header contains duplicate column: {name}"));
        }

        names.push(name.clone());
    }

    Ok(names)
}

fn csv_record(header: &[String], row: Val) -> Result<Val, String> {
    let Val::Arr(values) = row else {
        return Err("CSV row must be an array".to_owned());
    };

    if values.len() != header.len() {
        return Err(format!(
            "CSV row has {} columns, expected {}",
            values.len(),
            header.len()
        ));
    }

    let entries = header
        .iter()
        .cloned()
        .zip(values.iter().cloned())
        .map(|(name, value)| (Val::from(name), value));

    Ok(Val::obj(entries.collect()))
}