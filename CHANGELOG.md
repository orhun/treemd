# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.7] - 2025-01-10

### Fixed

- **Tab completion for current directory files** - Fixed bug where `treemd R<tab>` wouldn't complete to `README.md` in the current directory. Path::parent() returns empty string for simple filenames, which is now normalized to "." for proper completion matching.

### Added

- **Filename in title bar** - Title bar now displays the filename being viewed: "treemd - README.md - 15 headings"
- **Current heading in content pane** - Content pane header now shows the selected heading name instead of the generic "Content" label, providing better context while reading

### Changed

- **App struct enhancement** - Added `filename` field to track the source file for display purposes
- **Content pane title logic** - Title dynamically updates based on selected heading, falling back to "Content" when none selected

### Technical

- Normalized empty parent paths in file completer to fix `Path::new("R").parent()` returning `Some("")` instead of `Some(".")`
- Extracted filename from PathBuf when launching TUI mode using `file_name()` and `to_str()`

## [0.1.6] - 2025-01-09

### Fixed

- **TUI section extraction with inline markdown** - Fixed critical bug where selecting headings with inline formatting (like `**bold**`) would display the entire document instead of just that section
- **JSON output content extraction** - Fixed nested JSON output where parent sections incorrectly included child heading text in their content
- **Parent directory completions** - Shell completions now work with `../` relative paths and absolute paths, enabling navigation to parent directories

### Added

- **Offset-based parsing** - Implemented pulldown-cmark's `into_offset_iter()` for direct byte offset tracking, eliminating fragile string searching
- **Shared utilities module** - Created `parser/utils.rs` with `strip_markdown_inline()` and `get_heading_level()` helpers
- **Comprehensive test suite** - Added 8+ new tests covering bold headings, numbered headings, offset tracking, and section extraction edge cases
- **Context-aware completions** - Upgraded from `ArgValueCandidates` to `ArgValueCompleter` for dynamic path-based completions

### Changed

- **Heading struct enhancement** - Added `offset: usize` field to store byte position for O(1) section extraction
- **Parser optimization** - Changed from O(n²) string searching to O(n) offset-based extraction
- **Code deduplication** - Eliminated 40+ lines of duplicate code by centralizing utilities
- **Completion logic** - Completions now parse input path to determine target directory and filter appropriately

### Technical

- **DRY principle compliance** - Removed duplicate `strip_markdown_inline` and `get_heading_level` functions
- **Best practices adoption** - Using pulldown-cmark's built-in offset tracking as recommended by Rust markdown ecosystem
- **Performance improvement** - Section extraction now O(1) lookup instead of O(n) string search
- **Robustness improvement** - Handles all inline markdown formatting (bold, italic, code, strikethrough) correctly
- **Architecture cleanup** - Better separation of concerns with dedicated utils module
- **Zero clippy warnings** - Clean codebase with all lints addressed

## [0.1.5] - 2025-11-09

### Fixed

- **Binary corruption in releases** - Wrap binaries in platform-appropriate archives to preserve file permissions and prevent corruption through GitHub Actions artifact system
- **Executable permissions lost** - Fixed issue where Unix binaries lost +x permission during artifact upload/download
- **SHA256 mismatch** - Generate checksums at build time and verify after extraction to ensure binary integrity
- **Cache conflicts** - Use unique cache keys per target to prevent 409 conflicts in parallel builds
- **Cross-platform compatibility** - Use native tools on each platform (PowerShell on Windows, bash on Unix)

### Added

- **Individual SHA256 files** - Each binary now has a corresponding .sha256 file generated at build time
- **Combined SHA256SUMS** - All binaries included in a single SHA256SUMS file in releases for easy verification
- **Checksum verification** - Automated verification step that checksums match after artifact extraction
- **Build-time checksums** - SHA256 printed during build for traceability
- **Platform-specific packaging** - Unix binaries distributed as .tar.gz, Windows binaries as .zip

### Changed

- **Updated GitHub Actions** - Upgraded all actions to latest major versions for automatic updates and security
- **Optimized cross installation** - Use cargo-binstall for 100x faster installation (2s vs 3m39s)
  - Install from main branch (cross-rs hasn't released since v0.2.5 in Feb 2023)
  - Pinned to commit 8633ec6 (Nov 2025) for reproducibility
  - Follows 2025 best practices for actively-developed-but-not-released projects

### Technical

- Unix: tar.gz archives preserve executable permissions through artifact system
- Windows: zip archives created with PowerShell's Compress-Archive
- Unix: SHA256 generated with sha256sum, Windows: Get-FileHash cmdlet
- Cache keys now include both target and Cargo.lock hash
- Comprehensive logging at each stage (build, extract, verify)
- Release extraction handles both tar.gz and zip formats

## [0.1.4] - 2025-11-09

### Changed

- Version bump for development

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
