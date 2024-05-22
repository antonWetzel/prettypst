use typst_syntax::SyntaxNode;

use crate::state::State;

use super::settings::Settings;

#[derive(Debug, Clone, Copy)]
pub enum Whitespace {
    None,
    Space,
    Spaces(usize),
    LineBreak,
    LineBreaks(usize),
}

impl Whitespace {
    pub fn new(text: &str) -> Self {
        fn count_newlines(text: &str) -> usize {
            let mut newlines = 0;
            let mut chars = text.chars().peekable();
            while let Some(c) = chars.next() {
                if typst_syntax::is_newline(c) {
                    if c == '\r' {
                        if chars.peek() == Some(&'\n') {
                            chars.next();
                        }
                    }
                    newlines += 1;
                }
            }
            newlines
        }

        match count_newlines(text) {
            0 if text.is_empty() => Self::None,
            0 if text.len() == 1 => Self::Space,
            0 => Self::Spaces(text.len()),
            1 => Self::LineBreak,
            other => Self::LineBreaks(other),
        }
    }

    pub fn limit(self) -> Self {
        match self {
            Self::None => Self::None,
            Self::Space | Self::Spaces(_) => Self::Space,
            Self::LineBreak => Self::LineBreak,
            Self::LineBreaks(_) => Self::LineBreaks(2),
        }
    }

    pub fn as_linebreak(self) -> Self {
        match self {
            Self::LineBreaks(_) => Self::LineBreaks(2),
            _ => Self::LineBreak,
        }
    }
}

pub trait OutputTarget {
    fn emit(&mut self, data: &str, settings: &Settings);
}

impl<T: std::io::Write> OutputTarget for T {
    fn emit(&mut self, data: &str, _settings: &Settings) {
        self.write_all(data.as_bytes()).unwrap();
    }
}

pub struct Output<'a, Target: OutputTarget> {
    target: &'a mut Target,
    ws: Whitespace,
}

impl<'a, Target: OutputTarget> Output<'a, Target> {
    pub fn new(target: &'a mut Target) -> Self {
        Self {
            target,
            ws: Whitespace::None,
        }
    }

    fn emit_indentation(&mut self, state: &State, settings: &Settings) {
        if state.indentation + state.extra_indentation == 0 {
            return;
        }
        match settings.indentation {
            0 => self.target.emit(
                &format!(
                    "{0:\t<1$}{0: <2$}",
                    "", state.indentation, state.extra_indentation
                ),
                settings,
            ),
            amount => self.target.emit(
                &format!(
                    "{0: <1$}",
                    "",
                    state.indentation * amount + state.extra_indentation
                ),
                settings,
            ),
        }
    }

    pub fn emit_whitespace(&mut self, whitespace: Whitespace) {
        self.ws = whitespace;
    }

    pub fn text(&mut self, text: &str, state: &State, settings: &Settings) {
        if text.is_empty() {
            return;
        }
        match std::mem::replace(&mut self.ws, Whitespace::None) {
            Whitespace::None => {}
            Whitespace::Space => self.target.emit(" ", settings),
            Whitespace::Spaces(amount) => {
                self.target.emit(&format!("{0: <1$}", "", amount), settings);
            }
            Whitespace::LineBreak => {
                self.target.emit("\n", settings);
                self.emit_indentation(state, settings)
            }
            Whitespace::LineBreaks(amount) => {
                self.target
                    .emit(&format!("{0:\n<1$}", "", amount), settings);
                self.emit_indentation(state, settings)
            }
        }

        self.target.emit(text, settings);
    }
}

pub struct PositionCalculator {
    line: usize,
    column: usize,
}

impl PositionCalculator {
    pub fn new() -> Self {
        Self { line: 0, column: 0 }
    }

    pub fn reset(&mut self) {
        self.line = 0;
        self.column = 0;
    }
}

impl OutputTarget for PositionCalculator {
    fn emit(&mut self, data: &str, settings: &Settings) {
        for symbol in data.chars() {
            match symbol {
                '\t' => {
                    let tab_size = settings.indentation.max(1);
                    self.column += 1 + tab_size.overflowing_sub(self.column).0 % tab_size
                }
                '\n' => {
                    self.line += 1;
                    self.column = 1;
                }
                _ => self.column += 1,
            }
        }
    }
}

impl Output<'_, PositionCalculator> {
    pub fn position(&self) -> (usize, usize) {
        (self.target.line, self.target.column)
    }

    pub fn reset(&mut self) {
        self.target.reset();
    }
}
