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
        let Val::Obj(fields) = value else {
            return Err(format!("CSV output expects an object row, found: {value}"));
        };

        let header = fields.iter().map(|(key, _)| key.clone()).collect::<Vec<_>>();

        if let Some(expected) = &self.header {
            if expected != &header {
                return Err("CSV output rows have different columns".to_owned());
            }
        } else {
            self.header = Some(header.clone());
        }

        let row = fields.iter().map(|(_, value)| value.clone()).collect::<Vec<_>>();
        let mut output = String::new();

        if !self.header_written {
            output.push_str(&csv_row(&header)?);
            self.header_written = true;
        }

        output.push_str(&csv_row(&row)?);
        Ok(output)
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
    let rows = jaq_fmts::read::tabular::read_csv(input.bytes().map(Ok::<_, String>))
        .map(|row| row.map_err(|error| error.to_string()))
        .collect::<Result<Vec<_>, _>>();

    let rows = match rows {
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
        .map(|row| csv_record(&header, row))
        .collect()
}

fn csv_header(row: &Val) -> Result<Vec<String>, String> {
    let Val::Arr(columns) = row else {
        return Err("CSV header must be a row".to_owned());
    };

    let mut names = Vec::with_capacity(columns.len());
    let mut seen = HashSet::with_capacity(columns.len());

    for column in columns.iter() {
        let Val::TStr(bytes) = column else {
            return Err(format!("CSV header must contain strings, found: {column}"));
        };

        let name = String::from_utf8_lossy(bytes.as_ref()).into_owned();

        if !seen.insert(name.clone()) {
            return Err(format!("CSV header contains duplicate column: {name}"));
        }

        names.push(name);
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