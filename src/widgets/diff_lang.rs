use strum::{EnumMessage, VariantArray};

pub const MAX_STRUCTURAL_DIFF_INPUT_BYTES: usize = 256 * 1024;

#[derive(Default, Clone, Copy, PartialEq, Eq, VariantArray, EnumMessage)]
pub enum DiffLanguage {
    #[default]
    /// text
    #[strum(message = "Text  ")]
    Text,
    /// json
    #[strum(message = "JSON  ")]
    Json,
    /// yaml
    #[strum(message = "YAML  ")]
    Yaml,
    /// toml
    #[strum(message = "TOML  ")]
    Toml,
    /// xml
    #[strum(message = "XML   ")]
    Xml,
    /// rust
    #[strum(message = "Rust  ")]
    Rust,
    /// javascript
    #[strum(message = "JS    ")]
    JavaScript,
    /// typescript
    #[strum(message = "TS    ")]
    TypeScript,
    /// html
    #[strum(message = "HTML  ")]
    Html,
    /// css
    #[strum(message = "CSS   ")]
    Css,
    /// shell
    #[strum(message = "Shell ")]
    Shell,
    /// python
    #[strum(message = "Python")]
    Python,
    /// go
    #[strum(message = "Go    ")]
    Go,
    /// java
    #[strum(message = "Java  ")]
    Java,
    /// c
    #[strum(message = "C     ")]
    C,
    /// c++
    #[strum(message = "C++   ")]
    Cpp,
    /// sql
    #[strum(message = "SQL   ")]
    Sql,
}

impl DiffLanguage {
    pub const fn structural_diff_input_limit(self) -> Option<usize> {
        match self {
            Self::Toml | Self::TypeScript | Self::Html | Self::Css => {
                Some(MAX_STRUCTURAL_DIFF_INPUT_BYTES)
            }
            Self::Text
            | Self::Json
            | Self::Yaml
            | Self::Xml
            | Self::Rust
            | Self::JavaScript
            | Self::Shell
            | Self::Python
            | Self::Go
            | Self::Java
            | Self::C
            | Self::Cpp
            | Self::Sql => None,
        }
    }

    pub const fn virtual_path(self) -> &'static str {
        match self {
            Self::Text => "clipboard.txt",
            Self::Json => "clipboard.json",
            Self::Yaml => "clipboard.yaml",
            Self::Toml => "clipboard.toml",
            Self::Xml => "clipboard.xml",
            Self::Rust => "clipboard.rs",
            Self::JavaScript => "clipboard.js",
            Self::TypeScript => "clipboard.ts",
            Self::Html => "clipboard.html",
            Self::Css => "clipboard.css",
            Self::Shell => "clipboard.sh",
            Self::Python => "clipboard.py",
            Self::Go => "clipboard.go",
            Self::Java => "clipboard.java",
            Self::C => "clipboard.c",
            Self::Cpp => "clipboard.cpp",
            Self::Sql => "clipboard.sql",
        }
    }
}