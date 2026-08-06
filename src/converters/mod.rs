use crate::app::result::ConvertResult;
use crate::app::state::{AppState, SelectedTool};
use crate::converters::data::DataFormatConverter;

pub mod base64;
pub mod binary;
pub mod crypt;
pub mod escape;
pub mod data;
pub mod regex;

#[derive(Default)]
pub struct Converters {
    pub regex: regex::RegexConverter,
    pub jq: DataFormatConverter,
}

pub trait Converter {
    fn convert(&mut self, state: &AppState) -> ConvertResult;
}

impl Converters {
    pub fn convert(&mut self, state: &AppState) -> ConvertResult {
        match state.selected {
            SelectedTool::Base64 => base64::convert(&state.input, &state.options.base64),
            SelectedTool::Binary => binary::convert(&state.input, &state.options.binary),
            SelectedTool::Escape => escape::convert(&state.input, &state.options.escape),
            SelectedTool::Crypt => crypt::convert(&state.input, &state.options.crypt),
            SelectedTool::Regex => self.regex.convert(state),
            SelectedTool::Data => self.jq.convert(state),
            SelectedTool::Diff => ConvertResult::Empty,
        }
    }
}