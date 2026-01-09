use super::*;

pub fn format_markup(
    node: &SyntaxNode,
    state: State,
    settings: &Settings,
    output: &mut Output<impl OutputTarget>,
) {
    let mut disabled = false;
    for child in node.children() {
        if (child.kind() == SyntaxKind::LineComment || child.kind() == SyntaxKind::BlockComment)
            && child.text().contains("prettypst")
        {
            if child.text().contains("disable") {
                disabled = true;
            } else if child.text().contains("enable") {
                disabled = false;
            }
        }
        if disabled {
            skip_formatting(child, state, settings, output);
        } else {
            format(child, state, settings, output);
        }
    }
}

pub fn format_content_block(
    node: &SyntaxNode,
    mut state: State,
    settings: &Settings,
    output: &mut Output<impl OutputTarget>,
) {
    let mut start_space = false;
    let mut end_space = false;
    let mut linebreak = false;
    for child in node.children() {
        if child.kind() != SyntaxKind::Markup {
            continue;
        }
        if let Some(node) = child.children().next()
            && matches!(node.kind(), SyntaxKind::Space | SyntaxKind::Parbreak)
        {
            start_space = true;
            if node.text().contains('\n') {
                linebreak = true;
            }
        }
        if let Some(node) = child.children().next_back()
            && matches!(node.kind(), SyntaxKind::Space | SyntaxKind::Parbreak)
        {
            end_space = true;
            if node.text().contains('\n') {
                linebreak = true;
            }
        }
    }
    let single = !start_space || !end_space || !linebreak;
    state.mode = match settings.automatic_newline.max_width {
        0 => Mode::Markup,
        _ => Mode::MarkupBreakable,
    };

    for child in node.children() {
        match child.kind() {
            SyntaxKind::LeftBracket => {
                format(child, state, settings, output);
                if single {
                    if start_space {
                        output.set_whitespace(Whitespace::Space, Priority::Guaranteed);
                    } else {
                        output.set_whitespace(Whitespace::None, Priority::Guaranteed);
                    }
                } else {
                    state.indent();
                    match settings.block.long_block_style {
                        LongBlockStyle::Compact => {
                            output.set_whitespace(Whitespace::Space, Priority::Low)
                        }
                        LongBlockStyle::Separate => {
                            output.set_whitespace(Whitespace::LineBreak, Priority::Normal)
                        }
                    }
                }
            }

            SyntaxKind::RightBracket => {
                if single {
                    if end_space {
                        output.set_whitespace(Whitespace::Space, Priority::Guaranteed);
                    } else {
                        output.set_whitespace(Whitespace::None, Priority::Guaranteed);
                    }
                } else {
                    state.dedent();
                    match settings.block.long_block_style {
                        LongBlockStyle::Compact => {
                            output.set_whitespace(Whitespace::Space, Priority::Low)
                        }
                        LongBlockStyle::Separate => {
                            output.set_whitespace(Whitespace::LineBreak, Priority::Normal)
                        }
                    }
                }
                format(child, state, settings, output);
            }
            _ => format(child, state, settings, output),
        }
    }
}

pub fn format_text(
    node: &SyntaxNode,
    state: State,
    settings: &Settings,
    output: &mut Output<impl OutputTarget>,
) {
    match state.mode {
        Mode::MarkupBreakable => {
            let mut iter = node.text().split(' ');
            if let Whitespace::None = output.get_whitespace().0
                && let Some(word) = iter.next()
            {
                output.raw_text(word, &state, settings);
            }

            for word in iter {
                let fixpoint = output.create_fixpoint();
                output.raw_text(word, &state, settings);
                output.set_fixpoint(fixpoint);
                if output.position().1 > 80 {
                    output.set_whitespace(Whitespace::LineBreak, Priority::Normal);
                } else {
                    output.set_whitespace(Whitespace::Space, Priority::Normal);
                }
                output.raw_text(word, &state, settings);
            }
        }
        _ => output.raw(node, &state, settings),
    }
}

pub fn format_enclosed(
    node: &SyntaxNode,
    mut state: State,
    settings: &Settings,
    output: &mut Output<impl OutputTarget>,
) {
    state.mode = match node.kind() {
        SyntaxKind::Strong if settings.automatic_newline.in_strong => Mode::MarkupBreakable,
        SyntaxKind::Emph if settings.automatic_newline.in_emphasis => Mode::MarkupBreakable,
        _ => Mode::Markup,
    };
    for child in node.children() {
        format(child, state, settings, output);
    }
}

pub fn format_heading(
    node: &SyntaxNode,
    state: State,
    settings: &Settings,
    output: &mut Output<impl OutputTarget>,
) {
    output.set_whitespace(
        Whitespace::LineBreaks(settings.heading.blank_lines_before + 1),
        Priority::High,
    );
    format_default(node, state, settings, output);
    output.set_whitespace(
        Whitespace::LineBreaks(settings.heading.blank_lines_after + 1),
        Priority::High,
    );
}

pub fn format_label(
    node: &SyntaxNode,
    state: State,
    settings: &Settings,
    output: &mut Output<impl OutputTarget>,
) {
    match state.mode {
        Mode::Markup | Mode::MarkupBreakable => {
            let (whitespace, priority) = output.get_whitespace();
            if settings.separate_label {
                output.set_whitespace(Whitespace::Space, Priority::Guaranteed);
            } else {
                output.set_whitespace(Whitespace::None, Priority::Guaranteed);
            }
            output.raw(node, &state, settings);
            output.set_whitespace(whitespace, priority);
        }
        _ => output.raw(node, &state, settings),
    }
}

pub fn format_term(
    node: &SyntaxNode,
    mut state: State,
    settings: &Settings,
    output: &mut Output<impl OutputTarget>,
) {
    for child in node.children() {
        match child.kind() {
            SyntaxKind::Colon => {
                format_optional_padding(child, state, settings, output, &settings.term);
                state.indent();
            }
            _ => format(child, state, settings, output),
        }
    }
    output.set_whitespace(Whitespace::LineBreak, Priority::High);
}

pub fn format_end_of_file(
    _node: &SyntaxNode,
    _state: State,
    settings: &Settings,
    output: &mut Output<impl OutputTarget>,
) {
    if settings.final_newline {
        output.set_whitespace(Whitespace::LineBreak, Priority::Guaranteed);
    } else {
        output.set_whitespace(Whitespace::None, Priority::Guaranteed);
    }
}
