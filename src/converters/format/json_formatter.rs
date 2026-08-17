use std::sync::LazyLock;
use eframe::egui::text::LayoutJob;
use eframe::egui::{Color32, FontId, TextFormat};
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::{SyntaxReference, SyntaxSet};
// use crate::app::colors::{JsonTreeColorScheme, JSON_KEY};

// static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);
// static THEME_SET: LazyLock<ThemeSet> = LazyLock::new(ThemeSet::load_defaults);
//
// pub fn json_to_layout_job(json: &str, dark_mode: bool) -> LayoutJob {
//     let syntax: &SyntaxReference = SYNTAX_SET
//         .find_syntax_by_extension("json")
//         .expect("the default syntect syntax set contains JSON");
//
//     let theme = if dark_mode {
//         &THEME_SET.themes["base16-ocean.dark"]
//     } else {
//         &THEME_SET.themes["base16-ocean.light"]
//     };
//
//     let custom_visuals = JsonTreeColorScheme::new();
//
//     let mut highlighter = HighlightLines::new(syntax, theme);
//     let mut job = LayoutJob::default();
//
//     job.wrap.max_width = f32::INFINITY;
//
//     for line in json.split_inclusive('\n') {
//         let Ok(ranges) = highlighter.highlight_line(line, &SYNTAX_SET) else {
//             job.append(
//                 line,
//                 0.0,
//                 TextFormat {
//                     font_id: FontId::monospace(12.0),
//                     color: Color32::PLACEHOLDER,
//                     ..Default::default()
//                 },
//             );
//             continue;
//         };
//
//         for (style, text) in ranges {
//             let trimmed = text.trim();
//             let color = if trimmed.starts_with('"') && trimmed.ends_with(':') {
//                 custom_visuals.object_key_color
//             } else if trimmed == "true" || trimmed == "false" {
//                 custom_visuals.bool_color
//             } else if trimmed.parse::<f64>().is_ok() {
//                 custom_visuals.number_color
//             } else if trimmed == "null" {
//                 custom_visuals.null_color
//             } else if trimmed.starts_with('"') {
//                 custom_visuals.string_color
//             } else {
//                 // Color32::from_rgba_unmultiplied(
//                 //     style.foreground.r,
//                 //     style.foreground.g,
//                 //     style.foreground.b,
//                 //     style.foreground.a,
//                 // )
//                 JSON_KEY
//             };
//
//             job.append(
//                 text,
//                 0.0,
//                 TextFormat {
//                     font_id: FontId::monospace(12.0),
//                     color,
//                     ..Default::default()
//                 },
//             );
//         }
//     }
//
//     job
// }