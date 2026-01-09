use std::{io::Cursor, path::Path};

use prettypst::{format_node, Styles};

macro_rules! test_styles {
    ($input_data:expr) => {
        let input_data = include_str!(concat!("source/", $input_data, ".typ"));
        let root = typst_syntax::parse(input_data);

        let mut settings = insta::Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_path(Path::new("snapshots"));

        for (style_name, style) in [("default", Styles::Default), ("otbs", Styles::Otbs)] {
            let mut output = Cursor::new(Vec::new());
            format_node(&root, &style.settings(), &mut output);
            let output = output.into_inner();

            settings.set_snapshot_suffix(style_name);
            settings.bind(|| {
                insta::assert_binary_snapshot!(".typ", output);
            })
        }
    };
}

macro_rules! create_tests {
    (
       	$(($name:ident, $file:expr),)*
    ) => {
        $(
            #[test]
            fn $name() {
                test_styles!($file);
            }
        )*
    };
}

create_tests!(
    (columns, "columns"),
    (comments, "comments"),
    (example, "example"),
    (headings, "headings"),
    (label, "label"),
    (long, "long"),
    (math, "math"),
    (single_argument, "single_argument"),
    (term, "term"),
    (shebang_heading, "shebang/heading"),
    (shebang_linebreak, "shebang/linebreak"),
    (shebang_parbreak, "shebang/parbreak"),
);

#[test]
fn all_source_files_used() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let out_path = root.join("snapshots");

    let mut remaining = vec![(root.join("source"), String::new())];
    while let Some((in_path, out_name)) = remaining.pop() {
        for entry in std::fs::read_dir(&in_path).unwrap() {
            let entry = entry.unwrap();
            let file_type = entry.file_type().unwrap();

            let file_name = entry.path();
            let file_name = file_name.file_stem().unwrap();
            let out_name = match out_name.is_empty() {
                false => format!("{}_{}", out_name, file_name.display()),
                true => format!("{}", file_name.display()),
            };

            if file_type.is_dir() {
                remaining.push((in_path.join(file_name), out_name));
            } else if file_type.is_file() {
                for style in ["default", "otbs"] {
                    let path = out_path.join(format!("{}@{}.snap", out_name, style));
                    assert!(path.exists(), "Snapshot for {} missing", path.display());
                }
            }
        }
    }
}
