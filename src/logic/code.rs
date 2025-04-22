use super::*;

use std::{
    collections::{HashMap, HashSet},
    ops::Not,
};

pub fn format_code_block(
    node: &SyntaxNode,
    mut state: State,
    settings: &Settings,
    output: &mut Output<impl OutputTarget>,
) {
    let single = node
        .children()
        .all(|value| value.kind() != SyntaxKind::Space || value.text().contains('\n').not());
    state.mode = Mode::Code;
    for child in node.children() {
        match child.kind() {
            SyntaxKind::LeftBrace => {
                format(child, state, settings, output);
                if single {
                    output.set_whitespace(Whitespace::Space, Priority::Low);
                } else {
                    state.indent();
                    output.set_whitespace(Whitespace::LineBreak, Priority::Normal);
                }
            }
            SyntaxKind::Code => format(child, state, settings, output),
            SyntaxKind::RightBrace => {
                if single {
                    output.set_whitespace(Whitespace::Space, Priority::Low);
                } else {
                    state.dedent();
                    output.set_whitespace(Whitespace::LineBreak, Priority::Normal);
                }
                format(child, state, settings, output);
            }
            _ => format(child, state, settings, output),
        }
    }
}

pub fn format_func_call(
    node: &SyntaxNode,
    state: State,
    settings: &Settings,
    output: &mut Output<impl OutputTarget>,
) {
    #[derive(Debug)]
    enum Kind<'a> {
        Normal,
        Columns(&'a str),
    }
    let mut kind = Kind::Normal;
    for child in node.children() {
        match child.kind() {
            SyntaxKind::Ident => {
                kind = match settings.columns_methods.get(child.text().as_str()) {
                    None => Kind::Normal,
                    Some(column_argument) => Kind::Columns(column_argument),
                };
                format(child, state, settings, output);
            }
            SyntaxKind::Args => match kind {
                Kind::Normal => format_items(child, state, settings, output),
                Kind::Columns(column_argument) => {
                    format_column_args(child, state, settings, output, column_argument)
                }
            },
            _ => format(child, state, settings, output),
        }
    }
}

pub fn format_unary(
    node: &SyntaxNode,
    state: State,
    settings: &Settings,
    output: &mut Output<impl OutputTarget>,
) {
    for child in node.children() {
        match child.kind() {
            SyntaxKind::Plus | SyntaxKind::Minus => {
                format(child, state, settings, output);
                output.set_whitespace(Whitespace::None, Priority::High);
            }
            _ => format(child, state, settings, output),
        }
    }
}

pub fn format_named_argument(
    node: &SyntaxNode,
    state: State,
    settings: &Settings,
    output: &mut Output<impl OutputTarget>,
) {
    for child in node.children() {
        match child.kind() {
            SyntaxKind::Colon => {
                format_optional_padding(child, state, settings, output, &settings.named_argument)
            }
            _ => format(child, state, settings, output),
        }
    }
}

pub fn format_keyed(
    node: &SyntaxNode,
    state: State,
    settings: &Settings,
    output: &mut Output<impl OutputTarget>,
) {
    for child in node.children() {
        match child.kind() {
            SyntaxKind::Colon => {
                format_optional_padding(child, state, settings, output, &settings.dictionary_entry);
            }
            _ => format(child, state, settings, output),
        }
    }
}

pub fn format_semicolon(
    node: &SyntaxNode,
    state: State,
    settings: &Settings,
    output: &mut Output<impl OutputTarget>,
) {
    output.set_whitespace(Whitespace::None, Priority::Guaranteed);
    output.raw(node, &state, settings);
}

pub fn format_items(
    node: &SyntaxNode,
    mut state: State,
    settings: &Settings,
    output: &mut Output<impl OutputTarget>,
) {
    let mut trailing_comma = false;
    let mut comma_count = 0;
    for child in node.children() {
        match child.kind() {
            SyntaxKind::Comma => (trailing_comma, comma_count) = (true, comma_count + 1),
            SyntaxKind::RightParen => break,
            SyntaxKind::Space | SyntaxKind::LineComment | SyntaxKind::BlockComment => {}
            _ => trailing_comma = false,
        }
    }

    let force_single_inline = matches!(node.kind(), SyntaxKind::Array);
    let single = !trailing_comma || (force_single_inline && comma_count <= 1);
    state.mode = if single {
        Mode::Items
    } else {
        Mode::MultilineItems
    };

    for child in node.children() {
        match child.kind() {
            SyntaxKind::LeftParen => {
                format(child, state, settings, output);
                if single {
                    output.set_whitespace(Whitespace::None, Priority::Guaranteed);
                } else {
                    state.indent();
                    output.set_whitespace(Whitespace::LineBreak, Priority::High);
                }
            }
            SyntaxKind::Comma => {
                if single {
                    format_optional_padding(child, state, settings, output, &settings.comma);
                } else {
                    format(child, state, settings, output);
                    output.set_whitespace(Whitespace::LineBreak, Priority::Normal);
                }
            }
            SyntaxKind::RightParen => {
                if single {
                    output.set_whitespace(Whitespace::None, Priority::High);
                } else {
                    state.dedent();
                    output.set_whitespace(Whitespace::LineBreak, Priority::High);
                }
                format(child, state, settings, output);
            }
            _ => format(child, state, settings, output),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum CellSize {
    FullLine,
    Single(usize),
    Block { x: usize, y: usize, length: usize },
}

impl CellSize {
    pub fn new(
        node: &SyntaxNode,
        state: State,
        settings: &Settings,
        output: &mut Output<impl OutputTarget>,
    ) -> Self {
        let length = get_length(node, state, settings, output).unwrap_or(0) + 2;
        if node.kind() != SyntaxKind::FuncCall {
            return Self::Single(length);
        }
        let Some(base) = node.children().next() else {
            return Self::Single(length);
        };

        let Some(name) = base
            .children()
            .rev()
            .find(|child| child.kind() == SyntaxKind::Ident)
            .map(|child| child.text())
        else {
            return Self::Single(length);
        };

        match name.as_str() {
            "header" | "footer" => Self::FullLine,
            "cell" => {
                let Some(args) = node.children().find(|node| node.kind() == SyntaxKind::Args)
                else {
                    return Self::Single(length);
                };
                let mut x = 1;
                let mut y = 1;
                for child in args.children() {
                    if child.kind() != SyntaxKind::Named {
                        continue;
                    }
                    let name = child
                        .children()
                        .find(|node| node.kind() == SyntaxKind::Ident)
                        .map(|child| child.text().as_str());
                    let value = child
                        .children()
                        .find(|node| node.kind() == SyntaxKind::Int)
                        .map(|child| child.text().parse::<usize>());
                    match (name, value) {
                        (Some("colspan"), Some(Ok(size))) => x = size,
                        (Some("rowspan"), Some(Ok(size))) => y = size,
                        _ => continue,
                    }
                }
                Self::Block { x, y, length }
            }
            _ => Self::Single(length),
        }
    }
}

struct Aligner {
    index: usize,
    column: usize,
    row: usize,
    columns_count: usize,
    columns: Vec<usize>,
    cells: Vec<CellSize>,
    skip: HashSet<(usize, usize)>,
}

impl Aligner {
    pub fn new(
        node: &SyntaxNode,
        column_argument: &str,
        state: State,
        settings: &Settings,
        output: &mut Output<impl OutputTarget>,
    ) -> Self {
        let columns_count = get_column_count(node, column_argument);
        let mut cells = Vec::new();
        for child in node.children() {
            match child.kind() {
                SyntaxKind::LeftParen
                | SyntaxKind::RightParen
                | SyntaxKind::Comma
                | SyntaxKind::Space
                | SyntaxKind::LineComment
                | SyntaxKind::BlockComment
                | SyntaxKind::Named => {
                    continue;
                }
                _ => {}
            }
            cells.push(CellSize::new(child, state, settings, output));
        }

        let mut column = 0;
        let mut row = 0;
        let mut dependencies = vec![HashMap::<usize, usize>::new(); columns_count];
        let mut skip = HashSet::<(usize, usize)>::new();

        fn step(column: &mut usize, row: &mut usize, columns_count: usize) {
            *column += 1;
            if *column >= columns_count {
                *column = 0;
                *row += 1;
            }
        }
        for &cell in cells.iter() {
            while skip.contains(&(column, row)) {
                step(&mut column, &mut row, columns_count);
            }
            match cell {
                CellSize::FullLine => {
                    column = 0;
                    row += 1;
                }
                CellSize::Single(length) => {
                    let entry = dependencies[column].entry(1).or_default();
                    *entry = (*entry).max(length);
                    step(&mut column, &mut row, columns_count);
                }
                CellSize::Block { x, y, length } => {
                    if let Some(dependencies) = dependencies.get_mut(column + x - 1) {
                        let entry = dependencies.entry(x).or_default();
                        *entry = (*entry).max(length);
                    }

                    for x in column..(column + x) {
                        for y in row..(row + y) {
                            skip.insert((x, y));
                        }
                    }
                    skip.remove(&(column, row));
                    step(&mut column, &mut row, columns_count);
                }
            }
        }

        let mut columns = vec![0; columns_count + 1];
        for (idx, dependencies) in dependencies.into_iter().enumerate() {
            let idx = idx + 1;
            let mut min = columns[idx - 1];
            for (diff, length) in dependencies.into_iter() {
                min = min.max(columns[idx - diff] + length)
            }
            columns[idx] = min;
        }

        Self {
            index: 0,
            column: 0,
            row: 0,
            columns_count,
            columns,
            cells,
            skip,
        }
    }

    fn step(&mut self, amount: usize) -> bool {
        self.column += amount;
        if self.column >= self.columns_count {
            self.column = 0;
            self.row += 1;
            true
        } else {
            false
        }
    }

    pub fn spacing(&mut self) -> Option<Spacing> {
        let mut pre = 0;
        while self.skip.contains(&(self.column, self.row)) {
            pre += self.columns[self.column + 1] - self.columns[self.column];
            self.step(1);
        }
        let cell = self.cells[self.index];
        self.index += 1;

        match cell {
            CellSize::FullLine => {
                self.step(self.columns_count);
                None
            }
            CellSize::Single(length) => {
                let post = self.columns[self.column + 1] - self.columns[self.column] - length;
                let linebreak = self.step(1);
                Some(Spacing {
                    pre,
                    post,
                    linebreak,
                })
            }
            CellSize::Block { x, y: _, length } => {
                let post = if let Some(&column) = self.columns.get(self.column + x) {
                    column - self.columns[self.column] - length
                } else {
                    0
                };

                let linebreak = self.step(x);
                Some(Spacing {
                    pre,
                    post,
                    linebreak,
                })
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Spacing {
    pre: usize,
    post: usize,
    linebreak: bool,
}

pub fn format_column_args(
    node: &SyntaxNode,
    mut state: State,
    settings: &Settings,
    output: &mut Output<impl OutputTarget>,
    column_argument: &str,
) {
    let mut aligner = Aligner::new(node, column_argument, state, settings, output);
    state.mode = Mode::Items;

    let mut pad = Option::<Spacing>::None;
    for child in node.children() {
        match child.kind() {
            SyntaxKind::LeftParen => {
                format(child, state, settings, output);
                state.indent();
                output.set_whitespace(Whitespace::LineBreak, Priority::Normal);
            }
            SyntaxKind::Comma => {
                if let Some(spacing) = pad.take() {
                    match settings.columns.comma {
                        AlignComma::EndOfContent => {
                            output.set_whitespace(Whitespace::None, Priority::High);
                            format(child, state, settings, output);
                            output.set_whitespace(
                                Whitespace::Spaces(spacing.post + 1),
                                Priority::Normal,
                            );
                        }
                        AlignComma::EndOfCell => {
                            output.set_whitespace(Whitespace::Spaces(spacing.post), Priority::High);
                            format(child, state, settings, output);
                        }
                    }
                    if spacing.linebreak {
                        output.set_whitespace(Whitespace::LineBreak, Priority::High);
                    } else {
                        output.set_whitespace(Whitespace::Space, Priority::Low);
                    }
                } else {
                    format(child, state, settings, output);
                    output.set_whitespace(Whitespace::LineBreak, Priority::High);
                }
            }
            SyntaxKind::RightParen => {
                state.dedent();
                output.set_whitespace(Whitespace::LineBreak, Priority::High);
                format(child, state, settings, output);
            }
            SyntaxKind::Named
            | SyntaxKind::Space
            | SyntaxKind::LineComment
            | SyntaxKind::BlockComment => {
                format(child, state, settings, output);
            }

            _ => {
                pad = aligner.spacing();
                if let Some(Spacing { pre, .. }) = pad {
                    output.emit_whitespace(&state, settings);
                    output.set_whitespace(Whitespace::Spaces(pre), Priority::High);
                }
                format(child, state, settings, output);
            }
        }
    }
}

fn get_column_count(node: &SyntaxNode, column_argument: &str) -> usize {
    for child in node.children() {
        if child.kind() != SyntaxKind::Named {
            continue;
        }
        enum State {
            Start,
            IsColumns,
            Columns(usize),
        }

        let state = child.children().fold(State::Start, |state, sub_child| {
            match (&state, sub_child.kind()) {
                (State::Start, SyntaxKind::Ident) => {
                    if sub_child.text() == column_argument {
                        State::IsColumns
                    } else {
                        State::Start
                    }
                }
                (State::IsColumns, SyntaxKind::Array) => {
                    let count = sub_child
                        .children()
                        .fold(0, |count, value| match value.kind() {
                            SyntaxKind::Auto
                            | SyntaxKind::Int
                            | SyntaxKind::Numeric
                            | SyntaxKind::Float => count + 1,
                            _ => count,
                        });
                    State::Columns(count)
                }
                (State::IsColumns, SyntaxKind::Int) => {
                    State::Columns(sub_child.text().parse().unwrap_or(1))
                }
                _ => state,
            }
        });
        if let State::Columns(value) = state {
            return if value == 0 { 1 } else { value };
        }
    }
    1
}

pub fn format_code_statement(
    node: &SyntaxNode,
    state: State,
    settings: &Settings,
    output: &mut Output<impl OutputTarget>,
) {
    format_default(node, state, settings, output);
    match state.mode {
        Mode::Code => {}
        _ => output.set_whitespace(Whitespace::LineBreak, Priority::Normal),
    }
}

pub fn format_import(
    node: &SyntaxNode,
    state: State,
    settings: &Settings,
    output: &mut Output<impl OutputTarget>,
) {
    for child in node.children() {
        match child.kind() {
            SyntaxKind::Colon => {
                format_optional_padding(child, state, settings, output, &settings.import_statement)
            }
            _ => format(child, state, settings, output),
        }
    }
    match state.mode {
        Mode::Code => {}
        _ => output.set_whitespace(Whitespace::LineBreak, Priority::Normal),
    }
}
