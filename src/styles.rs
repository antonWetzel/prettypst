use std::fmt::{self, Display, Formatter};

use crate::settings::*;

use clap::ValueEnum;

#[derive(Debug, Copy, Clone, ValueEnum)]
pub enum Styles {
    /// Laurmaedje's style
    Default,
    /// One true bracket style
    Otbs,
}

impl Styles {
    pub fn settings(&self) -> Settings {
        match self {
            Self::Default => Settings {
                indentation: 2,
                seperate_label: true,
                preserve_newline: PreserveNewLine {
                    content: true,
                    math: true,
                },
                term: PaddingSettings {
                    space_before: false,
                    space_after: true,
                },
                named_argument: PaddingSettings {
                    space_before: false,
                    space_after: true,
                },
                dictionary_entry: PaddingSettings {
                    space_before: false,
                    space_after: true,
                },
                comma: PaddingSettings {
                    space_before: false,
                    space_after: true,
                },
                columns: ColumnsSettings {
                    comma: AlignComma::EndOfContent,
                },
                block: BlockSettings {
                    long_block_style: LongBlockStyle::Compact,
                },
                final_newline: true,
                heading: HeadingSettings {
                    blank_lines_before: 1,
                    blank_lines_after: 0,
                },
            },
            Self::Otbs => Settings {
                indentation: 0,
                seperate_label: true,
                preserve_newline: PreserveNewLine {
                    content: false,
                    math: true,
                },
                term: PaddingSettings {
                    space_before: false,
                    space_after: true,
                },
                named_argument: PaddingSettings {
                    space_before: false,
                    space_after: true,
                },
                dictionary_entry: PaddingSettings {
                    space_before: false,
                    space_after: true,
                },
                comma: PaddingSettings {
                    space_before: false,
                    space_after: true,
                },
                columns: ColumnsSettings {
                    comma: AlignComma::EndOfContent,
                },
                block: BlockSettings {
                    long_block_style: LongBlockStyle::Seperate,
                },
                final_newline: true,
                heading: HeadingSettings {
                    blank_lines_before: 2,
                    blank_lines_after: 1,
                },
            },
        }
    }
}

impl Display for Styles {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Default => write!(f, "default"),
            Self::Otbs => write!(f, "otbs"),
        }
    }
}
