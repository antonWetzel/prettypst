use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

use crate::FormatError;

trait Overwrite {
    type Partial;
    fn overwrite(&mut self, other: Self::Partial);
}

macro_rules! identity_overwrite {
    ($($t:ty),*$(,)?) => {
        $(
            impl Overwrite for $t {
                type Partial = Self;
                fn overwrite(&mut self, other: Self) {
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
            #[serde(rename_all = "kebab-case")]
            pub struct $name {
                $(
                    pub $member: $member_type,
                )*
            }

            #[derive(Deserialize, Debug)]
            #[serde(rename_all = "kebab-case")]
            struct $partial_name {
                $(
                    pub $member: Option<<$member_type as Overwrite>::Partial>,
                )*
            }

            impl Overwrite for $name {
                type Partial = $partial_name;
                fn overwrite(&mut self, other: $partial_name) {
                    $(
                        if let Some(value) = other.$member {
                            self.$member.overwrite(value);
                        }
                    )*
                }
            }
        )*
    };
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum UseLongBlock {
    Never,
    HasAligment,
    Always,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum LongBlockStyle {
    Compact,
    Seperate,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum AlignComma {
    EndOfContent,
    EndOfCell,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum ColumnArgument {
    Default,
    Custom(String),
}

identity_overwrite!(usize, bool, UseLongBlock, LongBlockStyle, AlignComma);

impl<K: std::hash::Hash + std::cmp::Eq, T> Overwrite for HashMap<K, T> {
    type Partial = Self;

    fn overwrite(&mut self, other: Self::Partial) {
        *self = other;
    }
}

impl<K: std::hash::Hash + std::cmp::Eq> Overwrite for HashSet<K> {
    type Partial = Self;

    fn overwrite(&mut self, other: Self::Partial) {
        *self = other;
    }
}

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

        pub columns_methods: HashMap<String, String>,
    }
);

impl Settings {
    pub fn overwrite(&mut self, path: &PathBuf) -> Result<(), FormatError> {
        let data =
            std::fs::read_to_string(path).map_err(FormatError::FailedToReadConfigurationFile)?;
        let partial = toml::from_str(&data)?;
        <Self as Overwrite>::overwrite(self, partial);
        Ok(())
    }
}
