mod logic;
mod output;
pub mod settings;
mod state;
pub mod styles;

use ecow::EcoString;
use state::State;
use typst_syntax::{SyntaxKind, SyntaxNode};

use output::Output;

pub use output::OutputTarget;
use std::{
    fs::{self, File},
    io::{BufWriter, Read, Write},
    path::PathBuf,
};

pub use styles::Styles;

use clap::Parser;

const CONFIG_NAME: &str = "prettypst.toml";

#[derive(Debug, Clone, Parser)]
pub struct Command {
    /// Input path for source file, used as output path if nothing else is specified
    #[arg(default_value = None)]
    pub path: Option<PathBuf>,

    /// Output path
    #[arg(short, long, default_value = None)]
    pub output: Option<PathBuf>,

    /// Base style for the formatting settings
    #[arg(short, long, default_value_t = Styles::Default)]
    pub style: Styles,

    /// Search for 'prettypst.toml' for additional formatting settings
    #[arg(long, default_value_t = false)]
    pub use_configuration: bool,

    /// Generate file with formatting settings based on the style
    #[arg(long, default_value_t = false)]
    pub save_configuration: bool,

    /// Use standard input as source
    #[arg(long, default_value_t = false)]
    pub use_std_in: bool,

    /// Use standard output as target
    #[arg(long, default_value_t = false)]
    pub use_std_out: bool,

    /// File location to search for configuration, defaults to input path if available
    #[arg(long, default_value = None)]
    pub file_location: Option<PathBuf>,
}

#[derive(thiserror::Error, Debug)]
pub enum FormatError {
    #[error("Failed to get project Folder")]
    FailedToGetProjectFolder,
    #[error("No configuration file")]
    NoConfigurationFile,
    #[error("Failed to read configuration file")]
    FailedToReadConfigurationFile,
    #[error("malformed configuration file: {0}")]
    MalformatedConfigurationFile(#[from] toml::de::Error),
    #[error("failed to serialize configuration: {0}")]
    FailedToSerializeConfiguration(#[from] toml::ser::Error),
    #[error("failed to save configuration file")]
    FailedToSaveConfigurationFile,

    #[error("failed to read from stdin")]
    FailedToReadStdIn,
    #[error("no input file or stdin specified")]
    NoInputFileOrStdInSpecified,
    #[error("input file and stdin specified")]
    InputFileAndStdInSpecified,
    #[error("failed to read input file")]
    FailedToReadInputFile,

    #[error("output file and stdout specified")]
    OutputFileAndStdOutSpecified,
    #[error("failed to create output file")]
    FailedToCreateOutputFile,
    #[error("failed to create temporary file")]
    FailedToCreateTemporaryFile,
    #[error("failed to get temporary file path")]
    FailedToGetTemporaryFilePath,
    #[error("failed to replace input file")]
    FailedToReplaceInputFile,
}

pub fn format_node(
    node: &SyntaxNode,
    settings: &settings::Settings,
    target: &mut impl OutputTarget,
) -> Result<(), FormatError> {
    let mut output = Output::new(target);
    let state = State::new();
    logic::format(node, state, settings, &mut output);

    // ensure end of file is always present
    logic::format(
        &SyntaxNode::leaf(SyntaxKind::Eof, EcoString::new()),
        state,
        settings,
        &mut output,
    );
    output.finish(&state, settings);
    Ok(())
}

pub fn format_str(
    text: &str,
    settings: &settings::Settings,
    target: &mut impl OutputTarget,
) -> Result<(), FormatError> {
    format_node(&typst_syntax::parse(text), settings, target)
}

pub fn format(command: &Command) -> Result<(), FormatError> {
    let mut settings = command.style.settings();

    if command.use_configuration {
        let path = match (&command.file_location, &command.path) {
            (Some(path), _) => {
                if path.extension().is_some() {
                    path.parent()
                        .ok_or(FormatError::FailedToGetProjectFolder)?
                        .to_owned()
                } else {
                    path.to_owned()
                }
            }
            (_, Some(path)) => path.to_owned(),
            _ => std::env::current_dir().unwrap().to_owned(),
        };
        let mut path = path.as_path();
        let file = loop {
            let mut file = PathBuf::from(path);
            file.push(CONFIG_NAME);
            if file.is_file() {
                break file;
            }
            path = path.parent().ok_or(FormatError::NoConfigurationFile)?;
        };
        settings.merge(&file)?;
    }

    if command.save_configuration {
        std::fs::write(CONFIG_NAME, toml::to_string_pretty(&settings)?)
            .map_err(|_| FormatError::FailedToSaveConfigurationFile)?;
        return Ok(());
    }

    let (input_data, input_name) = match (&command.path, command.use_std_in) {
        (Some(_), true) => return Err(FormatError::InputFileAndStdInSpecified),
        (Some(path), false) => {
            let input_data =
                std::fs::read_to_string(path).map_err(|_| FormatError::FailedToReadInputFile)?;
            (input_data, path.display().to_string())
        }
        (None, true) => {
            let mut data = String::new();
            std::io::stdin()
                .read_to_string(&mut data)
                .map_err(|_| FormatError::FailedToReadStdIn)?;
            (data, "stdin".into())
        }
        (None, false) => return Err(FormatError::NoInputFileOrStdInSpecified),
    };

    let root = typst_syntax::parse(&input_data);

    match (&command.output, command.use_std_out) {
        (Some(_), true) => return Err(FormatError::OutputFileAndStdOutSpecified),
        (Some(out), false) => {
            let file = File::create(out).map_err(|_| FormatError::FailedToCreateOutputFile)?;
            let mut target = FileTarget(BufWriter::new(file));
            format_node(&root, &settings, &mut target)?;
            drop(target);
        }
        (None, true) => {
            let mut target = FileTarget(BufWriter::new(std::io::stdout()));
            format_node(&root, &settings, &mut target)?;
            drop(target);
        }
        (None, false) => {
            let temp_path = input_name.clone() + ".tmp";
            let file =
                File::create(&temp_path).map_err(|_| FormatError::FailedToCreateTemporaryFile)?;
            let mut target = FileTarget(BufWriter::new(file));
            format_node(&root, &settings, &mut target)?;
            drop(target);

            fs::rename(temp_path, input_name).map_err(|err| {
                println!("{}", err);
                FormatError::FailedToReplaceInputFile
            })?;
        }
    };
    Ok(())
}

pub struct FileTarget<T: std::io::Write>(BufWriter<T>);

impl<T: std::io::Write> OutputTarget for FileTarget<T> {
    fn emit(&mut self, data: &EcoString, _settings: &settings::Settings) {
        self.0.write_all(data.as_bytes()).unwrap();
    }
}
