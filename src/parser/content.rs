//! Content parsing for markdown sections
//!
//! Parses markdown content into semantic blocks and inline elements.

use super::output::{Alignment, Block, InlineElement, ListItem};
use pulldown_cmark::{Alignment as CmarkAlignment, CodeBlockKind, Event, Parser, Tag, TagEnd};

/// Parse markdown content into structured blocks
pub fn parse_content(markdown: &str, start_line: usize) -> Vec<Block> {
    let parser = Parser::new(markdown);
    let mut blocks = Vec::new();
    let mut state = ParserState::new(start_line);

    for event in parser {
        process_event(event, &mut state, &mut blocks);
    }

    // Flush any pending block
    state.finalize(&mut blocks);
    blocks
}

struct ParserState {
    current_line: usize,
    paragraph_buffer: String,
    inline_buffer: Vec<InlineElement>,
    list_items: Vec<ListItem>,
    list_ordered: bool,
    code_buffer: String,
    code_language: Option<String>,
    code_start_line: usize,
    blockquote_buffer: String,
    table_headers: Vec<String>,
    table_alignments: Vec<Alignment>,
    table_rows: Vec<Vec<String>>,
    current_row: Vec<String>,
    in_paragraph: bool,
    in_list: bool,
    in_code: bool,
    in_blockquote: bool,
    in_table: bool,
    in_strong: bool,
    in_emphasis: bool,
    in_strikethrough: bool,
    in_code_inline: bool,
    in_link: bool,
    link_url: String,
    link_text: String,
}

impl ParserState {
    fn new(start_line: usize) -> Self {
        Self {
            current_line: start_line,
            paragraph_buffer: String::new(),
            inline_buffer: Vec::new(),
            list_items: Vec::new(),
            list_ordered: false,
            code_buffer: String::new(),
            code_language: None,
            code_start_line: 0,
            blockquote_buffer: String::new(),
            table_headers: Vec::new(),
            table_alignments: Vec::new(),
            table_rows: Vec::new(),
            current_row: Vec::new(),
            in_paragraph: false,
            in_list: false,
            in_code: false,
            in_blockquote: false,
            in_table: false,
            in_strong: false,
            in_emphasis: false,
            in_strikethrough: false,
            in_code_inline: false,
            in_link: false,
            link_url: String::new(),
            link_text: String::new(),
        }
    }

    fn finalize(&mut self, blocks: &mut Vec<Block>) {
        self.flush_paragraph(blocks);
        self.flush_list(blocks);
        self.flush_code(blocks);
        self.flush_blockquote(blocks);
        self.flush_table(blocks);
    }

    fn flush_paragraph(&mut self, blocks: &mut Vec<Block>) {
        if self.in_paragraph && !self.paragraph_buffer.is_empty() {
            blocks.push(Block::Paragraph {
                content: self.paragraph_buffer.clone(),
                inline: self.inline_buffer.clone(),
            });
            self.paragraph_buffer.clear();
            self.inline_buffer.clear();
            self.in_paragraph = false;
        }
    }

    fn flush_list(&mut self, blocks: &mut Vec<Block>) {
        if self.in_list && !self.list_items.is_empty() {
            blocks.push(Block::List {
                ordered: self.list_ordered,
                items: self.list_items.clone(),
            });
            self.list_items.clear();
            self.in_list = false;
        }
    }

    fn flush_code(&mut self, blocks: &mut Vec<Block>) {
        if self.in_code && !self.code_buffer.is_empty() {
            blocks.push(Block::Code {
                language: self.code_language.clone(),
                content: self.code_buffer.trim_end().to_string(),
                start_line: self.code_start_line,
                end_line: self.current_line,
            });
            self.code_buffer.clear();
            self.code_language = None;
            self.in_code = false;
        }
    }

    fn flush_blockquote(&mut self, blocks: &mut Vec<Block>) {
        if self.in_blockquote && !self.blockquote_buffer.is_empty() {
            let nested_blocks = parse_content(&self.blockquote_buffer, self.current_line);
            blocks.push(Block::Blockquote {
                content: self.blockquote_buffer.clone(),
                blocks: nested_blocks,
            });
            self.blockquote_buffer.clear();
            self.in_blockquote = false;
        }
    }

    fn flush_table(&mut self, blocks: &mut Vec<Block>) {
        if self.in_table && !self.table_headers.is_empty() {
            blocks.push(Block::Table {
                headers: self.table_headers.clone(),
                alignments: self.table_alignments.clone(),
                rows: self.table_rows.clone(),
            });
            self.table_headers.clear();
            self.table_alignments.clear();
            self.table_rows.clear();
            self.in_table = false;
        }
    }

    fn add_inline_text(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }

        let element = if self.in_code_inline {
            InlineElement::Code {
                value: text.to_string(),
            }
        } else if self.in_strong {
            InlineElement::Strong {
                value: text.to_string(),
            }
        } else if self.in_emphasis {
            InlineElement::Emphasis {
                value: text.to_string(),
            }
        } else if self.in_strikethrough {
            InlineElement::Strikethrough {
                value: text.to_string(),
            }
        } else {
            InlineElement::Text {
                value: text.to_string(),
            }
        };

        self.inline_buffer.push(element);
        self.paragraph_buffer.push_str(text);
    }
}

#[allow(clippy::too_many_lines)]
fn process_event(event: Event, state: &mut ParserState, blocks: &mut Vec<Block>) {
    match event {
        Event::Start(Tag::Paragraph) => {
            state.in_paragraph = true;
        }
        Event::End(TagEnd::Paragraph) => {
            state.flush_paragraph(blocks);
        }
        Event::Start(Tag::CodeBlock(kind)) => {
            state.in_code = true;
            state.code_start_line = state.current_line;
            state.code_language = match kind {
                CodeBlockKind::Fenced(lang) => {
                    if lang.is_empty() {
                        None
                    } else {
                        Some(lang.to_string())
                    }
                }
                CodeBlockKind::Indented => None,
            };
        }
        Event::End(TagEnd::CodeBlock) => {
            state.flush_code(blocks);
        }
        Event::Start(Tag::List(start_number)) => {
            state.in_list = true;
            state.list_ordered = start_number.is_some();
        }
        Event::End(TagEnd::List(_)) => {
            state.flush_list(blocks);
        }
        Event::Start(Tag::Item) => {
            state.paragraph_buffer.clear();
            state.inline_buffer.clear();
        }
        Event::End(TagEnd::Item) => {
            state.list_items.push(ListItem {
                checked: None,
                content: state.paragraph_buffer.clone(),
                inline: state.inline_buffer.clone(),
            });
            state.paragraph_buffer.clear();
            state.inline_buffer.clear();
        }
        Event::Start(Tag::BlockQuote(_)) => {
            state.in_blockquote = true;
        }
        Event::End(TagEnd::BlockQuote(_)) => {
            state.flush_blockquote(blocks);
        }
        Event::Start(Tag::Table(alignments)) => {
            state.in_table = true;
            state.table_alignments = alignments
                .iter()
                .map(|a| match a {
                    CmarkAlignment::Left => Alignment::Left,
                    CmarkAlignment::Center => Alignment::Center,
                    CmarkAlignment::Right => Alignment::Right,
                    CmarkAlignment::None => Alignment::None,
                })
                .collect();
        }
        Event::End(TagEnd::Table) => {
            state.flush_table(blocks);
        }
        Event::Start(Tag::TableHead) => {}
        Event::End(TagEnd::TableHead) => {
            state.table_headers = state.current_row.clone();
            state.current_row.clear();
        }
        Event::Start(Tag::TableRow) => {}
        Event::End(TagEnd::TableRow) => {
            state.table_rows.push(state.current_row.clone());
            state.current_row.clear();
        }
        Event::Start(Tag::TableCell) => {
            state.paragraph_buffer.clear();
        }
        Event::End(TagEnd::TableCell) => {
            state.current_row.push(state.paragraph_buffer.clone());
            state.paragraph_buffer.clear();
        }
        Event::Start(Tag::Strong) => {
            state.in_strong = true;
        }
        Event::End(TagEnd::Strong) => {
            state.in_strong = false;
        }
        Event::Start(Tag::Emphasis) => {
            state.in_emphasis = true;
        }
        Event::End(TagEnd::Emphasis) => {
            state.in_emphasis = false;
        }
        Event::Start(Tag::Strikethrough) => {
            state.in_strikethrough = true;
        }
        Event::End(TagEnd::Strikethrough) => {
            state.in_strikethrough = false;
        }
        Event::Code(text) => {
            state.in_code_inline = true;
            state.add_inline_text(&text);
            state.in_code_inline = false;
        }
        Event::Start(Tag::Link { dest_url, .. }) => {
            state.in_link = true;
            state.link_url = dest_url.to_string();
            state.link_text.clear();
        }
        Event::End(TagEnd::Link) => {
            state.in_link = false;
            state.inline_buffer.push(InlineElement::Link {
                text: state.link_text.clone(),
                url: state.link_url.clone(),
                title: None,
            });
            state
                .paragraph_buffer
                .push_str(&format!("[{}]({})", state.link_text, state.link_url));
            state.link_text.clear();
            state.link_url.clear();
        }
        Event::Start(Tag::Image { .. }) => {
            // Images are handled through Image event
        }
        Event::End(TagEnd::Image) => {}
        Event::Text(text) => {
            if state.in_code {
                state.code_buffer.push_str(&text);
            } else if state.in_blockquote {
                state.blockquote_buffer.push_str(&text);
            } else if state.in_link {
                state.link_text.push_str(&text);
            } else {
                state.add_inline_text(&text);
            }
        }
        Event::SoftBreak => {
            if state.in_paragraph {
                state.paragraph_buffer.push(' ');
                state.inline_buffer.push(InlineElement::Text {
                    value: " ".to_string(),
                });
            }
        }
        Event::HardBreak => {
            if state.in_paragraph {
                state.paragraph_buffer.push('\n');
                state.inline_buffer.push(InlineElement::Text {
                    value: "\n".to_string(),
                });
            }
        }
        Event::Rule => {
            state.flush_paragraph(blocks);
            blocks.push(Block::HorizontalRule);
        }
        _ => {}
    }
}

/// Generate URL-friendly slug from heading text
pub fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '-' {
                '-'
            } else {
                '\0'
            }
        })
        .filter(|&c| c != '\0')
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
