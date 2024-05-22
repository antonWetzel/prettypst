// mod code;
// mod markup;
// mod math;

// use code::*;
// use markup::*;
// use math::*;

use std::ops::Not;

use typst_syntax::{SyntaxKind, SyntaxNode};

use crate::{
    output::{Output, OutputTarget, Whitespace},
    settings::*,
    state::{Mode, State},
};

pub fn format(
    node: &SyntaxNode,
    mut state: State,
    settings: &Settings,
    output: &mut Output<impl OutputTarget>,
) {
    if node.erroneous()
        && node
            .children()
            .flat_map(|child| child.children())
            .any(|child| child.kind() == SyntaxKind::Error)
    {
        return skip_formatting(node, state, settings, output);
    }

    output.text(node.text(), &state, settings);
    let mut children = node.children();
    let Some(mut last) =
        children.find(|node| matches!(node.kind(), SyntaxKind::Space | SyntaxKind::Parbreak).not())
    else {
        return;
    };
    format(last, state, settings, output);
    let mut ws = Whitespace::None;
    for child in children {
        match child.kind() {
            SyntaxKind::Space | SyntaxKind::Parbreak => {
                ws = Whitespace::new(child.text());
            }
            _ => {
                whitespace(last, ws, child, &mut state, settings, output);
                format(child, state, settings, output);
                ws = Whitespace::None;
                last = child
            }
        }
    }
}

fn skip_formatting(
    node: &SyntaxNode,
    state: State,
    settings: &Settings,
    output: &mut Output<impl OutputTarget>,
) {
    output.text(node.text(), &state, settings);
    for child in node.children() {
        skip_formatting(child, state, settings, output);
    }
}

fn whitespace(
    left: &SyntaxNode,
    whitespace: Whitespace,
    right: &SyntaxNode,
    state: &mut State,
    settings: &Settings,
    output: &mut Output<impl OutputTarget>,
) {
    use SyntaxKind as K;
    use Whitespace as W;

    let ws = match (left.kind(), whitespace, right.kind()) {
        (_, _, K::Heading) => W::LineBreaks(settings.heading.blank_lines_before + 1),
        (K::Heading, _, _) => W::LineBreaks(settings.heading.blank_lines_after + 1),

        (K::Hash, _, _) => W::None,
        (_, _, K::Colon) => W::None,
        (K::Colon, _, _) => W::Space,

        (l, _, K::Semicolon) if l.is_stmt() => W::None,
        (l, ws, _) if l.is_stmt() => ws.as_linebreak(),
        (K::Semicolon, ws, _) => ws.limit(),

        (K::LineComment, ws, _) => ws.as_linebreak(),
        (_, ws, K::LineComment) => ws.limit(),
        (K::BlockComment, ws, _) => ws.limit(),
        (_, ws, K::BlockComment) => ws.limit(),

        (_, _, K::Args) => {
            if has_linebreak(right) {
                state.mode = Mode::MultilineArgs;
            } else {
                state.mode = Mode::SinglelineArgs;
            }
            W::None
        }

        (K::LeftParen, _, _) => match state.mode {
            Mode::MultilineArgs => {
                state.indent();
                W::LineBreak
            }
            _ => W::None,
        },
        (_, _, K::RightParen) => match state.mode {
            Mode::MultilineArgs => {
                state.dedent();
                W::LineBreak
            }
            _ => W::None,
        },

        (K::LeftBracket, _, K::Markup) => {
            if has_linebreak(right) {
                state.indent();
                state.mode = Mode::MultilineMarkdown;
                W::LineBreak
            } else {
                state.mode = Mode::SinglelineMarkdown;
                starting_whitespace(right)
            }
        }
        (K::Markup, _, K::RightBracket) => match state.mode {
            Mode::MultilineMarkdown => {
                state.dedent();
                W::LineBreak
            }
            _ => ending_whitespace(left),
        },

        (_, _, K::Comma) => W::None,
        (K::Comma, ws, _) => match state.mode {
            Mode::MultilineArgs => ws.as_linebreak(),
            _ => W::Space,
        },

        (_, _, K::Code) => {
            if has_linebreak(right) {
                state.mode = Mode::MultilineCode;
                state.indent();
                W::LineBreak
            } else {
                state.mode = Mode::SinglelineCode;
                W::Space
            }
        }
        (K::Code, _, _) => match state.mode {
            Mode::SinglelineCode => W::Space,
            Mode::MultilineCode => {
                state.dedent();
                W::LineBreak
            }
            _ => panic!(),
        },

        (_, W::LineBreak, _) if state.mode.preserve_linebreak() => W::LineBreak,
        (_, W::LineBreaks(_), _) if state.mode.preserve_linebreaks() => W::LineBreaks(2),

        (_, _, _) => W::Space,
    };
    output.emit_whitespace(ws)
}

fn has_linebreak(node: &SyntaxNode) -> bool {
    node.children()
        .filter_map(|node| {
            matches!(node.kind(), SyntaxKind::Space | SyntaxKind::Parbreak)
                .then(|| Whitespace::new(node.text()))
        })
        .any(|ws| matches!(ws, Whitespace::LineBreak | Whitespace::LineBreaks(_)))
}

fn starting_whitespace(node: &SyntaxNode) -> Whitespace {
    let Some(node) = node.children().next() else {
        return Whitespace::None;
    };
    match node.kind() {
        SyntaxKind::Space | SyntaxKind::Parbreak => Whitespace::new(node.text()).limit(),
        _ => Whitespace::None,
    }
}

fn ending_whitespace(node: &SyntaxNode) -> Whitespace {
    let Some(node) = node.children().last() else {
        return Whitespace::None;
    };
    match node.kind() {
        SyntaxKind::Space | SyntaxKind::Parbreak => Whitespace::new(node.text()).limit(),
        _ => Whitespace::None,
    }
}
