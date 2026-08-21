use eframe::egui::{self, ComboBox, ScrollArea, Ui};
use egui_extras::{Column, TableBuilder};
use jaq_json::Val;
use crate::converters::data::csv::parse_flexible_csv;

const SORT_KEY_SLOTS: usize = 3;

#[derive(Clone, PartialEq)]
pub struct SpreadsheetOptions {
    attempted_load: bool,
    has_header: bool,
    sort_keys: [Option<SortKey>; SORT_KEY_SLOTS],
    table: Option<CsvTable>,
    error: Option<String>,
}

impl Default for SpreadsheetOptions {
    fn default() -> Self {
        Self {
            attempted_load: false,
            has_header: true,
            sort_keys: [None; SORT_KEY_SLOTS],
            table: None,
            error: None,
        }
    }
}

#[derive(Clone, Default, PartialEq)]
struct CsvTable {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    has_header: bool,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum SortDirection {
    #[default]
    Ascending,
    Descending,
}

impl SortDirection {
    const fn label(self) -> &'static str {
        match self {
            Self::Ascending => "Ascending",
            Self::Descending => "Descending",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct SortKey {
    column: usize,
    direction: SortDirection,
}

impl CsvTable {

    fn csv_text(&self) -> Result<String, String> {
        let mut output = String::new();

        if self.has_header {
            append_csv_row(&mut output, &self.headers)?;
        }

        for row in &self.rows {
            append_csv_row(&mut output, row)?;
        }

        Ok(output)
    }

    fn sort_by_keys(&mut self, keys: &[SortKey]) {
        self.rows.sort_by(|left, right| {
            for key in keys {
                let left_value = &left[key.column];
                let right_value = &right[key.column];

                let order = match (left_value.is_empty(), right_value.is_empty()) {
                    (true, true) => std::cmp::Ordering::Equal,
                    (true, false) => std::cmp::Ordering::Greater,
                    (false, true) => std::cmp::Ordering::Less,
                    (false, false) => left_value.to_lowercase().cmp(&right_value.to_lowercase()),
                };

                let order = match key.direction {
                    SortDirection::Ascending => order,
                    SortDirection::Descending if left_value.is_empty() || right_value.is_empty() => {
                        order
                    }
                    SortDirection::Descending => order.reverse(),
                };

                if order != std::cmp::Ordering::Equal {
                    return order;
                }
            }

            std::cmp::Ordering::Equal
        });
    }

    fn sort_by_column(&mut self, column: usize, direction: SortDirection) {
        self.rows.sort_by(|left, right| {
            let left = &left[column];
            let right = &right[column];

            let order = match (left.is_empty(), right.is_empty()) {
                (true, true) => std::cmp::Ordering::Equal,
                (true, false) => std::cmp::Ordering::Greater,
                (false, true) => std::cmp::Ordering::Less,
                (false, false) => left.to_lowercase().cmp(&right.to_lowercase()),
            };

            match direction {
                SortDirection::Ascending => order,
                SortDirection::Descending if left.is_empty() || right.is_empty() => order,
                SortDirection::Descending => order.reverse(),
            }
        });
    }
}

impl SpreadsheetOptions {
    pub fn csv_text(&self) -> Result<String, String> {
        let table = self
            .table
            .as_ref()
            .ok_or_else(|| "no CSV table is loaded".to_owned())?;

        table.csv_text()
    }

    pub fn has_table(&self) -> bool {
        self.table.is_some()
    }

    pub fn open_csv(&mut self, csv: &str) {
        self.sort_keys = [None; SORT_KEY_SLOTS];
        self.load_csv(csv);
    }

    fn ensure_loaded(&mut self, input: &str) {
        if !self.attempted_load {
            self.load_csv(input);
        }
    }

    fn apply_sort(&mut self) {
        let column_count = self
            .table
            .as_ref()
            .map(|table| table.headers.len())
            .unwrap_or_default();

        let mut keys = Vec::with_capacity(SORT_KEY_SLOTS);

        for key in self.sort_keys.iter().flatten().copied() {
            if key.column < column_count && !keys.iter().any(|existing: &SortKey| {
                existing.column == key.column
            }) {
                keys.push(key);
            }
        }

        if let Some(table) = &mut self.table
            && !keys.is_empty()
        {
            table.sort_by_keys(&keys);
        }
    }

    fn sort_by_column(&mut self, column: usize, direction: SortDirection) {
        self.sort_keys = [
            Some(SortKey { column, direction }),
            None,
            None,
        ];
        self.apply_sort();
    }

    fn load_csv(&mut self, input: &str) {
        self.attempted_load = true;
        self.error = None;

        match parse_csv_table(input, self.has_header) {
            Ok(table) => self.table = Some(table),
            Err(error) => {
                self.table = None;
                self.error = Some(error);
            }
        }
    }
}

fn append_csv_row(output: &mut String, values: &[String]) -> Result<(), String> {
    let values = values.iter().cloned().map(Val::from).collect::<Vec<_>>();
    let row = crate::converters::data::csv::csv_row(&values)?;

    output.push_str(&row);
    Ok(())
}

pub fn spreadsheet_ui(ui: &mut Ui, options: &mut SpreadsheetOptions) {
    ui.horizontal(|ui| {
        let reload_requested = ui.button("Reload CSV").clicked();
        let header_changed = ui
            .checkbox(&mut options.has_header, "First row is header")
            .on_hover_text("treat the first CSV record as column names")
            .changed();

        if reload_requested || header_changed {
            if let Ok(csv) = options.csv_text() {
                options.load_csv(&csv);
            }
        }

        let headers = options
            .table
            .as_ref()
            .map(|table| table.headers.clone())
            .unwrap_or_default();

        if !headers.is_empty() {
            sort_menu_ui(ui, options, &headers);
        }

        if let Some(table) = &options.table {
            ui.label(format!(
                "{} rows × {} columns",
                table.rows.len(),
                table.headers.len()
            ));
        }
    });

    if let Some(error) = &options.error {
        ui.colored_label(
            ui.visuals().error_fg_color,
            format!("Spreadsheet could not load the input as CSV: {error}"),
        );
        return;
    }

    let Some(table) = &options.table else {
        return;
    };

    ui.add_space(4.0);

    let mut sort_request = None;
    let max_scroll_height = ui.available_height();

    ScrollArea::horizontal()
        .id_salt("spreadsheet.horizontal_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let mut builder = TableBuilder::new(ui)
                .id_salt("spreadsheet.table")
                .striped(true)
                .resizable(true)
                .auto_shrink([false, false])
                .max_scroll_height(max_scroll_height)
                .column(Column::exact(48.0));

            for _ in &table.headers {
                builder = builder.column(
                    Column::initial(160.0)
                        .at_least(96.0)
                        .clip(true)
                        .resizable(true),
                );
            }

            builder
                .header(24.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("#");
                    });

                    for (column, name) in table.headers.iter().enumerate() {
                        header.col(|ui| {
                            ui.horizontal(|ui| {
                                ui.strong(name);

                                ui.menu_button("⋮", |ui| {
                                    if ui.button("Sort ascending").clicked() {
                                        sort_request =
                                            Some((column, SortDirection::Ascending));
                                        ui.close_menu();
                                    }

                                    if ui.button("Sort descending").clicked() {
                                        sort_request =
                                            Some((column, SortDirection::Descending));
                                        ui.close_menu();
                                    }
                                })
                                    .response
                                    .on_hover_text("sort this column");
                            });
                        });
                    }
                })
                .body(|body| {
                    body.rows(24.0, table.rows.len(), |mut row| {
                        let row_index = row.index();

                        row.col(|ui| {
                            ui.weak((row_index + 1).to_string());
                        });

                        for value in &table.rows[row_index] {
                            row.col(|ui| {
                                ui.label(value);
                            });
                        }
                    });
                });
        });

    if let Some((column, direction)) = sort_request {
        options.sort_by_column(column, direction);
    }
}

fn sort_menu_ui(ui: &mut Ui, options: &mut SpreadsheetOptions, headers: &[String]) {
    ui.menu_button("Sort…", |ui| {
        ui.label("Sort rows by:");

        for slot in 0..SORT_KEY_SLOTS {
            let mut column = options.sort_keys[slot].map(|key| key.column);
            let mut direction = options.sort_keys[slot]
                .map(|key| key.direction)
                .unwrap_or_default();

            ui.horizontal(|ui| {
                ui.label(format!("{}.", slot + 1));

                ComboBox::from_id_salt(("spreadsheet.sort.column", slot))
                    .width(150.0)
                    .selected_text(
                        column
                            .and_then(|column| headers.get(column))
                            .map(String::as_str)
                            .unwrap_or("— none —"),
                    )
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut column, None, "— none —");

                        for (index, header) in headers.iter().enumerate() {
                            ui.selectable_value(&mut column, Some(index), header);
                        }
                    });

                ComboBox::from_id_salt(("spreadsheet.sort.direction", slot))
                    .width(110.0)
                    .selected_text(direction.label())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut direction,
                            SortDirection::Ascending,
                            SortDirection::Ascending.label(),
                        );
                        ui.selectable_value(
                            &mut direction,
                            SortDirection::Descending,
                            SortDirection::Descending.label(),
                        );
                    });
            });

            options.sort_keys[slot] = column.map(|column| SortKey { column, direction });
        }

        ui.separator();

        if ui.button("Apply sort").clicked() {
            options.apply_sort();
            ui.close_menu();
        }

        if ui.button("Clear criteria").clicked() {
            options.sort_keys = [None; SORT_KEY_SLOTS];
        }
    });
}

fn parse_csv_table(input: &str, has_header: bool) -> Result<CsvTable, String> {
    let records = parse_flexible_csv(input)?;

    let Some(first_row) = records.first() else {
        return Err("CSV input is empty".to_owned());
    };

    let column_count = first_row.len();

    let headers = if has_header {
        first_row.clone()
    } else {
        (1..=column_count)
            .map(|index| format!("Column {index}"))
            .collect()
    };

    let first_data_row = usize::from(has_header);
    let mut rows = Vec::with_capacity(records.len().saturating_sub(first_data_row));

    for (index, values) in records.into_iter().skip(first_data_row).enumerate() {
        let record_number = index + first_data_row + 1;

        if values.len() != column_count {
            return Err(format!(
                "CSV row {record_number} has {} columns, expected {column_count}",
                values.len()
            ));
        }

        rows.push(values);
    }

    Ok(CsvTable {
        headers,
        rows,
        has_header,
    })
}
