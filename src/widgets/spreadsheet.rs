use eframe::egui::{self, ComboBox, ScrollArea, Ui};
use egui_extras::{Column, TableBuilder};
use jaq_json::Val;
use regex::Regex;
use crate::converters::data::csv::parse_flexible_csv;
use crate::widgets::pivot::{pivot_csv_table, pivot_window_ui, PivotConfig, MAX_PIVOT_OUTPUT_COLUMNS};

const SORT_KEY_SLOTS: usize = 3;

#[derive(Clone, PartialEq)]
pub struct SpreadsheetOptions {
    attempted_load: bool,
    has_header: bool,
    sort_keys: [Option<SortKey>; SORT_KEY_SLOTS],
    sort_error: Option<String>,
    filters: Vec<ColumnFilter>,
    active_filter_column: Option<usize>,
    filter_before_edit: Option<ColumnFilter>,
    pub(crate) table: Option<CsvTable>,
    pub(crate) pivot_source: Option<CsvTable>,
    pub(crate) pivot_config: PivotConfig,
    pub(crate) pivot_window_open: bool,
    pub(crate) pivot_error: Option<String>,
    error: Option<String>,
}

impl Default for SpreadsheetOptions {
    fn default() -> Self {
        Self {
            attempted_load: false,
            has_header: true,
            sort_keys: empty_sort_keys(),
            sort_error: None,
            filters: Vec::new(),
            active_filter_column: None,
            filter_before_edit: None,
            table: None,
            pivot_source: None,
            pivot_config: PivotConfig::default(),
            pivot_window_open: false,
            pivot_error: None,
            error: None,
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum FilterMode {
    #[default]
    Contains,
    Equals,
    IsEmpty,
    IsNotEmpty,
    NumberEquals,
    NumberGreaterThan,
    NumberGreaterThanOrEqual,
    NumberLessThan,
    NumberLessThanOrEqual,
}

impl FilterMode {
    const fn label(self) -> &'static str {
        match self {
            Self::Contains => "Text contains",
            Self::Equals => "Text equals",
            Self::IsEmpty => "Is empty",
            Self::IsNotEmpty => "Is not empty",
            Self::NumberEquals => "Number =",
            Self::NumberGreaterThan => "Number >",
            Self::NumberGreaterThanOrEqual => "Number ≥",
            Self::NumberLessThan => "Number <",
            Self::NumberLessThanOrEqual => "Number ≤",
        }
    }

    const fn needs_value(self) -> bool {
        !matches!(self, Self::IsEmpty | Self::IsNotEmpty)
    }

    const fn is_numeric(self) -> bool {
        matches!(
            self,
            Self::NumberEquals
                | Self::NumberGreaterThan
                | Self::NumberGreaterThanOrEqual
                | Self::NumberLessThan
                | Self::NumberLessThanOrEqual
        )
    }
}

#[derive(Clone, PartialEq)]
struct ColumnFilter {
    mode: FilterMode,
    value: String,
}

impl Default for ColumnFilter {
    fn default() -> Self {
        Self {
            mode: FilterMode::Contains,
            value: String::new(),
        }
    }
}

impl ColumnFilter {
    fn is_active(&self) -> bool {
        match self.mode {
            FilterMode::IsEmpty | FilterMode::IsNotEmpty => true,
            _ => !self.value.trim().is_empty(),
        }
    }

    fn matches(&self, cell: &str) -> bool {
        if !self.is_active() {
            return true;
        }

        match self.mode {
            FilterMode::Contains => cell
                .to_lowercase()
                .contains(&self.value.trim().to_lowercase()),
            FilterMode::Equals => cell.eq_ignore_ascii_case(self.value.trim()),
            FilterMode::IsEmpty => cell.trim().is_empty(),
            FilterMode::IsNotEmpty => !cell.trim().is_empty(),
            FilterMode::NumberEquals
            | FilterMode::NumberGreaterThan
            | FilterMode::NumberGreaterThanOrEqual
            | FilterMode::NumberLessThan
            | FilterMode::NumberLessThanOrEqual => {
                let Some(cell) = parse_filter_number(cell) else {
                    return false;
                };
                let Some(value) = parse_filter_number(&self.value) else {
                    return false;
                };

                match self.mode {
                    FilterMode::NumberEquals => cell == value,
                    FilterMode::NumberGreaterThan => cell > value,
                    FilterMode::NumberGreaterThanOrEqual => cell >= value,
                    FilterMode::NumberLessThan => cell < value,
                    FilterMode::NumberLessThanOrEqual => cell <= value,
                    FilterMode::Contains
                    | FilterMode::Equals
                    | FilterMode::IsEmpty
                    | FilterMode::IsNotEmpty => unreachable!("numeric filter mode was matched"),
                }
            }
        }
    }
}

pub(crate) fn parse_filter_number(value: &str) -> Option<f64> {
    let value = value.trim().replace(',', "");
    let value = value.parse::<f64>().ok()?;

    value.is_finite().then_some(value)
}

#[derive(Clone, Default, PartialEq)]
pub(crate) struct CsvTable {
    pub(crate) headers: Vec<String>,
    pub(crate) rows: Vec<Vec<String>>,
    pub(crate) has_header: bool,
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

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum CaptureSortMode {
    #[default]
    Number,
    Text,
}

impl CaptureSortMode {
    const fn label(self) -> &'static str {
        match self {
            Self::Number => "number",
            Self::Text => "text",
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
struct SortKey {
    column: usize,
    direction: SortDirection,
    capture_pattern: String,
    capture_mode: CaptureSortMode,
}

struct CompiledSortKey {
    column: usize,
    direction: SortDirection,
    capture_regex: Option<Regex>,
    capture_mode: CaptureSortMode,
}

fn empty_sort_keys() -> [Option<SortKey>; SORT_KEY_SLOTS] {
    std::array::from_fn(|_| None)
}

fn compile_sort_keys(keys: &[SortKey]) -> Result<Vec<CompiledSortKey>, String> {
    keys.iter()
        .map(|key| {
            let pattern = key.capture_pattern.trim();

            let capture_regex = if pattern.is_empty() {
                None
            } else {
                let regex = Regex::new(pattern)
                    .map_err(|error| format!("invalid sort capture regex `{pattern}`: {error}"))?;

                if regex.captures_len() < 2 {
                    return Err(format!(
                        "sort capture regex `{pattern}` must contain a capture group, such as `第(\\d+)章`"
                    ));
                }

                Some(regex)
            };

            Ok(CompiledSortKey {
                column: key.column,
                direction: key.direction,
                capture_regex,
                capture_mode: key.capture_mode,
            })
        })
        .collect()
}

fn sort_indicator(keys: &[Option<SortKey>], column: usize) -> Option<(usize, SortDirection)> {
    keys.iter()
        .flatten()
        .enumerate()
        .find(|(_, key)| key.column == column)
        .map(|(index, key)| (index + 1, key.direction))
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

    fn sort_by_keys(&mut self, keys: &[CompiledSortKey]) {
        self.rows.sort_by(|left, right| {
            for key in keys {
                let left_value = &left[key.column];
                let right_value = &right[key.column];

                let (order, reverse_for_descending) = match &key.capture_regex {
                    Some(regex) => match key.capture_mode {
                        CaptureSortMode::Number => {
                            compare_captured_numbers(left_value, right_value, regex)
                        }
                        CaptureSortMode::Text => {
                            compare_captured_text(left_value, right_value, regex)
                        }
                    },
                    None => {
                        let order = match (left_value.is_empty(), right_value.is_empty()) {
                            (true, true) => std::cmp::Ordering::Equal,
                            (true, false) => std::cmp::Ordering::Greater,
                            (false, true) => std::cmp::Ordering::Less,
                            (false, false) => compare_sort_values(left_value, right_value),
                        };

                        (order, !left_value.is_empty() && !right_value.is_empty())
                    }
                };

                let order = match key.direction {
                    SortDirection::Ascending => order,
                    SortDirection::Descending if reverse_for_descending => order.reverse(),
                    SortDirection::Descending => order,
                };

                if order != std::cmp::Ordering::Equal {
                    return order;
                }
            }

            std::cmp::Ordering::Equal
        });
    }

    #[allow(unused)]
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

fn compare_captured_numbers(
    left: &str,
    right: &str,
    regex: &Regex,
) -> (std::cmp::Ordering, bool) {
    let left_number = regex
        .captures(left)
        .and_then(|captures| captures.get(1))
        .and_then(|capture| parse_filter_number(capture.as_str()));

    let right_number = regex
        .captures(right)
        .and_then(|captures| captures.get(1))
        .and_then(|capture| parse_filter_number(capture.as_str()));

    match (left_number, right_number) {
        (Some(left), Some(right)) => (left.total_cmp(&right), true),
        (Some(_), None) => (std::cmp::Ordering::Less, false),
        (None, Some(_)) => (std::cmp::Ordering::Greater, false),
        (None, None) => (left.to_lowercase().cmp(&right.to_lowercase()), true),
    }
}

fn compare_captured_text(
    left: &str,
    right: &str,
    regex: &Regex,
) -> (std::cmp::Ordering, bool) {
    let left_text = regex
        .captures(left)
        .and_then(|captures| captures.get(1))
        .map(|capture| capture.as_str());

    let right_text = regex
        .captures(right)
        .and_then(|captures| captures.get(1))
        .map(|capture| capture.as_str());

    match (left_text, right_text) {
        (Some(left), Some(right)) => (left.to_lowercase().cmp(&right.to_lowercase()), true),
        (Some(_), None) => (std::cmp::Ordering::Less, false),
        (None, Some(_)) => (std::cmp::Ordering::Greater, false),
        (None, None) => (left.to_lowercase().cmp(&right.to_lowercase()), true),
    }
}

fn compare_sort_values(left: &str, right: &str) -> std::cmp::Ordering {
    match (
        left.trim().parse::<f64>(),
        right.trim().parse::<f64>(),
    ) {
        (Ok(left_number), Ok(right_number))
        if left_number.is_finite() && right_number.is_finite() =>
            {
                left_number.total_cmp(&right_number)
            }
        _ => left.to_lowercase().cmp(&right.to_lowercase()),
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

    pub(crate) fn cancel_active_filter(&mut self) -> bool {
        let Some(column) = self.active_filter_column.take() else {
            return false;
        };

        if let Some(filter) = self.filter_before_edit.take()
            && let Some(current_filter) = self.filters.get_mut(column)
        {
            *current_filter = filter;
        }

        true
    }

    pub fn open_csv(&mut self, csv: &str) {
        self.sort_keys = empty_sort_keys();
        self.sort_error = None;
        self.filters.clear();
        self.active_filter_column = None;
        self.filter_before_edit = None;
        self.pivot_source = None;
        self.pivot_config = PivotConfig::default();
        self.pivot_window_open = false;
        self.pivot_error = None;
        self.load_csv(csv);
    }

    fn reset_pivot(&mut self) {
        let Some(source) = self.pivot_source.take() else {
            return;
        };

        self.ensure_filter_slots(source.headers.len());
        self.table = Some(source);
        self.filters.fill(ColumnFilter::default());
        self.sort_keys = empty_sort_keys();
        self.sort_error = None;
        self.active_filter_column = None;
        self.filter_before_edit = None;
        self.pivot_config = PivotConfig::default();
        self.pivot_error = None;
    }

    pub(crate) fn apply_pivot(&mut self) -> Result<(), String> {
        let source = self
            .pivot_source
            .as_ref()
            .or(self.table.as_ref())
            .cloned()
            .ok_or_else(|| "no CSV table is loaded".to_owned())?;

        let row_columns = self.pivot_config.rows.clone();
        let column = self
            .pivot_config
            .columns
            .ok_or_else(|| "drag one field into Columns".to_owned())?;
        let value = self
            .pivot_config
            .value
            .ok_or_else(|| "drag one field into Values".to_owned())?;

        if row_columns.is_empty() {
            return Err("drag at least one field into Rows".to_owned());
        }

        let column_count = source.headers.len();

        if row_columns.iter().any(|column| *column >= column_count)
            || column >= column_count
            || value >= column_count
        {
            return Err("pivot field no longer exists in the source table".to_owned());
        }

        let mut fields = row_columns.clone();
        fields.push(column);
        fields.push(value);
        fields.sort_unstable();

        if fields.windows(2).any(|window| window[0] == window[1]) {
            return Err("a field may be used only once in a pivot".to_owned());
        }

        let output = pivot_csv_table(&source, &self.pivot_config)?;

        if output.headers.len() > MAX_PIVOT_OUTPUT_COLUMNS {
            return Err(format!(
                "pivot would create {} columns; the limit is {MAX_PIVOT_OUTPUT_COLUMNS}",
                output.headers.len()
            ));
        }

        if self.pivot_source.is_none() {
            self.pivot_source = self.table.clone();
        }

        self.ensure_filter_slots(output.headers.len());
        self.filters.fill(ColumnFilter::default());
        self.sort_keys = empty_sort_keys();
        self.sort_error = None;
        self.active_filter_column = None;
        self.filter_before_edit = None;
        self.table = Some(output);
        self.pivot_error = None;

        Ok(())
    }

    fn ensure_filter_slots(&mut self, column_count: usize) {
        self.filters.resize_with(column_count, ColumnFilter::default);
        self.filters.truncate(column_count);
    }

    fn visible_row_indices(&self) -> Vec<usize> {
        let Some(table) = &self.table else {
            return Vec::new();
        };

        table
            .rows
            .iter()
            .enumerate()
            .filter(|(_, row)| {
                self.filters
                    .iter()
                    .enumerate()
                    .all(|(column, filter)| filter.matches(&row[column]))
            })
            .map(|(index, _)| index)
            .collect()
    }

    #[allow(unused)]
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

        for key in self.sort_keys.iter().flatten().cloned() {
            if key.column < column_count
                && !keys.iter().any(|existing: &SortKey| existing.column == key.column)
            {
                keys.push(key);
            }
        }

        self.sort_error = None;

        let compiled_keys = match compile_sort_keys(&keys) {
            Ok(keys) => keys,
            Err(error) => {
                self.sort_error = Some(error);
                return;
            }
        };

        if let Some(table) = &mut self.table
            && !compiled_keys.is_empty()
        {
            table.sort_by_keys(&compiled_keys);
        }
    }

    fn sort_by_column(&mut self, column: usize, direction: SortDirection) {
        self.sort_keys = [
            Some(SortKey {
                column,
                direction,
                capture_pattern: String::new(),
                capture_mode: CaptureSortMode::Number,
            }),
            None,
            None,
        ];
        self.apply_sort();
    }

    fn load_csv(&mut self, input: &str) {
        self.attempted_load = true;
        self.error = None;
        self.sort_error = None;
        self.pivot_source = None;
        self.pivot_config = PivotConfig::default();
        self.pivot_window_open = false;
        self.pivot_error = None;

        match parse_csv_table(input, self.has_header) {
            Ok(table) => {
                self.ensure_filter_slots(table.headers.len());
                self.table = Some(table);
            }
            Err(error) => {
                self.table = None;
                self.filters.clear();
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

        if (reload_requested || header_changed)
            && let Ok(csv) = options.csv_text() {
                options.load_csv(&csv);
        }

        let has_table = options.table.is_some();

        if ui
            .add_enabled(has_table, egui::Button::new("Pivot…"))
            .on_hover_text("build a pivot table from the current CSV data")
            .clicked()
        {
            options.pivot_window_open = true;
            options.pivot_error = None;
        }

        if options.pivot_source.is_some()
            && ui
            .button("Reset pivot")
            .on_hover_text("restore the source table before pivoting")
            .clicked()
        {
            options.reset_pivot();
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
            let visible_row_count = options.visible_row_indices().len();
            let label = if options.pivot_source.is_some() {
                "Pivot result"
            } else {
                "Table"
            };

            ui.label(format!(
                "{label}: {visible_row_count} / {} rows × {} columns",
                table.rows.len(),
                table.headers.len()
            ));
        }
    });

    pivot_window_ui(ui, options);

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

    let visible_rows = options.visible_row_indices();

    ui.add_space(4.0);

    let mut sort_request = None;
    let mut filter_request = None;
    let max_scroll_height = ui.available_height();
    let sort_keys = options.sort_keys.clone();

    {
        let filters = &mut options.filters;

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

                                    let filter_active = filters[column].is_active();

                                    if filter_button(ui, column, filter_active).clicked() {
                                        filter_request = Some(column);
                                    }

                                    let sort_button_label =
                                        match sort_indicator(&sort_keys, column) {
                                            Some((priority, SortDirection::Ascending)) => {
                                                format!("⋮↑{priority}")
                                            }
                                            Some((priority, SortDirection::Descending)) => {
                                                format!("⋮↓{priority}")
                                            }
                                            None => "⋮".to_owned(),
                                        };

                                    ui.menu_button(sort_button_label, |ui| {
                                        if ui.button("Sort ascending").clicked() {
                                            sort_request = Some((column, SortDirection::Ascending));
                                            ui.close_menu();
                                        }

                                        if ui.button("Sort descending").clicked() {
                                            sort_request = Some((column, SortDirection::Descending));
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
                        body.rows(24.0, visible_rows.len(), |mut row| {
                            let visible_row_index = row.index();
                            let row_index = visible_rows[visible_row_index];
                            let values = &table.rows[row_index];

                            row.col(|ui| {
                                ui.weak((row_index + 1).to_string());
                            });

                            for value in values {
                                row.col(|ui| {
                                    ui.label(value);
                                });
                            }
                        });
                    });
            });
    }

    if let Some(column) = filter_request {
        options.active_filter_column = Some(column);
        options.filter_before_edit = Some(options.filters[column].clone());
    }

    if let Some((column, direction)) = sort_request {
        options.sort_by_column(column, direction);
    }

    filter_window_ui(ui, options);
}

fn filter_window_ui(ui: &mut Ui, options: &mut SpreadsheetOptions) {
    let Some(column) = options.active_filter_column else {
        return;
    };

    let Some(table) = &options.table else {
        options.active_filter_column = None;
        return;
    };

    let Some(header) = table.headers.get(column) else {
        options.active_filter_column = None;
        return;
    };

    let mut open = true;
    let mut close_requested = false;
    let filter = &mut options.filters[column];

    egui::Window::new(format!("Filter: {header}"))
        .id(egui::Id::new(("spreadsheet.filter.window", column)))
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .collapsible(false)
        .resizable(false)
        .default_width(260.0)
        .open(&mut open)
        .show(ui.ctx(), |ui| {
            ui.label("Match mode:");

            ComboBox::from_id_salt(("spreadsheet.filter.mode", column))
                .width(ui.available_width())
                .selected_text(filter.mode.label())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut filter.mode,
                        FilterMode::Contains,
                        "Text contains",
                    );
                    ui.selectable_value(
                        &mut filter.mode,
                        FilterMode::Equals,
                        "Text equals",
                    );

                    ui.separator();

                    ui.selectable_value(
                        &mut filter.mode,
                        FilterMode::IsEmpty,
                        "Is empty",
                    );
                    ui.selectable_value(
                        &mut filter.mode,
                        FilterMode::IsNotEmpty,
                        "Is not empty",
                    );

                    ui.separator();

                    ui.selectable_value(
                        &mut filter.mode,
                        FilterMode::NumberEquals,
                        "Number =",
                    );
                    ui.selectable_value(
                        &mut filter.mode,
                        FilterMode::NumberGreaterThan,
                        "Number >",
                    );
                    ui.selectable_value(
                        &mut filter.mode,
                        FilterMode::NumberGreaterThanOrEqual,
                        "Number ≥",
                    );
                    ui.selectable_value(
                        &mut filter.mode,
                        FilterMode::NumberLessThan,
                        "Number <",
                    );
                    ui.selectable_value(
                        &mut filter.mode,
                        FilterMode::NumberLessThanOrEqual,
                        "Number ≤",
                    );
                });

            if filter.mode.needs_value() {
                ui.add_space(6.0);

                let hint = if filter.mode.is_numeric() {
                    "Number, e.g. 1,234"
                } else {
                    "Filter text"
                };

                ui.add(
                    egui::TextEdit::singleline(&mut filter.value)
                        .id_salt(("spreadsheet.filter.value", column))
                        .hint_text(hint)
                        .desired_width(f32::INFINITY),
                );

                if filter.mode.is_numeric() {
                    ui.add_space(4.0);
                    ui.weak("Non-numeric cells are excluded");
                }
            }

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui.button("Clear filter").clicked() {
                    *filter = ColumnFilter::default();
                }

                if ui.button("Close").clicked() {
                    close_requested = true;
                }
            });
        });

    if close_requested {
        open = false;
    }

    if !open {
        options.active_filter_column = None;
        options.filter_before_edit = None;
    }
}

#[allow(unused)]
fn column_menu_ui(ui: &mut Ui, filters: &mut [ColumnFilter], column: usize) {
    let filter = &mut filters[column];
    let active = filter.is_active();

    ui.collapsing(if active { "Filter*" } else { "Filter" }, |ui| {
        ui.label("Match:");

        ui.selectable_value(&mut filter.mode, FilterMode::Contains, "Text contains");
        ui.selectable_value(&mut filter.mode, FilterMode::Equals, "Text equals");

        ui.separator();

        ui.selectable_value(&mut filter.mode, FilterMode::IsEmpty, "Is empty");
        ui.selectable_value(&mut filter.mode, FilterMode::IsNotEmpty, "Is not empty");

        ui.separator();

        ui.label("Number:");
        ui.selectable_value(&mut filter.mode, FilterMode::NumberEquals, "=");
        ui.selectable_value(&mut filter.mode, FilterMode::NumberGreaterThan, ">");
        ui.selectable_value(
            &mut filter.mode,
            FilterMode::NumberGreaterThanOrEqual,
            "≥",
        );
        ui.selectable_value(&mut filter.mode, FilterMode::NumberLessThan, "<");
        ui.selectable_value(
            &mut filter.mode,
            FilterMode::NumberLessThanOrEqual,
            "≤",
        );

        if filter.mode.needs_value() {
            let hint = if filter.mode.is_numeric() {
                "Number, e.g. 1,234"
            } else {
                "Filter text"
            };

            ui.add(
                egui::TextEdit::singleline(&mut filter.value)
                    .id_salt(("spreadsheet.filter.value", column))
                    .hint_text(hint)
                    .desired_width(180.0),
            );

            if filter.mode.is_numeric() {
                ui.weak("Non-numeric cells are excluded");
            }
        }

        if filter.is_active() && ui.button("Clear filter").clicked() {
            *filter = ColumnFilter::default();
        }
    });
}

fn sort_menu_ui(ui: &mut Ui, options: &mut SpreadsheetOptions, headers: &[String]) {
    ui.menu_button("Sort…", |ui| {
        ui.label("Sort rows by:");

        for slot in 0..SORT_KEY_SLOTS {
            let mut column = options.sort_keys[slot].as_ref().map(|key| key.column);
            let mut direction = options.sort_keys[slot]
                .as_ref()
                .map(|key| key.direction)
                .unwrap_or_default();
            let mut capture_pattern = options.sort_keys[slot]
                .as_ref()
                .map(|key| key.capture_pattern.clone())
                .unwrap_or_default();
            let mut capture_mode = options.sort_keys[slot]
                .as_ref()
                .map(|key| key.capture_mode)
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

            if column.is_some() {
                ui.horizontal(|ui| {
                    ComboBox::from_id_salt(("spreadsheet.sort.capture_mode", slot))
                        .width(60.0)
                        .selected_text(capture_mode.label())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut capture_mode,
                                CaptureSortMode::Number,
                                CaptureSortMode::Number.label(),
                            );
                            ui.selectable_value(
                                &mut capture_mode,
                                CaptureSortMode::Text,
                                CaptureSortMode::Text.label(),
                            );
                        });

                    ui.add(
                        egui::TextEdit::singleline(&mut capture_pattern)
                            .id_salt(("spreadsheet.sort.capture_pattern", slot))
                            .hint_text("Capture regex, e.g. 第(\\d+)章")
                            .desired_width(f32::INFINITY),
                    )
                        .on_hover_text(
                            "The first capture group is used for sorting. Leave blank for automatic sorting.",
                        );
                });
            }

            options.sort_keys[slot] = column.map(|column| SortKey {
                column,
                direction,
                capture_pattern,
                capture_mode,
            });

            ui.add_space(4.0);
        }

        if let Some(error) = &options.sort_error {
            ui.colored_label(ui.visuals().error_fg_color, error);
        }

        ui.separator();

        if ui.button("Apply sort").clicked() {
            options.apply_sort();

            if options.sort_error.is_none() {
                ui.close_menu();
            }
        }

        if ui.button("Clear criteria").clicked() {
            options.sort_keys = empty_sort_keys();
            options.sort_error = None;
        }

        ui.separator();

        if options.filters.iter().any(ColumnFilter::is_active)
            && ui.button("Clear all filters").clicked()
        {
            options.filters.fill(ColumnFilter::default());
        }
    });
}

fn normalize_csv_headers(source: &[String]) -> Vec<String> {
    let mut headers = Vec::with_capacity(source.len());

    for (index, header) in source.iter().enumerate() {
        let base_name = if header.trim().is_empty() {
            format!("Column {}", index + 1)
        } else {
            header.clone()
        };

        let mut name = base_name.clone();
        let mut duplicate_number = 2;

        while headers.contains(&name) {
            name = format!("{base_name} ({duplicate_number})");
            duplicate_number += 1;
        }

        headers.push(name);
    }

    headers
}

fn parse_csv_table(input: &str, has_header: bool) -> Result<CsvTable, String> {
    let records = parse_flexible_csv(input)?;

    let Some(first_row) = records.first() else {
        return Err("CSV input is empty".to_owned());
    };

    let column_count = first_row.len();

    let headers = if has_header {
        normalize_csv_headers(first_row)
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


fn filter_button(ui: &mut Ui, _column: usize, active: bool) -> egui::Response {
    let response = ui.add_sized(
        [20.0, 20.0],
        egui::Button::new("")
            .frame(false)
            .sense(egui::Sense::click()),
    );

    let color = if active {
        ui.visuals().selection.stroke.color
    } else if response.hovered() {
        ui.visuals().text_color()
    } else {
        ui.visuals().weak_text_color()
    };

    let rect = response.rect;
    let center = rect.center();
    let stroke = egui::Stroke::new(1.5_f32, color);

    ui.painter().line_segment(
        [
            egui::pos2(center.x - 6.0, center.y - 5.0),
            egui::pos2(center.x + 6.0, center.y - 5.0),
        ],
        stroke,
    );
    ui.painter().line_segment(
        [
            egui::pos2(center.x - 6.0, center.y - 5.0),
            egui::pos2(center.x - 2.0, center.y),
        ],
        stroke,
    );
    ui.painter().line_segment(
        [
            egui::pos2(center.x + 6.0, center.y - 5.0),
            egui::pos2(center.x + 2.0, center.y),
        ],
        stroke,
    );
    ui.painter().line_segment(
        [
            egui::pos2(center.x - 2.0, center.y),
            egui::pos2(center.x, center.y + 6.0),
        ],
        stroke,
    );
    ui.painter().line_segment(
        [
            egui::pos2(center.x + 2.0, center.y),
            egui::pos2(center.x, center.y + 6.0),
        ],
        stroke,
    );

    response.on_hover_text(if active {
        "edit active column filter"
    } else {
        "filter this column"
    })
}