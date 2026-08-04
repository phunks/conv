use crate::app::result::ConvertResult;
use crate::app::state::AppState;
use crate::converters::Converter;
use crate::widgets::menu::RegexMenu;
use eframe::egui::text::LayoutJob;
use eframe::egui::{Color32, FontFamily, FontId, TextFormat};
use fancy_regex::Regex;
use std::iter::zip;

#[derive(Default, Clone, PartialEq)]
pub struct RegexOptions {
    pub mode: RegexMenu,
    /// pattern
    pub pattern: String,
    /// replace text
    pub replace: String,
    /// single line (dot matches new line)
    pub single_line: bool,
    /// use replace instead of match highlight
    pub replace_enabled: bool,
    /// case-insensitive match
    pub ignore_case: bool,
    /// invert (grep)
    pub invert: bool,
}

#[derive(Default)]
pub struct RegexConverter;

impl Converter for RegexConverter {
    fn convert(&mut self, state: &AppState) -> ConvertResult {
        let options = &state.options.regex;

        if options.pattern.is_empty() {
            return ConvertResult::Text(state.input.clone());
        }

        let regex = match build_regex(
            &options.pattern,
            options.ignore_case,
            options.single_line,
        ) {
            Ok(regex) => regex,
            Err(e) => return ConvertResult::Error(format!("warn: {e}")),
        };

        match options.mode {
            RegexMenu::Regex => regex_replace_result(&regex, state, options),
            RegexMenu::Grep => grep_result(&regex, state, options),
        }
    }
}

fn build_regex(
    pattern: &str,
    ignore_case: bool,
    single_line: bool,
) -> Result<Regex, fancy_regex::Error> {
    let mut prefix = String::new();

    if ignore_case {
        prefix.push_str("(?i)");
    }

    if single_line {
        prefix.push_str("(?s)");
    }

    Regex::new(&format!("{prefix}{pattern}"))
}

fn regex_replace_result(regex: &Regex, state: &AppState, options: &RegexOptions) -> ConvertResult {
    let mut job = LayoutJob::default();

    if options.single_line {
        highlight_line(&mut job, regex, &state.input, options);
        return ConvertResult::RichText(job);
    }

    let mut lines = state.input.split('\n').peekable();

    while let Some(line) = lines.next() {
        highlight_line(&mut job, regex, line, options);

        if lines.peek().is_some() {
            append_plain(&mut job, "\n");
        }
    }

    ConvertResult::RichText(job)
}

fn grep_result(regex: &Regex, state: &AppState, options: &RegexOptions) -> ConvertResult {
    let mut job = LayoutJob::default();
    let mut lines = state.input.split('\n').peekable();

    while let Some(line) = lines.next() {
        let has_match = regex.is_match(line).unwrap_or(false);

        let show = if options.invert { !has_match } else { has_match };

        if show {
            highlight_line(&mut job, regex, line, options);

            if lines.peek().is_some() {
                append_plain(&mut job, "\n");
            }
        }
    }

    ConvertResult::RichText(job)
}

/// Split one line by regex matches and append to the job.
///
/// - non-match spans: default color
/// - match spans: orange (replaced text if `replace_enabled`)
fn highlight_line(job: &mut LayoutJob, regex: &Regex, hay: &str, options: &RegexOptions) {
    let matches = regex
        .find_iter(hay)
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    if matches.is_empty() {
        append_plain(job, hay);
        return;
    }

    let mut splits = regex
        .split(hay)
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    let match_strings: Vec<String> = if options.replace_enabled {
        matches
            .iter()
            .map(|m| regex.replace(m.as_str(), options.replace.as_str()).into_owned())
            .collect()
    } else {
        matches.iter().map(|m| m.as_str().to_string()).collect()
    };

    let mut match_strings = match_strings;
    match_strings.push(String::new());

    if splits.len() < match_strings.len() {
        splits.resize(match_strings.len(), "");
    }

    for (matched, split) in zip(match_strings, splits) {
        append_plain(job, split);
        append_match(job, &matched);
    }
}

fn append_plain(job: &mut LayoutJob, text: &str) {
    if text.is_empty() {
        return;
    }

    job.append(
        text,
        0.0,
        TextFormat {
            font_id: FontId::new(12.5, FontFamily::Proportional),
            ..Default::default()
        },
    );
}

fn append_match(job: &mut LayoutJob, text: &str) {
    if text.is_empty() {
        return;
    }

    job.append(
        text,
        0.0,
        TextFormat {
            font_id: FontId::new(12.5, FontFamily::Proportional),
            color: Color32::ORANGE,
            ..Default::default()
        },
    );
}