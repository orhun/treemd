use crate::parser::{Document, HeadingNode, Link, extract_links};
use crate::tui::syntax::SyntaxHighlighter;
use crate::tui::theme::{Theme, ThemeName};
use ratatui::widgets::{ListState, ScrollbarState};
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Focus {
    Outline,
    Content,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppMode {
    Normal,
    LinkFollow,
    Search,
    ThemePicker,
    Help,
}

pub struct App {
    pub document: Document,
    pub filename: String,
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

    // Link following state
    pub mode: AppMode,
    pub current_file_path: PathBuf, // Path to current file for resolving relative links
    pub links_in_view: Vec<Link>,   // Links in currently displayed content
    pub selected_link_idx: Option<usize>, // Currently selected link index
    pub file_history: Vec<FileState>, // Back navigation stack
    pub file_future: Vec<FileState>, // Forward navigation stack (for undo back)
    pub status_message: Option<String>, // Temporary status message to display

    // Persistent clipboard for Linux X11 compatibility
    // On Linux, the clipboard instance must stay alive to serve paste requests
    clipboard: Option<arboard::Clipboard>,
}

/// Saved state for file navigation history
#[derive(Debug, Clone)]
pub struct FileState {
    pub path: PathBuf,
    pub document: Document,
    pub filename: String,
    pub selected_heading: Option<String>,
    pub content_scroll: u16,
    pub outline_state_selected: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct OutlineItem {
    pub level: usize,
    pub text: String,
    pub expanded: bool,
    pub has_children: bool, // Track if this heading has children in the tree
}

impl App {
    pub fn new(document: Document, filename: String, file_path: PathBuf) -> Self {
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
            filename,
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

            // Link following state
            mode: AppMode::Normal,
            current_file_path: file_path,
            links_in_view: Vec::new(),
            selected_link_idx: None,
            file_history: Vec::new(),
            file_future: Vec::new(),
            status_message: None,

            // Initialize persistent clipboard (None if unavailable)
            clipboard: arboard::Clipboard::new().ok(),
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

    pub fn jump_to_parent(&mut self) {
        if self.focus == Focus::Outline {
            if let Some(current_idx) = self.outline_state.selected() {
                if current_idx < self.outline_items.len() {
                    let current_level = self.outline_items[current_idx].level;

                    // Search backwards for a heading with lower level (parent)
                    for i in (0..current_idx).rev() {
                        if self.outline_items[i].level < current_level {
                            self.outline_state.select(Some(i));
                            self.outline_scroll_state = self.outline_scroll_state.position(i);
                            return;
                        }
                    }

                    // If no parent found, stay at current position
                    // (we're already at a top-level heading or first item)
                }
            }
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
                // Use persistent clipboard for Linux X11 compatibility
                if let Some(clipboard) = &mut self.clipboard {
                    match clipboard.set_text(section) {
                        Ok(_) => {
                            self.status_message = Some("✓ Section copied to clipboard".to_string());
                        }
                        Err(e) => {
                            self.status_message = Some(format!("✗ Clipboard error: {}", e));
                        }
                    }
                } else {
                    self.status_message = Some("✗ Clipboard not available".to_string());
                }
            } else {
                self.status_message = Some("✗ Could not extract section".to_string());
            }
        } else {
            self.status_message = Some("✗ No heading selected".to_string());
        }
    }

    pub fn copy_anchor(&mut self) {
        // Copy the anchor link for the currently selected heading
        if let Some(heading_text) = self.selected_heading_text() {
            // Convert heading to anchor format (lowercase, replace spaces with dashes)
            let anchor = Self::heading_to_anchor(heading_text);
            let anchor_link = format!("#{}", anchor);

            // Use persistent clipboard for Linux X11 compatibility
            if let Some(clipboard) = &mut self.clipboard {
                match clipboard.set_text(anchor_link) {
                    Ok(_) => {
                        self.status_message =
                            Some(format!("✓ Anchor link copied: #{}", anchor));
                    }
                    Err(e) => {
                        self.status_message = Some(format!("✗ Clipboard error: {}", e));
                    }
                }
            } else {
                self.status_message = Some("✗ Clipboard not available".to_string());
            }
        } else {
            self.status_message = Some("✗ No heading selected".to_string());
        }
    }

    /// Convert heading text to anchor format (lowercase, replace spaces with dashes)
    fn heading_to_anchor(heading: &str) -> String {
        heading
            .to_lowercase()
            .replace(|c: char| !c.is_alphanumeric() && c != ' ', "")
            .replace(' ', "-")
    }

    /// Enter link follow mode - extract links from current section and highlight them
    pub fn enter_link_follow_mode(&mut self) {
        // Extract content for current section
        let content = if let Some(heading_text) = self.selected_heading_text() {
            self.document
                .extract_section(heading_text)
                .unwrap_or_else(|| self.document.content.clone())
        } else {
            self.document.content.clone()
        };

        // Extract all links from the content
        self.links_in_view = extract_links(&content);

        // Always enter mode, even if no links (so user sees "no links" message)
        self.mode = AppMode::LinkFollow;

        // Select first link if any exist
        if !self.links_in_view.is_empty() {
            self.selected_link_idx = Some(0);
        } else {
            self.selected_link_idx = None;
        }
    }

    /// Exit link follow mode and return to normal mode
    pub fn exit_link_follow_mode(&mut self) {
        self.mode = AppMode::Normal;
        self.links_in_view.clear();
        self.selected_link_idx = None;
        // Don't clear status message here - let it display for a moment
    }

    /// Cycle to the next link (Tab in link follow mode)
    pub fn next_link(&mut self) {
        if self.mode == AppMode::LinkFollow && !self.links_in_view.is_empty() {
            self.selected_link_idx = Some(match self.selected_link_idx {
                Some(idx) => {
                    if idx >= self.links_in_view.len() - 1 {
                        0 // Wrap to first
                    } else {
                        idx + 1
                    }
                }
                None => 0,
            });
        }
    }

    /// Cycle to the previous link (Shift+Tab in link follow mode)
    pub fn previous_link(&mut self) {
        if self.mode == AppMode::LinkFollow && !self.links_in_view.is_empty() {
            self.selected_link_idx = Some(match self.selected_link_idx {
                Some(idx) => {
                    if idx == 0 {
                        self.links_in_view.len() - 1 // Wrap to last
                    } else {
                        idx - 1
                    }
                }
                None => 0,
            });
        }
    }

    /// Jump to parent heading while staying in link follow mode
    pub fn jump_to_parent_links(&mut self) {
        if self.mode == AppMode::LinkFollow {
            // First, jump to parent in outline
            if let Some(current_idx) = self.outline_state.selected() {
                if current_idx < self.outline_items.len() {
                    let current_level = self.outline_items[current_idx].level;

                    // Search backwards for a heading with lower level (parent)
                    for i in (0..current_idx).rev() {
                        if self.outline_items[i].level < current_level {
                            // Jump to parent in outline
                            self.outline_state.select(Some(i));
                            self.outline_scroll_state = self.outline_scroll_state.position(i);

                            // Now extract links from parent's content
                            let content = if let Some(heading_text) = self.selected_heading_text() {
                                self.document
                                    .extract_section(heading_text)
                                    .unwrap_or_else(|| self.document.content.clone())
                            } else {
                                self.document.content.clone()
                            };
                            self.links_in_view = extract_links(&content);

                            // Reset link selection
                            if !self.links_in_view.is_empty() {
                                self.selected_link_idx = Some(0);
                                self.status_message = Some(format!(
                                    "✓ Jumped to parent ({} links found)",
                                    self.links_in_view.len()
                                ));
                            } else {
                                self.selected_link_idx = None;
                                self.status_message = Some("⚠ Parent has no links".to_string());
                            }

                            return;
                        }
                    }

                    // If no parent found (already at top-level)
                    self.status_message = Some("⚠ Already at top-level heading".to_string());
                }
            }
        }
    }

    /// Get the currently selected link
    pub fn get_selected_link(&self) -> Option<&Link> {
        self.selected_link_idx
            .and_then(|idx| self.links_in_view.get(idx))
    }

    /// Follow the currently selected link
    pub fn follow_selected_link(&mut self) -> Result<(), String> {
        let link = match self.get_selected_link() {
            Some(link) => link.clone(),
            None => return Err("No link selected".to_string()),
        };

        match link.target {
            crate::parser::LinkTarget::Anchor(anchor) => {
                // Jump to heading in current document
                self.jump_to_anchor(&anchor)?;
                self.status_message = Some(format!("✓ Jumped to #{}", anchor));
                self.exit_link_follow_mode();
                Ok(())
            }
            crate::parser::LinkTarget::RelativeFile { path, anchor } => {
                // Load the linked file
                self.load_file(&path, anchor.as_deref())?;
                let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("file");
                self.status_message = Some(format!("✓ Opened {}", filename));
                self.exit_link_follow_mode();
                Ok(())
            }
            crate::parser::LinkTarget::WikiLink { target, .. } => {
                // Try to find and load the wikilinked file
                self.load_wikilink(&target)?;
                self.status_message = Some(format!("✓ Opened [[{}]]", target));
                self.exit_link_follow_mode();
                Ok(())
            }
            crate::parser::LinkTarget::External(url) => {
                // Try to open in default browser
                let open_result = open::that(&url);

                // Also copy to clipboard as backup
                let mut clipboard_success = false;
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    clipboard_success = clipboard.set_text(url.clone()).is_ok();
                }

                // Set status message
                self.status_message = match (open_result, clipboard_success) {
                    (Ok(_), true) => Some(format!(
                        "✓ Opened {} in browser (also copied to clipboard)",
                        url
                    )),
                    (Ok(_), false) => Some(format!("✓ Opened {} in browser", url)),
                    (Err(_), true) => Some(format!(
                        "⚠ Could not open browser, URL copied to clipboard: {}",
                        url
                    )),
                    (Err(_), false) => Some(format!("✗ Failed to open URL: {}", url)),
                };

                self.exit_link_follow_mode();
                Ok(())
            }
        }
    }

    /// Jump to a heading by anchor name
    fn jump_to_anchor(&mut self, anchor: &str) -> Result<(), String> {
        // Find heading that matches the anchor
        for (idx, item) in self.outline_items.iter().enumerate() {
            let item_anchor = Self::heading_to_anchor(&item.text);
            if item_anchor == anchor {
                self.outline_state.select(Some(idx));
                self.outline_scroll_state = self.outline_scroll_state.position(idx);
                return Ok(());
            }
        }
        Err(format!("Heading '{}' not found", anchor))
    }

    /// Load a file by relative path
    fn load_file(&mut self, relative_path: &PathBuf, anchor: Option<&str>) -> Result<(), String> {
        // Resolve path relative to current file
        let current_dir = self
            .current_file_path
            .parent()
            .ok_or("Cannot determine current directory")?;
        let absolute_path = current_dir.join(relative_path);

        // Parse the new file
        let new_document = crate::parser::parse_file(&absolute_path)
            .map_err(|e| format!("Failed to load file: {}", e))?;

        let new_filename = absolute_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Save current state to history
        self.save_to_history();

        // Load new document
        self.load_document(new_document, new_filename, absolute_path);

        // Jump to anchor if specified
        if let Some(anchor_name) = anchor {
            let _ = self.jump_to_anchor(anchor_name);
        }

        Ok(())
    }

    /// Find and load a wikilinked file
    fn load_wikilink(&mut self, target: &str) -> Result<(), String> {
        // Try to find the file relative to current directory
        let current_dir = self
            .current_file_path
            .parent()
            .ok_or("Cannot determine current directory")?;

        // Try various extensions
        let candidates = vec![
            format!("{}.md", target),
            format!("{}.markdown", target),
            target.to_string(),
        ];

        for candidate in candidates {
            let path = current_dir.join(&candidate);
            if path.exists() {
                return self.load_file(&PathBuf::from(candidate), None);
            }
        }

        Err(format!("Wikilink target '{}' not found", target))
    }

    /// Save current state to history before navigating away
    fn save_to_history(&mut self) {
        let state = FileState {
            path: self.current_file_path.clone(),
            document: self.document.clone(),
            filename: self.filename.clone(),
            selected_heading: self.selected_heading_text().map(|s| s.to_string()),
            content_scroll: self.content_scroll,
            outline_state_selected: self.outline_state.selected(),
        };
        self.file_history.push(state);

        // Clear forward history when navigating to a new file
        self.file_future.clear();
    }

    /// Load a new document and update all related state
    fn load_document(&mut self, document: Document, filename: String, path: PathBuf) {
        self.document = document;
        self.filename = filename;
        self.current_file_path = path;

        // Rebuild tree and outline
        self.tree = self.document.build_tree();
        self.outline_items = Self::flatten_tree(&self.tree, &self.collapsed_headings);

        // Reset selection to first item
        let mut outline_state = ListState::default();
        if !self.outline_items.is_empty() {
            outline_state.select(Some(0));
        }
        self.outline_state = outline_state;
        self.outline_scroll_state = ScrollbarState::new(self.outline_items.len());

        // Reset content scroll
        self.content_scroll = 0;
        let content_lines = self.document.content.lines().count();
        self.content_height = content_lines as u16;
        self.content_scroll_state = ScrollbarState::new(content_lines);

        // Clear previous selection tracking
        self.previous_selection = None;
    }

    /// Navigate back in file history
    pub fn go_back(&mut self) -> Result<(), String> {
        let previous_state = self
            .file_history
            .pop()
            .ok_or("No previous file in history")?;

        // Save current state to future stack
        let current_state = FileState {
            path: self.current_file_path.clone(),
            document: self.document.clone(),
            filename: self.filename.clone(),
            selected_heading: self.selected_heading_text().map(|s| s.to_string()),
            content_scroll: self.content_scroll,
            outline_state_selected: self.outline_state.selected(),
        };
        self.file_future.push(current_state);

        // Restore previous state
        self.restore_file_state(previous_state);

        Ok(())
    }

    /// Navigate forward in file history
    pub fn go_forward(&mut self) -> Result<(), String> {
        let next_state = self.file_future.pop().ok_or("No next file in history")?;

        // Save current state to history stack
        let current_state = FileState {
            path: self.current_file_path.clone(),
            document: self.document.clone(),
            filename: self.filename.clone(),
            selected_heading: self.selected_heading_text().map(|s| s.to_string()),
            content_scroll: self.content_scroll,
            outline_state_selected: self.outline_state.selected(),
        };
        self.file_history.push(current_state);

        // Restore next state
        self.restore_file_state(next_state);

        Ok(())
    }

    /// Restore a file state from history
    fn restore_file_state(&mut self, state: FileState) {
        self.load_document(state.document, state.filename, state.path);

        // Restore selection and scroll position
        if let Some(selected_idx) = state.outline_state_selected {
            if selected_idx < self.outline_items.len() {
                self.outline_state.select(Some(selected_idx));
                self.outline_scroll_state = self.outline_scroll_state.position(selected_idx);
            }
        }

        self.content_scroll = state.content_scroll;
        self.content_scroll_state = self
            .content_scroll_state
            .position(state.content_scroll as usize);
    }

    /// Reload current file from disk (used after external editing)
    pub fn reload_current_file(&mut self) -> Result<(), String> {
        // Save current state to restore after reload
        let current_selection = self.selected_heading_text().map(|s| s.to_string());
        let current_scroll = self.content_scroll;

        // Reload the file
        let content = std::fs::read_to_string(&self.current_file_path)
            .map_err(|e| format!("Failed to reload file: {}", e))?;

        let document = crate::parser::parse_markdown(&content);
        let filename = self
            .current_file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file")
            .to_string();

        self.load_document(document, filename, self.current_file_path.clone());

        // Try to restore selection if the heading still exists
        if let Some(heading) = current_selection {
            self.select_by_text(&heading);
        }

        // Restore scroll position (may be adjusted if content changed)
        if current_scroll < self.content_height {
            self.content_scroll = current_scroll;
            self.content_scroll_state = self
                .content_scroll_state
                .position(current_scroll as usize);
        }

        Ok(())
    }
}
