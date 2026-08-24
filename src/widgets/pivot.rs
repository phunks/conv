use std::sync::Arc;
use eframe::egui::{Align, ComboBox, Frame, Id, Label, Layout, ScrollArea, Sense, Ui, Window};
use polars::frame::PivotColumnNaming;
use polars::prelude::{element, Column as PolarsColumn, DataFrame, IntoLazy, PlSmallStr, Selector};
use crate::widgets::spreadsheet::{parse_filter_number, CsvTable, SpreadsheetOptions};

pub(crate) const MAX_PIVOT_OUTPUT_COLUMNS: usize = 512;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum PivotAggregation {
    #[default]
    Count,
    Sum,
    Average,
    Minimum,
    Maximum,
}

impl PivotAggregation {
    const fn label(self) -> &'static str {
        match self {
            Self::Count => "Count",
            Self::Sum => "Sum",
            Self::Average => "Average",
            Self::Minimum => "Minimum",
            Self::Maximum => "Maximum",
        }
    }

    const fn requires_numeric_values(self) -> bool {
        !matches!(self, Self::Count)
    }
}

#[derive(Clone, Default, PartialEq, Eq)]
pub(crate) struct PivotConfig {
    pub(crate) rows: Vec<usize>,
    pub(crate) columns: Option<usize>,
    pub(crate) value: Option<usize>,
    aggregation: PivotAggregation,
}

pub(crate) fn pivot_csv_table(source: &CsvTable, config: &PivotConfig) -> Result<CsvTable, String> {
    let row_names = config
        .rows
        .iter()
        .map(|column| source.headers[*column].as_str())
        .collect::<Vec<_>>();
    let column_name = source.headers[config.columns.expect("validated by caller")].as_str();
    let value_column = config.value.expect("validated by caller");
    let value_name = source.headers[value_column].as_str();

    let mut columns = Vec::with_capacity(source.headers.len());

    for (column_index, header) in source.headers.iter().enumerate() {
        if column_index == value_column && config.aggregation.requires_numeric_values() {
            let values = source
                .rows
                .iter()
                .enumerate()
                .map(|(row_index, row)| {
                    let cell = row[column_index].trim();

                    if cell.is_empty() {
                        return Ok(None);
                    }

                    parse_filter_number(cell).map(Some).ok_or_else(|| {
                        format!(
                            "row {}: {header:?} contains a non-numeric value: {:?}",
                            row_index + 1,
                            row[column_index]
                        )
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;

            columns.push(PolarsColumn::new(header.clone().into(), values));
        } else {
            let values = source
                .rows
                .iter()
                .map(|row| row[column_index].clone())
                .collect::<Vec<_>>();

            columns.push(PolarsColumn::new(header.clone().into(), values));
        }
    }

    let dataframe = DataFrame::new(source.rows.len(), columns)
        .map_err(|error| error.to_string())?;

    let on_columns = source
        .rows
        .iter()
        .map(|row| row[config.columns.expect("validated by caller")].clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    if on_columns.len() > MAX_PIVOT_OUTPUT_COLUMNS {
        return Err(format!(
            "pivot would create {} columns; the limit is {MAX_PIVOT_OUTPUT_COLUMNS}",
            on_columns.len()
        ));
    }

    let on_columns_height = on_columns.len();
    let on_columns = DataFrame::new(
        on_columns_height,
        vec![PolarsColumn::new("".into(), on_columns)],
    )
        .map_err(|error| error.to_string())?;

    let rows = Selector::ByName {
        names: row_names
            .iter()
            .map(|name| PlSmallStr::from(*name))
            .collect::<Vec<_>>()
            .into(),
        strict: true,
    };
    let columns = Selector::ByName {
        names: vec![PlSmallStr::from(column_name)].into(),
        strict: true,
    };
    let values = Selector::ByName {
        names: vec![PlSmallStr::from(value_name)].into(),
        strict: true,
    };

    let aggregation = match config.aggregation {
        PivotAggregation::Count => element().count(),
        PivotAggregation::Sum => element().sum(),
        PivotAggregation::Average => element().mean(),
        PivotAggregation::Minimum => element().min(),
        PivotAggregation::Maximum => element().max(),
    };

    let pivoted = dataframe
        .lazy()
        .pivot(
            columns,
            Arc::new(on_columns),
            rows,
            values,
            aggregation,
            true,
            "_".into(),
            PivotColumnNaming::Auto,
        )
        .collect()
        .map_err(|error| error.to_string())?;

    csv_table_from_dataframe(&pivoted)
}

fn csv_table_from_dataframe(dataframe: &DataFrame) -> Result<CsvTable, String> {
    let headers = dataframe
        .get_column_names()
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();

    let mut rows = Vec::with_capacity(dataframe.height());

    for row_index in 0..dataframe.height() {
        let mut row = Vec::with_capacity(dataframe.width());

        for column in dataframe.columns() {
            let value = column
                .get(row_index)
                .map_err(|error| error.to_string())?;

            let cell = match value {
                polars::prelude::AnyValue::Null => String::new(),
                polars::prelude::AnyValue::String(value) => value.to_owned(),
                polars::prelude::AnyValue::StringOwned(value) => value.to_string(),
                value => value.to_string(),
            };

            row.push(cell);
        }

        rows.push(row);
    }

    Ok(CsvTable {
        headers,
        rows,
        has_header: true,
    })
}

pub(crate) fn pivot_window_ui(ui: &mut Ui, options: &mut SpreadsheetOptions) {
    if !options.pivot_window_open {
        return;
    }

    let Some(source) = options.pivot_source.as_ref().or(options.table.as_ref()) else {
        options.pivot_window_open = false;
        return;
    };

    let headers = source.headers.clone();
    let mut open = true;
    let mut close_requested = false;
    let mut apply_requested = false;

    Window::new("Pivot table")
        .id(Id::new("spreadsheet.pivot.window"))
        .collapsible(false)
        .resizable(true)
        .default_width(760.0)
        .default_height(340.0)
        .open(&mut open)
        .show(ui.ctx(), |ui| {
            ui.label("Drag fields into Rows, Columns, and Values.");
            ui.weak("Rows accepts multiple fields. Columns and Values accept one field each.");

            ui.add_space(8.0);

            ui.columns(2, |columns| {
                let available_ui = &mut columns[0];
                available_ui.strong("Available fields");

                ScrollArea::vertical()
                    .id_salt("spreadsheet.pivot.available")
                    .max_height(210.0)
                    .show(available_ui, |ui| {
                        for (column, name) in headers.iter().enumerate() {
                            let field_id = Id::new(("spreadsheet.pivot.field", column));

                            ui.dnd_drag_source(field_id, column, |ui| {
                                ui.add_sized(
                                    [ui.available_width(), 24.0],
                                    Label::new(name).sense(Sense::drag()),
                                );
                            });
                        }
                    });

                let fields_ui = &mut columns[1];

                pivot_drop_zone_ui(
                    fields_ui,
                    "Rows",
                    "spreadsheet.pivot.rows",
                    &headers,
                    &mut options.pivot_config.rows,
                    true,
                );

                fields_ui.add_space(6.0);

                pivot_single_drop_zone_ui(
                    fields_ui,
                    "Columns",
                    "spreadsheet.pivot.columns",
                    &headers,
                    &mut options.pivot_config.columns,
                );

                fields_ui.add_space(6.0);

                pivot_single_drop_zone_ui(
                    fields_ui,
                    "Values",
                    "spreadsheet.pivot.values",
                    &headers,
                    &mut options.pivot_config.value,
                );

                normalize_pivot_fields(&mut options.pivot_config);
            });

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label("Aggregation:");

                ComboBox::from_id_salt("spreadsheet.pivot.aggregation")
                    .width(150.0)
                    .selected_text(options.pivot_config.aggregation.label())
                    .show_ui(ui, |ui| {
                        for aggregation in [
                            PivotAggregation::Count,
                            PivotAggregation::Sum,
                            PivotAggregation::Average,
                            PivotAggregation::Minimum,
                            PivotAggregation::Maximum,
                        ] {
                            ui.selectable_value(
                                &mut options.pivot_config.aggregation,
                                aggregation,
                                aggregation.label(),
                            );
                        }
                    });

                if options.pivot_config.aggregation.requires_numeric_values() {
                    ui.weak("Values must contain numbers.");
                }
            });

            if let Some(error) = &options.pivot_error {
                ui.add_space(6.0);
                ui.colored_label(ui.visuals().error_fg_color, error);
            }

            ui.add_space(12.0);

            ui.horizontal(|ui| {
                if ui.button("Clear fields").clicked() {
                    options.pivot_config.rows.clear();
                    options.pivot_config.columns = None;
                    options.pivot_config.value = None;
                    options.pivot_error = None;
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("Close").clicked() {
                        close_requested = true;
                    }

                    if ui.button("Apply pivot").clicked() {
                        apply_requested = true;
                    }
                });
            });
        });

    if apply_requested {
        if let Err(error) = options.apply_pivot() {
            options.pivot_error = Some(error);
        } else {
            close_requested = true;
        }
    }

    if close_requested {
        open = false;
    }

    if !open {
        options.pivot_window_open = false;
    }
}


fn normalize_pivot_fields(config: &mut PivotConfig) {
    let mut used = std::collections::HashSet::new();

    config.rows.retain(|column| used.insert(*column));

    if let Some(column) = config.columns
        && !used.insert(column)
    {
        config.columns = None;
    }

    if let Some(column) = config.value
        && !used.insert(column)
    {
        config.value = None;
    }
}

fn pivot_drop_zone_ui(
    ui: &mut Ui,
    title: &str,
    id: &'static str,
    headers: &[String],
    fields: &mut Vec<usize>,
    multiple: bool,
) {
    ui.strong(title);

    let frame = Frame::group(ui.style()).inner_margin(6.0);
    let (_, dropped_field) = ui.dnd_drop_zone::<usize, ()>(frame, |ui| {
        ui.set_min_height(42.0);

        if fields.is_empty() {
            ui.weak("Drop field here");
        }

        let mut remove = None;

        for (position, column) in fields.iter().copied().enumerate() {
            ui.horizontal(|ui| {
                let field_id = Id::new((id, "field", position, column));

                ui.dnd_drag_source(field_id, column, |ui| {
                    ui.label(headers.get(column).map(String::as_str).unwrap_or("Unknown field"));
                });

                if ui.small_button("×").on_hover_text("remove field").clicked() {
                    remove = Some(position);
                }
            });
        }

        if let Some(position) = remove {
            fields.remove(position);
        }
    });

    if let Some(column) = dropped_field {
        fields.retain(|field| *field != *column);

        if multiple {
            fields.push(*column);
        } else {
            fields.clear();
            fields.push(*column);
        }
    }
}

fn pivot_single_drop_zone_ui(
    ui: &mut Ui,
    title: &str,
    id: &'static str,
    headers: &[String],
    field: &mut Option<usize>,
) {
    let mut fields = field.iter().copied().collect::<Vec<_>>();

    pivot_drop_zone_ui(ui, title, id, headers, &mut fields, false);

    *field = fields.into_iter().next();
}