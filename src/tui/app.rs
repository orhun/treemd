use crate::parser::{Document, HeadingNode};
use crate::tui::syntax::SyntaxHighlighter;
use crate::tui::theme::{Theme, ThemeName};
use ratatui::widgets::{ListState, ScrollbarState};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Focus {
    Outline,
    Content,
}

pub struct App {
    pub document: Document,
    pub tree: Vec<HeadingNode>,
    pub outline_state: ListState,
    pub outline_scroll_state: ScrollbarState,
    pub focus: Focus,
    pub outline_items: Vec<OutlineItem>,
    pub content_scroll: u16,
    pub content_scroll_state: ScrollbarState,
    pub content_height: u16,
    pub show_help: bool,
    pub help_scroll: u16,
    pub show_search: bool,
    pub search_query: String,
    pub highlighter: SyntaxHighlighter,
    pub show_outline: bool,
    pub outline_width: u16,                // Percentage: 20, 30, or 40
    pub bookmark_position: Option<String>, // Bookmarked heading text (was: outline position)
    collapsed_headings: HashSet<String>,   // Track which headings are collapsed by text
    pub current_theme: ThemeName,
    pub theme: Theme,
    pub show_theme_picker: bool,
    pub theme_picker_selected: usize,
    previous_selection: Option<String>, // Track previous selection to detect changes
}

#[derive(Debug, Clone)]
pub struct OutlineItem {
    pub level: usize,
    pub text: String,
    pub expanded: bool,
    pub has_children: bool, // Track if this heading has children in the tree
}

impl App {
    pub fn new(document: Document) -> Self {
        let tree = document.build_tree();
        let collapsed_headings = HashSet::new();
        let outline_items = Self::flatten_tree(&tree, &collapsed_headings);

        let mut outline_state = ListState::default();
        if !outline_items.is_empty() {
            outline_state.select(Some(0));
        }

        let content_lines = document.content.lines().count();

        let current_theme = ThemeName::OceanDark;
        let theme = Theme::from_name(current_theme);

        Self {
            document,
            tree,
            outline_state,
            outline_scroll_state: ScrollbarState::new(outline_items.len()),
            focus: Focus::Outline,
            outline_items,
            content_scroll: 0,
            content_scroll_state: ScrollbarState::new(content_lines),
            content_height: content_lines as u16,
            show_help: false,
            help_scroll: 0,
            show_search: false,
            search_query: String::new(),
            highlighter: SyntaxHighlighter::new(),
            show_outline: true,
            outline_width: 30,
            bookmark_position: None,
            collapsed_headings,
            current_theme,
            theme,
            show_theme_picker: false,
            theme_picker_selected: 0,
            previous_selection: None,
        }
    }

    fn flatten_tree(
        tree: &[HeadingNode],
        collapsed_headings: &HashSet<String>,
    ) -> Vec<OutlineItem> {
        let mut items = Vec::new();

        fn flatten_recursive(
            node: &HeadingNode,
            items: &mut Vec<OutlineItem>,
            collapsed_headings: &HashSet<String>,
        ) {
            let is_collapsed = collapsed_headings.contains(&node.heading.text);
            let expanded = !is_collapsed;
            let has_children = !node.children.is_empty();

            items.push(OutlineItem {
                level: node.heading.level,
                text: node.heading.text.clone(),
                expanded,
                has_children,
            });

            // Only show children if this node is expanded
            if expanded {
                for child in &node.children {
                    flatten_recursive(child, items, collapsed_headings);
                }
            }
        }

        for node in tree {
            flatten_recursive(node, &mut items, collapsed_headings);
        }

        items
    }

    /// Select a heading by its text. Returns true if found and selected.
    fn select_by_text(&mut self, text: &str) -> bool {
        for (idx, item) in self.outline_items.iter().enumerate() {
            if item.text == text {
                self.outline_state.select(Some(idx));
                self.outline_scroll_state = self.outline_scroll_state.position(idx);
                return true;
            }
        }
        false
    }

    /// Update content height based on current selection and reset scroll if selection changed
    pub fn update_content_metrics(&mut self) {
        let current_selection = self.selected_heading_text().map(|s| s.to_string());

        // Check if selection changed
        if current_selection != self.previous_selection {
            // Reset content scroll when selection changes
            self.content_scroll = 0;
            self.previous_selection = current_selection.clone();
        }

        // Update content height based on current section
        let content_text = if let Some(heading_text) = &current_selection {
            if let Some(_heading) = self.document.find_heading(heading_text) {
                // Use extract_section_content to get the actual displayed content
                self.document
                    .extract_section(heading_text)
                    .unwrap_or_else(|| self.document.content.clone())
            } else {
                self.document.content.clone()
            }
        } else {
            self.document.content.clone()
        };

        let content_lines = content_text.lines().count();
        self.content_height = content_lines as u16;
        self.content_scroll_state =
            ScrollbarState::new(content_lines).position(self.content_scroll as usize);
    }

    pub fn next(&mut self) {
        if self.focus == Focus::Outline {
            let i = match self.outline_state.selected() {
                Some(i) => {
                    if i >= self.outline_items.len().saturating_sub(1) {
                        i
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.outline_state.select(Some(i));
            self.outline_scroll_state = self.outline_scroll_state.position(i);
        } else {
            // Scroll content
            let new_scroll = self.content_scroll.saturating_add(1);
            if new_scroll < self.content_height {
                self.content_scroll = new_scroll;
                self.content_scroll_state = self.content_scroll_state.position(new_scroll as usize);
            }
        }
    }

    pub fn previous(&mut self) {
        if self.focus == Focus::Outline {
            let i = match self.outline_state.selected() {
                Some(i) => i.saturating_sub(1),
                None => 0,
            };
            self.outline_state.select(Some(i));
            self.outline_scroll_state = self.outline_scroll_state.position(i);
        } else {
            // Scroll content
            self.content_scroll = self.content_scroll.saturating_sub(1);
            self.content_scroll_state = self
                .content_scroll_state
                .position(self.content_scroll as usize);
        }
    }

    pub fn first(&mut self) {
        if self.focus == Focus::Outline && !self.outline_items.is_empty() {
            self.outline_state.select(Some(0));
            self.outline_scroll_state = self.outline_scroll_state.position(0);
        } else {
            self.content_scroll = 0;
            self.content_scroll_state = self.content_scroll_state.position(0);
        }
    }

    pub fn last(&mut self) {
        if self.focus == Focus::Outline && !self.outline_items.is_empty() {
            let last = self.outline_items.len() - 1;
            self.outline_state.select(Some(last));
            self.outline_scroll_state = self.outline_scroll_state.position(last);
        } else {
            let last = self.content_height.saturating_sub(1);
            self.content_scroll = last;
            self.content_scroll_state = self.content_scroll_state.position(last as usize);
        }
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
        if self.show_help {
            self.help_scroll = 0; // Reset scroll when opening help
        }
    }

    pub fn scroll_help_down(&mut self) {
        self.help_scroll = self.help_scroll.saturating_add(1);
    }

    pub fn scroll_help_up(&mut self) {
        self.help_scroll = self.help_scroll.saturating_sub(1);
    }

    pub fn toggle_search(&mut self) {
        self.show_search = !self.show_search;
        if !self.show_search {
            self.search_query.clear();
        }
    }

    pub fn search_input(&mut self, c: char) {
        self.search_query.push(c);
        self.filter_outline();
    }

    pub fn search_backspace(&mut self) {
        self.search_query.pop();
        self.filter_outline();
    }

    fn filter_outline(&mut self) {
        // Save current selection text
        let current_selection = self.selected_heading_text().map(|s| s.to_string());

        if self.search_query.is_empty() {
            // Reset to full tree
            self.outline_items = Self::flatten_tree(&self.tree, &self.collapsed_headings);
        } else {
            // Filter by search query
            let query_lower = self.search_query.to_lowercase();
            self.outline_items = Self::flatten_tree(&self.tree, &self.collapsed_headings)
                .into_iter()
                .filter(|item| item.text.to_lowercase().contains(&query_lower))
                .collect();
        }

        // Try to restore previous selection, otherwise select first item
        if !self.outline_items.is_empty() {
            let restored = if let Some(text) = current_selection {
                self.select_by_text(&text)
            } else {
                false
            };

            if !restored {
                self.outline_state.select(Some(0));
                self.outline_scroll_state =
                    ScrollbarState::new(self.outline_items.len()).position(0);
            }
        }
    }

    pub fn scroll_page_down(&mut self) {
        if self.focus == Focus::Content {
            let new_scroll = self.content_scroll.saturating_add(10);
            self.content_scroll = new_scroll.min(self.content_height.saturating_sub(1));
            self.content_scroll_state = self
                .content_scroll_state
                .position(self.content_scroll as usize);
        }
    }

    pub fn scroll_page_up(&mut self) {
        if self.focus == Focus::Content {
            self.content_scroll = self.content_scroll.saturating_sub(10);
            self.content_scroll_state = self
                .content_scroll_state
                .position(self.content_scroll as usize);
        }
    }

    pub fn toggle_expand(&mut self) {
        if self.focus == Focus::Outline {
            if let Some(i) = self.outline_state.selected() {
                if i < self.outline_items.len() && self.outline_items[i].has_children {
                    let heading_text = self.outline_items[i].text.clone();

                    // Toggle the collapsed state
                    if self.collapsed_headings.contains(&heading_text) {
                        self.collapsed_headings.remove(&heading_text);
                    } else {
                        self.collapsed_headings.insert(heading_text.clone());
                    }

                    // Rebuild the flattened list
                    self.outline_items = Self::flatten_tree(&self.tree, &self.collapsed_headings);

                    // Restore selection by text (not by index)
                    if !self.select_by_text(&heading_text) {
                        // If heading not found (shouldn't happen), clamp to valid index
                        let safe_idx = i.min(self.outline_items.len().saturating_sub(1));
                        self.outline_state.select(Some(safe_idx));
                        self.outline_scroll_state =
                            ScrollbarState::new(self.outline_items.len()).position(safe_idx);
                    }
                }
            }
        }
    }

    pub fn expand(&mut self) {
        if self.focus == Focus::Outline {
            if let Some(i) = self.outline_state.selected() {
                if i < self.outline_items.len() && self.outline_items[i].has_children {
                    let heading_text = self.outline_items[i].text.clone();

                    // Remove from collapsed set to expand
                    self.collapsed_headings.remove(&heading_text);

                    // Rebuild the flattened list
                    self.outline_items = Self::flatten_tree(&self.tree, &self.collapsed_headings);

                    // Restore selection by text (not by index)
                    if !self.select_by_text(&heading_text) {
                        // If heading not found (shouldn't happen), clamp to valid index
                        let safe_idx = i.min(self.outline_items.len().saturating_sub(1));
                        self.outline_state.select(Some(safe_idx));
                        self.outline_scroll_state =
                            ScrollbarState::new(self.outline_items.len()).position(safe_idx);
                    }
                }
            }
        }
    }

    pub fn collapse(&mut self) {
        if self.focus == Focus::Outline {
            if let Some(i) = self.outline_state.selected() {
                if i < self.outline_items.len() {
                    let current_level = self.outline_items[i].level;
                    let current_text = self.outline_items[i].text.clone();

                    // If current heading has children, collapse it
                    if self.outline_items[i].has_children {
                        self.collapsed_headings.insert(current_text.clone());

                        // Rebuild the flattened list
                        self.outline_items =
                            Self::flatten_tree(&self.tree, &self.collapsed_headings);

                        // Restore selection by text
                        if !self.select_by_text(&current_text) {
                            let safe_idx = i.min(self.outline_items.len().saturating_sub(1));
                            self.outline_state.select(Some(safe_idx));
                            self.outline_scroll_state =
                                ScrollbarState::new(self.outline_items.len()).position(safe_idx);
                        }
                    } else {
                        // If no children, find parent and collapse it
                        // Look backwards for first heading with lower level
                        let mut parent_text: Option<String> = None;
                        for idx in (0..i).rev() {
                            if self.outline_items[idx].level < current_level {
                                // Found parent
                                parent_text = Some(self.outline_items[idx].text.clone());
                                break;
                            }
                        }

                        if let Some(parent) = parent_text {
                            // Collapse the parent
                            self.collapsed_headings.insert(parent.clone());

                            // Rebuild and move selection to parent
                            self.outline_items =
                                Self::flatten_tree(&self.tree, &self.collapsed_headings);

                            // Select the parent by text
                            if !self.select_by_text(&parent) {
                                // Fallback: select first item if parent not found
                                if !self.outline_items.is_empty() {
                                    self.outline_state.select(Some(0));
                                    self.outline_scroll_state =
                                        ScrollbarState::new(self.outline_items.len()).position(0);
                                }
                            }
                        }
                        // No parent found, do nothing
                    }
                }
            }
        }
    }

    pub fn toggle_focus(&mut self) {
        if self.show_outline {
            self.focus = match self.focus {
                Focus::Outline => Focus::Content,
                Focus::Content => Focus::Outline,
            };
        }
    }

    pub fn toggle_outline(&mut self) {
        self.show_outline = !self.show_outline;
        if !self.show_outline {
            // When hiding outline, switch focus to content
            self.focus = Focus::Content;
        } else {
            // When showing outline, switch focus back to outline
            self.focus = Focus::Outline;
        }
    }

    pub fn cycle_outline_width(&mut self, increase: bool) {
        if increase {
            self.outline_width = match self.outline_width {
                20 => 30,
                30 => 40,
                _ => 40,
            };
        } else {
            self.outline_width = match self.outline_width {
                40 => 30,
                30 => 20,
                _ => 20,
            };
        }
    }

    pub fn jump_to_heading(&mut self, index: usize) {
        if index < self.outline_items.len() {
            self.outline_state.select(Some(index));
            self.outline_scroll_state = self.outline_scroll_state.position(index);
        }
    }

    pub fn set_bookmark(&mut self) {
        // Store bookmark as heading text instead of index
        self.bookmark_position = self.selected_heading_text().map(|s| s.to_string());
    }

    pub fn jump_to_bookmark(&mut self) {
        // Jump to bookmark by finding the heading text
        if let Some(text) = self.bookmark_position.clone() {
            self.select_by_text(&text);
        }
    }

    pub fn selected_heading_text(&self) -> Option<&str> {
        self.outline_state
            .selected()
            .and_then(|i| self.outline_items.get(i))
            .map(|item| item.text.as_str())
    }

    pub fn toggle_theme_picker(&mut self) {
        self.show_theme_picker = !self.show_theme_picker;
        if self.show_theme_picker {
            // Set selected to current theme when opening
            self.theme_picker_selected = match self.current_theme {
                ThemeName::OceanDark => 0,
                ThemeName::Nord => 1,
                ThemeName::Dracula => 2,
                ThemeName::Solarized => 3,
                ThemeName::Monokai => 4,
                ThemeName::Gruvbox => 5,
                ThemeName::TokyoNight => 6,
                ThemeName::CatppuccinMocha => 7,
            };
        }
    }

    pub fn theme_picker_next(&mut self) {
        if self.theme_picker_selected < 7 {
            self.theme_picker_selected += 1;
        }
    }

    pub fn theme_picker_previous(&mut self) {
        if self.theme_picker_selected > 0 {
            self.theme_picker_selected -= 1;
        }
    }

    pub fn apply_selected_theme(&mut self) {
        let new_theme = match self.theme_picker_selected {
            0 => ThemeName::OceanDark,
            1 => ThemeName::Nord,
            2 => ThemeName::Dracula,
            3 => ThemeName::Solarized,
            4 => ThemeName::Monokai,
            5 => ThemeName::Gruvbox,
            6 => ThemeName::TokyoNight,
            7 => ThemeName::CatppuccinMocha,
            _ => ThemeName::OceanDark,
        };

        self.current_theme = new_theme;
        self.theme = Theme::from_name(new_theme);
        self.show_theme_picker = false;
    }

    pub fn copy_content(&mut self) {
        // Copy the currently selected section's content
        if let Some(heading_text) = self.selected_heading_text() {
            if let Some(section) = self.document.extract_section(heading_text) {
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    let _ = clipboard.set_text(section);
                }
            }
        }
    }

    pub fn copy_anchor(&mut self) {
        // Copy the anchor link for the currently selected heading
        if let Some(heading_text) = self.selected_heading_text() {
            // Convert heading to anchor format (lowercase, replace spaces with dashes)
            let anchor = heading_text
                .to_lowercase()
                .replace(|c: char| !c.is_alphanumeric() && c != ' ', "")
                .replace(' ', "-");

            let anchor_link = format!("#{}", anchor);

            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                let _ = clipboard.set_text(anchor_link);
            }
        }
    }
}
