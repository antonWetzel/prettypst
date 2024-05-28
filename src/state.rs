#[derive(Debug, Clone, Copy)]
pub enum Mode {
    /// `[...]` or top level
    Markup,
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
    pub fn new() -> Self {
        Self {
            indentation: 0,
            extra_indentation: 0,
            mode: Mode::Markup,
        }
    }
    pub fn indent(&mut self) {
        self.indentation += 1;
    }

    pub fn dedent(&mut self) {
        self.indentation -= 1
    }
}
