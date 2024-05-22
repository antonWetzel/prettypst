#[derive(Debug, Clone, Copy)]
pub enum Mode {
    SinglelineMarkdown,
    MultilineMarkdown,

    SinglelineCode,
    MultilineCode,

    SinglelineArgs,
    MultilineArgs,
}

impl Mode {
    pub fn preserve_linebreak(self) -> bool {
        match self {
            Self::SinglelineMarkdown => false,
            Self::MultilineMarkdown => false,
            Self::SinglelineCode => false,
            Self::MultilineCode => true,
            Self::SinglelineArgs => false,
            Self::MultilineArgs => true,
        }
    }
    pub fn preserve_linebreaks(self) -> bool {
        match self {
            Self::SinglelineMarkdown => true,
            Self::MultilineMarkdown => true,
            Self::SinglelineCode => false,
            Self::MultilineCode => true,
            Self::SinglelineArgs => false,
            Self::MultilineArgs => true,
        }
    }
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
            mode: Mode::MultilineMarkdown,
        }
    }
    pub fn indent(&mut self) {
        self.indentation += 1;
    }

    pub fn dedent(&mut self) {
        self.indentation -= 1
    }
}
