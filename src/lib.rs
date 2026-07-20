mod logic;
mod output;
mod settings;
mod state;
mod styles;

use std::{
    fs::{self, File},
    io::{BufWriter, Read},
    path::PathBuf,
};

use clap::Parser;
use output::Output;
use state::State;
use typst_syntax::{SyntaxKind, SyntaxNode};

pub use crate::{output::OutputTarget, settings::Settings, styles::Styles};

const CONFIG_NAME: &str = "prettypst.toml";

#[derive(Debug, Clone, Parser)]
pub struct Command {
    /// Input path for source file, used as output path if nothing else is specified.
    ///
    /// If standard-input is used, this is only the file location to search for the configuration.
    #[arg(default_value = None)]
    pub path: Option<PathBuf>,

    /// Output path.
    #[arg(short, long, default_value = None)]
    pub output: Option<PathBuf>,

    /// Base style for the formatting settings.
    #[arg(short, long, default_value_t = Styles::Default)]
    pub style: Styles,

    /// Generate file with formatting settings based on the style.
    #[arg(long, default_value_t = false)]
    pub save_configuration: bool,

    /// Use standard input as source.
    #[arg(long, default_value_t = false)]
    pub use_std_in: bool,

    /// Use standard output as target.
    #[arg(long, default_value_t = false)]
    pub use_std_out: bool,

    /// Directory to search for configuration.
    ///
    /// In the directory and all parent directories the `prettypst.toml` files
    /// are combined to create the configuration.
    ///
    /// If unspecified, the following is used instead:
    /// 1. Containing directory of the input path if available
    /// 1. Current working directory
    #[arg(long, default_value = None)]
    pub config_directory: Option<PathBuf>,
}

#[derive(thiserror::Error, Debug)]
pub enum FormatError {
    #[error("failed to get canonicalize path")]
    FailedToCanonicalizePath(std::io::Error),
    #[error("failed to get working directory")]
    FailedToGetWorkingDirectory(std::io::Error),
    #[error("failed to read configuration file")]
    FailedToReadConfigurationFile(std::io::Error),
    #[error("malformed configuration file: {0}")]
    MalformatedConfigurationFile(#[from] toml::de::Error),
    #[error("failed to serialize configuration: {0}")]
    FailedToSerializeConfiguration(#[from] toml::ser::Error),
    #[error("failed to save configuration file")]
    FailedToSaveConfigurationFile(std::io::Error),

    #[error("failed to read from stdin")]
    FailedToReadStdIn(std::io::Error),
    #[error("no input file or stdin specified")]
    NoInputFileOrStdInSpecified,
    #[error("failed to read input file")]
    FailedToReadInputFile(std::io::Error),

    #[error("output file and stdout specified")]
    OutputFileAndStdOutSpecified,
    #[error("failed to create output file")]
    FailedToCreateOutputFile(std::io::Error),
    #[error("failed to create temporary file")]
    FailedToCreateTemporaryFile(std::io::Error),
    #[error("failed to get temporary file path")]
    FailedToGetTemporaryFilePath(std::io::Error),
    #[error("failed to replace input file")]
    FailedToReplaceInputFile(std::io::Error),
}

pub fn format_node(node: &SyntaxNode, settings: &Settings, target: &mut impl OutputTarget) {
    let mut output = Output::new(target);
    let state = State::new(settings);
    logic::format(node, state, settings, &mut output);

    #[cfg(feature = "print-root")]
    println!("{:#?}", node);

    // ensure end of file is always present
    logic::format(
        &SyntaxNode::leaf(SyntaxKind::End, ""),
        state,
        settings,
        &mut output,
    );
    output.finish(&state, settings);
}

pub fn format_str(text: &str, settings: &settings::Settings, target: &mut impl OutputTarget) {
    format_node(&typst_syntax::parse(text), settings, target)
}

pub fn format(command: &Command) -> Result<(), FormatError> {
    let mut settings = command.style.settings();

    let settings_dir = if let Some(dir) = &command.config_directory {
        dir.canonicalize()
            .map_err(FormatError::FailedToCanonicalizePath)?
    } else if let Some(path) = &command.path
        && let Some(dir) = path.parent()
    {
        dir.canonicalize()
            .map_err(FormatError::FailedToCanonicalizePath)?
    } else {
        std::env::current_dir().map_err(FormatError::FailedToGetWorkingDirectory)?
    };

    // from root to settings directory find all config files
    let mut settings_path = PathBuf::new();
    for component in settings_dir.components() {
        settings_path.push(component);
        settings_path.push(CONFIG_NAME);
        if settings_path.is_file() {
            settings.overwrite(&settings_path)?;
        }
        settings_path.pop();
    }

    if command.save_configuration {
        std::fs::write(
            settings_dir.join(CONFIG_NAME),
            toml::to_string_pretty(&settings)?,
        )
        .map_err(FormatError::FailedToSaveConfigurationFile)?;
        return Ok(());
    }

    let input_data = if command.use_std_in {
        let mut data = String::new();
        std::io::stdin()
            .read_to_string(&mut data)
            .map_err(FormatError::FailedToReadStdIn)?;
        data
    } else if let Some(path) = &command.path {
        std::fs::read_to_string(path).map_err(FormatError::FailedToReadInputFile)?
    } else {
        Err(FormatError::NoInputFileOrStdInSpecified)?
    };

    let root = typst_syntax::parse(&input_data);

    match (&command.output, command.use_std_out) {
        (Some(_), true) => return Err(FormatError::OutputFileAndStdOutSpecified),
        (Some(out), false) => {
            let file = File::create(out).map_err(FormatError::FailedToCreateOutputFile)?;
            let mut target = BufWriter::new(file);
            format_node(&root, &settings, &mut target);
            drop(target);
        }
        (None, true) => {
            let mut target = BufWriter::new(std::io::stdout());
            format_node(&root, &settings, &mut target);
            drop(target);
        }
        (None, false) => {
            let input_name = command.path.as_deref().map_or_else(
                || String::from("unknown.typ"),
                |path| path.display().to_string(),
            );

            let temp_path = format!("{}.tmp", input_name);
            let file =
                File::create(&temp_path).map_err(FormatError::FailedToCreateTemporaryFile)?;
            let mut target = BufWriter::new(file);
            format_node(&root, &settings, &mut target);
            drop(target);

            fs::rename(temp_path, input_name).map_err(FormatError::FailedToReplaceInputFile)?;
        }
    };
    Ok(())
}
