use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::FormatError;

trait Merge {
    type Partial;
    fn merge(&mut self, other: Self::Partial);
}

macro_rules! identity_merge {
    ($($t:ty),*$(,)?) => {
        $(
            impl Merge for $t {
                type Partial = Self;
                fn merge(&mut self, other: Self) {
                    *self = other;
                }
            }
        )*
    };
}

macro_rules! create_normal_and_partial {
    ($(struct $name:ident | $partial_name:ident {$(pub $member:ident: $member_type:ty,)*})*) => {
        $(
            #[derive(Serialize, Debug)]
            pub struct $name {
                $(
                    pub $member: $member_type,
                )*
            }

            #[derive(Deserialize, Debug)]
            struct $partial_name {
                $(
                    pub $member: Option<<$member_type as Merge>::Partial>,
                )*
            }

            impl Merge for $name {
                type Partial = $partial_name;
                fn merge(&mut self, other: $partial_name) {
                    $(
                        if let Some(value) = other.$member {
                            self.$member.merge(value);
                        }
                    )*
                }
            }
        )*
    };
}

#[derive(Deserialize, Serialize, Debug)]
pub enum UseLongBlock {
    Never,
    HasAligment,
    Always,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum LongBlockStyle {
    Compact,
    Seperate,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum AlignComma {
    EndOfContent,
    EndOfCell,
}

identity_merge!(usize, bool, UseLongBlock, LongBlockStyle, AlignComma);

create_normal_and_partial!(
    struct BlockSettings | PartialBlockSettings {
        pub long_block_style: LongBlockStyle,
    }

    struct HeadingSettings | PartialHeadingSettings {
        pub blank_lines_before: usize,
        pub blank_lines_after: usize,
    }

    struct ColumnsSettings | PartialColumnsSettings {
        pub comma: AlignComma,
    }

    struct PaddingSettings | PartialPaddingSettings {
        pub space_before: bool,
        pub space_after: bool,
    }

    struct PreserveNewLine | PartialPreserveNewLine {
        pub content: bool,
        pub math: bool,
    }

    struct Settings | PartialSettings {
        pub indentation: usize,
        pub seperate_label: bool,
        pub final_newline: bool,
        pub preserve_newline: PreserveNewLine,
        pub block: BlockSettings,
        pub term: PaddingSettings,
        pub named_argument: PaddingSettings,
        pub dictionary_entry: PaddingSettings,
        pub import_statement: PaddingSettings,
        pub comma: PaddingSettings,
        pub columns: ColumnsSettings,
        pub heading: HeadingSettings,
    }
);

impl Settings {
    pub fn merge(&mut self, path: &PathBuf) -> Result<(), FormatError> {
        let data =
            std::fs::read_to_string(path).map_err(FormatError::FailedToReadConfigurationFile)?;
        let partial = toml::from_str(&data)?;
        <Self as Merge>::merge(self, partial);
        Ok(())
    }
}
