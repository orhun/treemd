use crate::tui::app::{App, Focus};
use crate::tui::theme::Theme;
use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{
    Block, Borders, Clear, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation,
    ScrollbarState, Wrap,
};

pub fn render(frame: &mut Frame, app: &mut App) {
    // Update content metrics before rendering to ensure content height and scroll are correct
    app.update_content_metrics();

    let area = frame.area();

    // Create main layout with title bar and content
    let main_chunks = Layout::vertical([Constraint::Length(2), Constraint::Min(0)]).split(area);

    // Render title bar
    render_title_bar(frame, app, main_chunks[0]);

    // Create horizontal layout for outline and content (conditional based on outline visibility)
    let content_chunks = if app.show_outline {
        let content_width = 100 - app.outline_width;
        Layout::horizontal([
            Constraint::Percentage(app.outline_width),
            Constraint::Percentage(content_width),
        ])
        .split(main_chunks[1])
    } else {
        // Full-width content when outline is hidden
        Layout::horizontal([Constraint::Percentage(100)]).split(main_chunks[1])
    };

    // Render outline (left pane) only if visible
    if app.show_outline {
        render_outline(frame, app, content_chunks[0]);
        // Render content (right pane)
        render_content(frame, app, content_chunks[1]);
    } else {
        // Full-width content
        render_content(frame, app, content_chunks[0]);
    }

    // Render status bar at bottom
    let status_area = Rect {
        x: area.x,
        y: area.height.saturating_sub(1),
        width: area.width,
        height: 1,
    };
    render_status_bar(frame, app, status_area);

    // Render help popup if shown
    if app.show_help {
        render_help_popup(frame, app, area);
    }

    // Render search overlay if shown
    if app.show_search {
        render_search_overlay(frame, app, area);
    }

    // Render theme picker if shown
    if app.show_theme_picker {
        render_theme_picker(frame, app, area);
    }
}

fn render_title_bar(frame: &mut Frame, app: &App, area: Rect) {
    let heading_count = app.document.headings.len();
    let title_text = format!("treemd - {} - {} headings", app.filename, heading_count);

    let title = Paragraph::new(title_text)
        .style(
            Style::default()
                .fg(Color::Rgb(100, 200, 255))
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, area);
}

fn render_outline(frame: &mut Frame, app: &mut App, area: Rect) {
    let theme = &app.theme;

    let items: Vec<ListItem> = app
        .outline_items
        .iter()
        .map(|item| {
            let indent = "  ".repeat(item.level.saturating_sub(1));
            let prefix = "#".repeat(item.level);

            // Show expand/collapse indicator if heading has children
            let expand_indicator = if item.has_children {
                if item.expanded { "▼ " } else { "▶ " }
            } else {
                "  "
            };

            // Show bookmark indicator if this item's text matches the bookmark
            let bookmark_indicator =
                if app.bookmark_position.as_deref() == Some(&item.text) {
                    "⚑ "
                } else {
                    ""
                };

            // Color headings by level using theme
            let color = theme.heading_color(item.level);

            let text = format!(
                "{}{}{}{} {}",
                indent, expand_indicator, bookmark_indicator, prefix, item.text
            );
            let line = Line::from(Span::styled(text, Style::default().fg(color)));

            ListItem::new(line)
        })
        .collect();

    let block_style = theme.border_style(app.focus == Focus::Outline);

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(block_style)
                .title(" Outline "),
        )
        .style(theme.content_style())
        .highlight_style(theme.selection_style())
        .highlight_symbol("► ");

    frame.render_stateful_widget(list, area, &mut app.outline_state);

    // Render scrollbar
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"))
        .style(Style::default().fg(Color::Rgb(80, 80, 100)));

    frame.render_stateful_widget(
        scrollbar,
        area.inner(ratatui::layout::Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut app.outline_scroll_state,
    );
}

fn render_content(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let block_style = theme.border_style(app.focus == Focus::Content);

    // Get content for selected section and determine title
    let (content_text, title) = if let Some(heading_text) = app.selected_heading_text() {
        let content = app.document
            .extract_section(heading_text)
            .unwrap_or_else(|| app.document.content.clone());
        let title = format!(" {} ", heading_text);
        (content, title)
    } else {
        (app.document.content.clone(), " Content ".to_string())
    };

    // Enhanced markdown rendering with syntax highlighting
    let rendered_text = render_markdown_enhanced(&content_text, &app.highlighter, theme);

    let paragraph = Paragraph::new(rendered_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(block_style)
                .title(title),
        )
        .style(theme.content_style())
        .wrap(Wrap { trim: false })
        .scroll((app.content_scroll, 0));

    frame.render_widget(paragraph, area);

    // Render scrollbar
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"))
        .style(Style::default().fg(Color::Rgb(80, 80, 100)));

    frame.render_stateful_widget(
        scrollbar,
        area.inner(ratatui::layout::Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut app.content_scroll_state.clone(),
    );
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let focus_indicator = match app.focus {
        Focus::Outline => "Outline",
        Focus::Content => "Content",
    };

    let selected_idx = app.outline_state.selected().unwrap_or(0);
    let total = app.outline_items.len();
    let percentage = if total > 0 {
        (selected_idx + 1) * 100 / total
    } else {
        0
    };

    let outline_status = if app.show_outline {
        format!("Outline:{}%", app.outline_width)
    } else {
        "Outline:Hidden".to_string()
    };

    let bookmark_indicator = if app.bookmark_position.is_some() {
        " ⚑"
    } else {
        ""
    };

    let status_text = format!(
        " [{}] {}/{} ({}%){} • {} • w:View • []:Size • m:Mark • Copy: y:Text Y:Link • t:Theme • ?:Help ",
        focus_indicator,
        selected_idx + 1,
        total,
        percentage,
        bookmark_indicator,
        outline_status
    );

    let theme_name = format!(" Theme:{} ", app.theme.name);
    let status_text = format!("{}{}", status_text, theme_name);

    let status = Paragraph::new(status_text).style(app.theme.status_bar_style());

    frame.render_widget(status, area);
}

fn render_help_popup(frame: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_area(area, 70, 80);

    // Clear the area
    frame.render_widget(Clear, popup_area);

    let help_text = vec![
        Line::from(vec![Span::styled(
            "treemd - Keyboard Shortcuts",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            "Use j/k or ↓/↑ to scroll | Press Esc or ? to close",
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::ITALIC),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Navigation",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  j/↓      ", Style::default().fg(Color::Yellow)),
            Span::raw("Move down"),
        ]),
        Line::from(vec![
            Span::styled("  k/↑      ", Style::default().fg(Color::Yellow)),
            Span::raw("Move up"),
        ]),
        Line::from(vec![
            Span::styled("  g        ", Style::default().fg(Color::Yellow)),
            Span::raw("Jump to top"),
        ]),
        Line::from(vec![
            Span::styled("  G        ", Style::default().fg(Color::Yellow)),
            Span::raw("Jump to bottom"),
        ]),
        Line::from(vec![
            Span::styled("  d        ", Style::default().fg(Color::Yellow)),
            Span::raw("Page down (content)"),
        ]),
        Line::from(vec![
            Span::styled("  u        ", Style::default().fg(Color::Yellow)),
            Span::raw("Page up (content)"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Tree Operations",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  Enter/Space ", Style::default().fg(Color::Yellow)),
            Span::raw("Toggle expand/collapse"),
        ]),
        Line::from(vec![
            Span::styled("  l/→      ", Style::default().fg(Color::Yellow)),
            Span::raw("Expand heading"),
        ]),
        Line::from(vec![
            Span::styled("  h/←      ", Style::default().fg(Color::Yellow)),
            Span::raw("Collapse (or parent if no children)"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "General",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  Tab      ", Style::default().fg(Color::Yellow)),
            Span::raw("Switch between Outline and Content"),
        ]),
        Line::from(vec![
            Span::styled("  /        ", Style::default().fg(Color::Yellow)),
            Span::raw("Search/filter headings"),
        ]),
        Line::from(vec![
            Span::styled("  ?        ", Style::default().fg(Color::Yellow)),
            Span::raw("Toggle this help"),
        ]),
        Line::from(vec![
            Span::styled("  q/Esc    ", Style::default().fg(Color::Yellow)),
            Span::raw("Quit"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "UX Features",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  w        ", Style::default().fg(Color::Cyan)),
            Span::raw("Toggle outline visibility (full-width content)"),
        ]),
        Line::from(vec![
            Span::styled("  [ ]      ", Style::default().fg(Color::Cyan)),
            Span::raw("Decrease/increase outline width (20%, 30%, 40%)"),
        ]),
        Line::from(vec![
            Span::styled("  1-9      ", Style::default().fg(Color::Cyan)),
            Span::raw("Jump to heading 1-9"),
        ]),
        Line::from(vec![
            Span::styled("  m        ", Style::default().fg(Color::Cyan)),
            Span::raw("Set bookmark (shows ⚑ indicator)"),
        ]),
        Line::from(vec![
            Span::styled("  '        ", Style::default().fg(Color::Cyan)),
            Span::raw("Jump to bookmarked position"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Themes & Clipboard",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  t        ", Style::default().fg(Color::Magenta)),
            Span::raw("Cycle color theme"),
        ]),
        Line::from(vec![
            Span::styled("  y        ", Style::default().fg(Color::Magenta)),
            Span::raw("Copy current section content"),
        ]),
        Line::from(vec![
            Span::styled("  Y        ", Style::default().fg(Color::Magenta)),
            Span::raw("Copy anchor link (#heading-name)"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Use j/k or ↓/↑ to scroll | Press Esc or ? to close",
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::ITALIC),
        )]),
    ];

    let help_text_len = help_text.len();

    let paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Help ")
                .style(Style::default().bg(Color::Rgb(20, 20, 40))),
        )
        .wrap(Wrap { trim: false })
        .scroll((app.help_scroll, 0));

    frame.render_widget(paragraph, popup_area);

    // Render scrollbar for help
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"))
        .style(Style::default().fg(Color::Cyan));

    let mut scrollbar_state = ScrollbarState::new(help_text_len).position(app.help_scroll as usize);

    frame.render_stateful_widget(
        scrollbar,
        popup_area.inner(ratatui::layout::Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );
}

fn centered_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

use crate::tui::syntax::SyntaxHighlighter;

fn render_markdown_enhanced(
    content: &str,
    highlighter: &SyntaxHighlighter,
    theme: &Theme,
) -> Text<'static> {
    let mut lines = Vec::new();
    let mut in_code_block = false;
    let mut code_lang = String::new();
    let mut code_buffer = String::new();

    for line in content.lines() {
        let trimmed = line.trim_start();

        // Handle code blocks
        if trimmed.starts_with("```") {
            if in_code_block {
                // End of code block - highlight accumulated code
                if !code_buffer.is_empty() {
                    let highlighted = highlighter.highlight_code(&code_buffer, &code_lang);
                    lines.extend(highlighted);
                    code_buffer.clear();
                }
                in_code_block = false;
                lines.push(Line::from(vec![Span::styled(
                    line.to_string(),
                    theme.code_fence_style(),
                )]));
            } else {
                // Start of code block
                in_code_block = true;
                code_lang = SyntaxHighlighter::detect_language(&trimmed[3..]);
                lines.push(Line::from(vec![Span::styled(
                    line.to_string(),
                    theme.code_fence_style(),
                )]));
            }
            continue;
        }

        if in_code_block {
            code_buffer.push_str(line);
            code_buffer.push('\n');
            continue;
        }

        // Heading styling
        if trimmed.starts_with('#') {
            let level = trimmed.chars().take_while(|&c| c == '#').count();
            let text = trimmed.trim_start_matches('#').trim();
            let color = theme.heading_color(level);
            lines.push(Line::from(vec![Span::styled(
                format!("{} {}", "#".repeat(level), text),
                Style::default()
                    .fg(color)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            )]));
        }
        // List items
        else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            let text = &trimmed[2..];
            let formatted = format_inline_markdown(text, theme);
            let mut spans = vec![Span::styled("  • ", Style::default().fg(theme.list_bullet))];
            spans.extend(formatted);
            lines.push(Line::from(spans));
        }
        // Numbered lists
        else if trimmed.chars().next().is_some_and(|c| c.is_numeric()) && trimmed.contains(". ")
        {
            let formatted = format_inline_markdown(line, theme);
            lines.push(Line::from(formatted));
        }
        // Blockquotes
        else if trimmed.starts_with('>') {
            let text = trimmed[1..].trim();
            let formatted = format_inline_markdown(text, theme);
            let mut spans = vec![Span::styled(
                "│ ",
                Style::default().fg(theme.blockquote_border),
            )];
            spans.extend(formatted.into_iter().map(|span| {
                Span::styled(
                    span.content,
                    span.style
                        .fg(theme.blockquote_fg)
                        .add_modifier(Modifier::ITALIC),
                )
            }));
            lines.push(Line::from(spans));
        }
        // Regular text with inline formatting
        else {
            let formatted = format_inline_markdown(line, theme);
            lines.push(Line::from(formatted));
        }
    }

    Text::from(lines)
}

fn format_inline_markdown<'a>(text: &str, theme: &Theme) -> Vec<Span<'a>> {
    let mut spans = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Check for inline code `code`
        if chars[i] == '`' {
            if !current.is_empty() {
                spans.push(Span::raw(current.clone()));
                current.clear();
            }
            i += 1;
            let mut code = String::new();
            while i < chars.len() && chars[i] != '`' {
                code.push(chars[i]);
                i += 1;
            }
            if i < chars.len() {
                i += 1; // Skip closing `
            }
            spans.push(Span::styled(code, theme.inline_code_style()));
        }
        // Check for bold **text**
        else if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '*' {
            if !current.is_empty() {
                spans.push(Span::raw(current.clone()));
                current.clear();
            }
            i += 2;
            let mut bold_text = String::new();
            while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '*') {
                bold_text.push(chars[i]);
                i += 1;
            }
            if i + 1 < chars.len() {
                i += 2; // Skip closing **
            }
            spans.push(Span::styled(bold_text, theme.bold_style()));
        }
        // Check for italic *text*
        else if chars[i] == '*' {
            if !current.is_empty() {
                spans.push(Span::raw(current.clone()));
                current.clear();
            }
            i += 1;
            let mut italic_text = String::new();
            while i < chars.len() && chars[i] != '*' {
                italic_text.push(chars[i]);
                i += 1;
            }
            if i < chars.len() {
                i += 1; // Skip closing *
            }
            spans.push(Span::styled(italic_text, theme.italic_style()));
        } else {
            current.push(chars[i]);
            i += 1;
        }
    }

    if !current.is_empty() {
        spans.push(Span::styled(current, theme.text_style()));
    }

    if spans.is_empty() {
        spans.push(Span::styled(text.to_string(), theme.text_style()));
    }

    spans
}

fn render_search_overlay(frame: &mut Frame, app: &App, area: Rect) {
    let search_area = Rect {
        x: area.x + 2,
        y: area.y + 2,
        width: area.width.saturating_sub(4).max(40),
        height: 3,
    };

    frame.render_widget(Clear, search_area);

    let search_text = format!("Search: {}_", app.search_query);
    let paragraph = Paragraph::new(search_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .title(" Filter Headings ")
                .style(Style::default().bg(Color::Rgb(30, 30, 50))),
        )
        .style(Style::default().fg(Color::White));

    frame.render_widget(paragraph, search_area);
}

fn render_theme_picker(frame: &mut Frame, app: &App, area: Rect) {
    use crate::tui::theme::ThemeName;

    // All available themes
    let themes = [
        (
            ThemeName::OceanDark,
            "Ocean Dark",
            "Base16 Ocean with cool blues",
        ),
        (ThemeName::Nord, "Nord", "Arctic, north-bluish palette"),
        (
            ThemeName::Dracula,
            "Dracula",
            "Dark theme with vibrant colors",
        ),
        (
            ThemeName::Solarized,
            "Solarized",
            "Precision colors for machines and people",
        ),
        (
            ThemeName::Monokai,
            "Monokai",
            "Sublime Text's iconic scheme",
        ),
        (ThemeName::Gruvbox, "Gruvbox", "Retro groove color scheme"),
        (
            ThemeName::TokyoNight,
            "Tokyo Night",
            "Modern night theme for low-light",
        ),
        (
            ThemeName::CatppuccinMocha,
            "Catppuccin Mocha",
            "Soothing pastel theme for night coding",
        ),
    ];

    // Create centered popup area
    let popup_area = centered_area(area, 60, 50);

    // Clear background
    frame.render_widget(Clear, popup_area);

    // Create lines for each theme
    let mut lines = vec![
        Line::from(vec![Span::styled(
            "Select Theme (j/k to navigate, Enter to apply, Esc to cancel)",
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::ITALIC),
        )]),
        Line::from(""),
    ];

    for (idx, (theme_name, name, description)) in themes.iter().enumerate() {
        let is_selected = idx == app.theme_picker_selected;
        let is_current = *theme_name == app.current_theme;

        let (prefix, style) = if is_selected {
            (
                "▶ ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            ("  ", Style::default().fg(Color::White))
        };

        let current_marker = if is_current { " ✓" } else { "" };
        let line_text = format!("{}{}{}", prefix, name, current_marker);

        lines.push(Line::from(vec![Span::styled(line_text, style)]));

        // Add description on next line if selected
        if is_selected {
            lines.push(Line::from(vec![Span::styled(
                format!("  {}", description),
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::ITALIC),
            )]));
        }
    }

    lines.push(Line::from(""));

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Theme Selector ")
                .style(Style::default().bg(Color::Rgb(20, 20, 30))),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, popup_area);
}
