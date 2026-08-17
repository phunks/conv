use eframe::egui::Color32;
use egui_json_tree::JsonTreeVisuals;

pub(crate) const WARNING: Color32 = Color32::ORANGE;
pub(crate) const WARNING_TEXT: Color32 = Color32::from_rgb(180, 190, 120);

pub(crate) const TEXT_MUTED: Color32 = Color32::GRAY;
pub(crate) const TEXT_ON_ACCENT: Color32 = Color32::WHITE;
pub(crate) const TRANSPARENT: Color32 = Color32::TRANSPARENT;

pub(crate) const JSON_NULL: Color32 = Color32::from_rgb(160, 154, 154);
pub(crate) const JSON_BOOLEAN: Color32 = Color32::from_rgb(72, 146, 239);
pub(crate) const JSON_NUMBER: Color32 = Color32::from_rgb(251, 171, 35);
pub(crate) const JSON_KEY: Color32 = Color32::from_rgb(125, 155, 181);
pub(crate) const JSON_STRING: Color32 = Color32::from_rgb(223, 223, 223);

pub(crate) const DIAGNOSTIC_RED: Color32 = Color32::from_rgb(171, 127, 26);
pub(crate) const DIAGNOSTIC_YELLOW: Color32 = Color32::from_rgb(213, 49, 48);

pub(crate) const DIFF_LEFT_BG: Color32 = Color32::from_rgb(45, 22, 19);
pub(crate) const DIFF_LEFT_FG: Color32 = Color32::from_rgb(81, 21, 19);
pub(crate) const DIFF_RIGHT_BG: Color32 = Color32::from_rgb(11, 30, 15);
pub(crate) const DIFF_RIGHT_FG: Color32 = Color32::from_rgb(19, 70, 21);
pub(crate) const DIFF_SELECTION: Color32 = Color32::from_rgb(0, 92, 128);
pub(crate) const DIFF_HIGHLIGHT_UNDERLINE: Color32 = Color32::LIGHT_YELLOW;
pub(crate) const DIFF_BORDER: Color32 = Color32::GRAY;

pub(crate) const STATUS_CHANGED: Color32 = Color32::from_rgb(198, 255, 45);
pub(crate) const STATUS_UNCHANGED: Color32 = Color32::from_rgb(0, 238, 58);

pub struct JsonTreeColorScheme;
impl JsonTreeColorScheme {
    pub(crate) fn new() -> JsonTreeVisuals {
        JsonTreeVisuals {
            object_key_color: JSON_KEY,
            array_idx_color: TEXT_MUTED,
            bool_color: JSON_BOOLEAN,
            number_color: JSON_NUMBER,
            string_color: JSON_STRING,
            null_color: JSON_NULL,
            ..Default::default()
        }
    }
}
