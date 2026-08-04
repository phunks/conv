use crate::converters::base64::Base64Options;
use crate::converters::binary::BinaryOptions;
use crate::converters::crypt::CryptOptions;
use crate::converters::escape::EscapeOptions;
use crate::converters::jq::JqOptions;
use crate::converters::regex::RegexOptions;

#[derive(Default, Clone, PartialEq)]
pub struct AppState {
    pub input: String,
    pub selected: SelectedTool,
    pub options: ToolOptions,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum SelectedTool {
    #[default]
    Base64,
    Binary,
    Escape,
    Crypt,
    Regex,
    Jq,
}

impl SelectedTool {
    pub const COUNT: usize = 6;

    pub const fn index(self) -> usize {
        match self {
            Self::Base64 => 0,
            Self::Binary => 1,
            Self::Escape => 2,
            Self::Crypt => 3,
            Self::Regex => 4,
            Self::Jq => 5,
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
    pub jq: JqOptions,
}

