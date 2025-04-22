use crate::settings::Settings;

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    /// `[...]` or top level
    Markup,
    /// `[...]` or top level, where automatic linebreaks may be inserted
    MarkupBreakable,
    /// `{...}`
    Code,
    /// `$...$`
    Math,
    /// `(_, ...)`
    Items,
    MultilineItems,
}

#[derive(Debug, Clone, Copy)]
pub struct State {
    pub indentation: usize,
    pub extra_indentation: usize,
    pub mode: Mode,
}

impl State {
    pub fn new(settings: &Settings) -> Self {
        Self {
            indentation: 0,
            extra_indentation: 0,
            mode: match settings.automatic_newline.max_width {
                0 => Mode::Markup,
                _ => Mode::MarkupBreakable,
            },
        }
    }

    pub fn indent(&mut self) {
        self.indentation += 1;
    }

    pub fn dedent(&mut self) {
        self.indentation -= 1
    }
}
