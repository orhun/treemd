# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.3] - 2025-11-08

### Added

- **Shell tab completions** - Native dynamic completions for bash, zsh, and fish shells
- **Intelligent file filtering** - Tab completion intelligently filters to only show `.md` and `.markdown` files
- **Interactive setup helper** - `--setup-completions` flag to auto-detect shell and configure completion with one command
- **Auto shell detection** - Automatically detects bash, zsh, or fish and locates shell config files
- **Enhanced help system** - Comprehensive `--help` menu with detailed descriptions, examples, and usage patterns
- **Setup instructions** - Clear instructions for manual completion setup if automated setup is declined

### Changed

- **Feature enablement** - `unstable-dynamic` feature now enabled by default for seamless completion experience
- **Help documentation** - All CLI options now have detailed descriptions with inline examples

### Technical

- Integrated `clap_complete 4.5.60` with `unstable-dynamic` feature flag
- Implemented `CompleteEnv` for runtime completion generation
- Created custom `ArgValueCandidates` for markdown file filtering
- Added interactive setup module with shell detection (`src/cli/setup.rs`)

## [0.1.2] - 2025-11-08

### Fixed

- **Content display robustness** - Fixed critical bugs where content would disappear or show incorrect sections after navigation and collapse/expand operations
- **Selection preservation** - Selection now correctly preserved by heading text instead of index during collapse/expand operations
- **Content scroll reset** - Content scroll now properly resets to top when navigating between different sections
- **Dynamic content height** - Content height and scrollbar now correctly update based on the currently selected section
- **Collapse parent behavior** - Collapsing a parent heading now correctly selects the parent instead of an arbitrary item
- **Search filter preservation** - Search filtering now maintains the current selection when possible instead of always jumping to first item
- **Bookmark stability** - Bookmarks now store heading text instead of indices, remaining valid after collapse operations

### Technical

- Added `select_by_text()` helper method for robust text-based selection
- Added `update_content_metrics()` to synchronize content height and scroll state
- Added `previous_selection` tracking to detect selection changes
- Changed bookmark storage from `Option<usize>` to `Option<String>`

## [0.1.1] - 2025-11-08 - Add library

## [0.1.0] - 2025-11-08 - Initial Release 🚀

A modern markdown navigator with tree-based structural navigation and syntax highlighting.

### Features

#### 🎨 Interactive TUI

- **Dual-pane interface** - Navigate outline while viewing content
- **Syntax highlighting** - 50+ languages with full syntect integration
- **Vim-style navigation** - j/k, g/G, d/u for efficient browsing
- **Search & filter** - Press `/` to filter headings in real-time
- **Collapsible tree** - Expand/collapse sections with Space/Enter
- **Bookmarks** - Mark positions (`m`) and jump back (`'`)
- **Adjustable layout** - Toggle outline visibility, resize panes (20%, 30%, 40%)
- **Rich rendering** - Bold, italic, inline code, lists, blockquotes, code blocks
- **8 Beautiful Themes** - Ocean Dark (default), Nord, Dracula, Solarized, Monokai, Gruvbox, Tokyo Night, Catppuccin Mocha
- **Clipboard integration** - Copy section content (`y`) or anchor links (`Y`)
- **Help overlay** - Press `?` for keyboard shortcuts
- **Scrollbars** - Visual position indicators

#### ⚡ CLI Mode

- **List headings** - Quick overview of document structure (`-l`)
- **Tree visualization** - Hierarchical display with box-drawing (`--tree`)
- **Section extraction** - Extract specific sections by heading name (`-s`)
- **Smart filtering** - Filter by text or heading level (`--filter`, `-L`)
- **Multiple formats** - Plain text, JSON output (`-o json`)
- **Statistics** - Count headings by level (`--count`)

### Technical

- Built with Rust for performance and reliability
- Ratatui 0.29 for beautiful TUI rendering
- Syntect 5.2 for syntax highlighting
- Pulldown-cmark 0.13 for markdown parsing
- Arboard 3.4 for cross-platform clipboard support
- Optimized release binary with LTO and size optimization
- Comprehensive documentation on docs.rs
- MIT licensed
