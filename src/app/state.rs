use crate::converters::base64::Base64Options;
use crate::converters::binary::BinaryOptions;
use crate::converters::crypt::CryptOptions;
use crate::converters::escape::EscapeOptions;
use crate::converters::data::DataOptions;
use crate::converters::regex::RegexOptions;
use crate::widgets::diff_lang::DiffLanguage;

#[derive(Default, Clone, PartialEq)]
pub struct AppState {
    pub input: String,
    pub selected: SelectedTool,
    pub options: ToolOptions,
}

#[derive(Clone, PartialEq)]
pub struct DiffToolOptions {
    pub left: String,
    pub right: String,
    pub language: DiffLanguage,
    pub context_lines: usize,
    pub ignore_comments: bool,
    pub show_hex: bool,
    pub scroll_offset: f32,
    pub change_index: usize,
    pub pending_change_delta: i32,
}

impl Default for DiffToolOptions {
    fn default() -> Self {
        Self {
            left: String::new(),
            right: String::new(),
            language: DiffLanguage::Text,
            context_lines: 3,
            ignore_comments: false,
            show_hex: false,
            scroll_offset: 0.0,
            change_index: usize::MAX,
            pending_change_delta: 0,
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum SelectedTool {
    #[default]
    Base64,
    Binary,
    Escape,
    Crypt,
    Regex,
    Data,
    Diff,
}

impl SelectedTool {
    pub const COUNT: usize = 7;

    pub const fn index(self) -> usize {
        match self {
            Self::Base64 => 0,
            Self::Binary => 1,
            Self::Escape => 2,
            Self::Crypt => 3,
            Self::Regex => 4,
            Self::Data => 5,
            Self::Diff => 6,
        }
    }
}

#[derive(Default, Clone, PartialEq)]
pub struct ToolOptions {
    pub base64: Base64Options,
    pub binary: BinaryOptions,
    pub escape: EscapeOptions,
    pub crypt: CryptOptions,
    pub regex: RegexOptions,
    pub data: DataOptions,
    pub diff: DiffToolOptions,
}

