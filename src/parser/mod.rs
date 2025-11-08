mod document;

pub use document::{Document, Heading, HeadingNode};

use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use std::path::Path;

/// Parse a markdown file and extract its structure
pub fn parse_file(path: &Path) -> std::io::Result<Document> {
    let content = std::fs::read_to_string(path)?;
    Ok(parse_markdown(&content))
}

/// Parse markdown content and extract headings
pub fn parse_markdown(content: &str) -> Document {
    let parser = Parser::new(content);
    let mut headings = Vec::new();
    let mut current_heading: Option<(usize, String)> = None;
    let mut in_heading = false;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                in_heading = true;
                current_heading = Some((level as usize, String::new()));
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some((level, text)) = current_heading.take() {
                    headings.push(Heading {
                        level,
                        text: text.trim().to_string(),
                    });
                }
                in_heading = false;
            }
            Event::Text(text) if in_heading => {
                if let Some((_, ref mut heading_text)) = current_heading {
                    heading_text.push_str(&text);
                }
            }
            _ => {}
        }
    }

    Document::new(content.to_string(), headings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_headings() {
        let md = r#"# Title
Some content

## Section 1
More content

### Subsection
Details

## Section 2
End"#;

        let doc = parse_markdown(md);
        assert_eq!(doc.headings.len(), 4);
        assert_eq!(doc.headings[0].level, 1);
        assert_eq!(doc.headings[0].text, "Title");
        assert_eq!(doc.headings[1].level, 2);
        assert_eq!(doc.headings[1].text, "Section 1");
    }
}
