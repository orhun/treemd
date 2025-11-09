use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThemeName {
    OceanDark,
    Nord,
    Dracula,
    Solarized,
    Monokai,
    Gruvbox,
    TokyoNight,
    CatppuccinMocha,
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: &'static str,
    pub background: Color,
    pub foreground: Color,
    pub heading_1: Color,
    pub heading_2: Color,
    pub heading_3: Color,
    pub heading_4: Color,
    pub heading_5: Color,
    pub border_focused: Color,
    pub border_unfocused: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
    pub status_bar_bg: Color,
    pub status_bar_fg: Color,
    pub inline_code_fg: Color,
    pub inline_code_bg: Color,
    pub bold_fg: Color,
    pub italic_fg: Color,
    pub list_bullet: Color,
    pub blockquote_border: Color,
    pub blockquote_fg: Color,
    pub code_fence: Color,
}

impl Theme {
    pub fn from_name(name: ThemeName) -> Self {
        match name {
            ThemeName::OceanDark => Self::ocean_dark(),
            ThemeName::Nord => Self::nord(),
            ThemeName::Dracula => Self::dracula(),
            ThemeName::Solarized => Self::solarized(),
            ThemeName::Monokai => Self::monokai(),
            ThemeName::Gruvbox => Self::gruvbox(),
            ThemeName::TokyoNight => Self::tokyo_night(),
            ThemeName::CatppuccinMocha => Self::catppuccin_mocha(),
        }
    }

    /// Base16 Ocean Dark - Default theme
    pub fn ocean_dark() -> Self {
        Self {
            name: "Ocean Dark",
            background: Color::Rgb(43, 48, 59),
            foreground: Color::Rgb(192, 197, 206),
            heading_1: Color::Rgb(100, 200, 255),
            heading_2: Color::Rgb(150, 200, 255),
            heading_3: Color::Rgb(150, 255, 200),
            heading_4: Color::Rgb(200, 255, 150),
            heading_5: Color::Rgb(200, 200, 200),
            border_focused: Color::Cyan,
            border_unfocused: Color::DarkGray,
            selection_bg: Color::Rgb(40, 40, 60),
            selection_fg: Color::White,
            status_bar_bg: Color::Rgb(52, 61, 70),
            status_bar_fg: Color::Rgb(192, 197, 206),
            inline_code_fg: Color::Rgb(255, 200, 100),
            inline_code_bg: Color::Rgb(50, 50, 50),
            bold_fg: Color::White,
            italic_fg: Color::Rgb(192, 143, 255),
            list_bullet: Color::Cyan,
            blockquote_border: Color::Rgb(150, 150, 150),
            blockquote_fg: Color::Rgb(150, 150, 150),
            code_fence: Color::Rgb(150, 180, 200),
        }
    }

    /// Nord theme - Arctic, north-bluish color palette
    pub fn nord() -> Self {
        Self {
            name: "Nord",
            background: Color::Rgb(46, 52, 64),
            foreground: Color::Rgb(216, 222, 233),
            heading_1: Color::Rgb(136, 192, 208), // Nord Frost
            heading_2: Color::Rgb(143, 188, 187), // Nord Frost
            heading_3: Color::Rgb(163, 190, 140), // Nord Aurora Green
            heading_4: Color::Rgb(235, 203, 139), // Nord Aurora Yellow
            heading_5: Color::Rgb(180, 142, 173), // Nord Aurora Purple
            border_focused: Color::Rgb(136, 192, 208),
            border_unfocused: Color::Rgb(76, 86, 106),
            selection_bg: Color::Rgb(59, 66, 82),
            selection_fg: Color::Rgb(236, 239, 244),
            status_bar_bg: Color::Rgb(59, 66, 82),
            status_bar_fg: Color::Rgb(216, 222, 233),
            inline_code_fg: Color::Rgb(235, 203, 139),
            inline_code_bg: Color::Rgb(59, 66, 82),
            bold_fg: Color::Rgb(236, 239, 244),
            italic_fg: Color::Rgb(180, 142, 173),
            list_bullet: Color::Rgb(136, 192, 208),
            blockquote_border: Color::Rgb(76, 86, 106),
            blockquote_fg: Color::Rgb(76, 86, 106),
            code_fence: Color::Rgb(143, 188, 187),
        }
    }

    /// Dracula theme - Dark theme with vibrant colors
    pub fn dracula() -> Self {
        Self {
            name: "Dracula",
            background: Color::Rgb(40, 42, 54),
            foreground: Color::Rgb(248, 248, 242),
            heading_1: Color::Rgb(139, 233, 253), // Cyan
            heading_2: Color::Rgb(80, 250, 123),  // Green
            heading_3: Color::Rgb(255, 184, 108), // Orange
            heading_4: Color::Rgb(255, 121, 198), // Pink
            heading_5: Color::Rgb(189, 147, 249), // Purple
            border_focused: Color::Rgb(189, 147, 249),
            border_unfocused: Color::Rgb(68, 71, 90),
            selection_bg: Color::Rgb(68, 71, 90),
            selection_fg: Color::Rgb(248, 248, 242),
            status_bar_bg: Color::Rgb(68, 71, 90),
            status_bar_fg: Color::Rgb(248, 248, 242),
            inline_code_fg: Color::Rgb(241, 250, 140),
            inline_code_bg: Color::Rgb(68, 71, 90),
            bold_fg: Color::Rgb(255, 255, 255),
            italic_fg: Color::Rgb(189, 147, 249),
            list_bullet: Color::Rgb(139, 233, 253),
            blockquote_border: Color::Rgb(98, 114, 164),
            blockquote_fg: Color::Rgb(98, 114, 164),
            code_fence: Color::Rgb(189, 147, 249),
        }
    }

    /// Solarized Dark - Precision colors for machines and people
    pub fn solarized() -> Self {
        Self {
            name: "Solarized",
            background: Color::Rgb(0, 43, 54),
            foreground: Color::Rgb(131, 148, 150),
            heading_1: Color::Rgb(38, 139, 210), // Blue
            heading_2: Color::Rgb(42, 161, 152), // Cyan
            heading_3: Color::Rgb(133, 153, 0),  // Green
            heading_4: Color::Rgb(181, 137, 0),  // Yellow
            heading_5: Color::Rgb(203, 75, 22),  // Orange
            border_focused: Color::Rgb(38, 139, 210),
            border_unfocused: Color::Rgb(7, 54, 66),
            selection_bg: Color::Rgb(7, 54, 66),
            selection_fg: Color::Rgb(147, 161, 161),
            status_bar_bg: Color::Rgb(7, 54, 66),
            status_bar_fg: Color::Rgb(131, 148, 150),
            inline_code_fg: Color::Rgb(181, 137, 0),
            inline_code_bg: Color::Rgb(7, 54, 66),
            bold_fg: Color::Rgb(147, 161, 161),
            italic_fg: Color::Rgb(108, 113, 196),
            list_bullet: Color::Rgb(42, 161, 152),
            blockquote_border: Color::Rgb(88, 110, 117),
            blockquote_fg: Color::Rgb(88, 110, 117),
            code_fence: Color::Rgb(42, 161, 152),
        }
    }

    /// Monokai - Sublime Text's iconic color scheme
    pub fn monokai() -> Self {
        Self {
            name: "Monokai",
            background: Color::Rgb(39, 40, 34),
            foreground: Color::Rgb(248, 248, 242),
            heading_1: Color::Rgb(102, 217, 239), // Cyan
            heading_2: Color::Rgb(166, 226, 46),  // Green
            heading_3: Color::Rgb(253, 151, 31),  // Orange
            heading_4: Color::Rgb(249, 38, 114),  // Pink
            heading_5: Color::Rgb(174, 129, 255), // Purple
            border_focused: Color::Rgb(102, 217, 239),
            border_unfocused: Color::Rgb(73, 72, 62),
            selection_bg: Color::Rgb(73, 72, 62),
            selection_fg: Color::Rgb(248, 248, 242),
            status_bar_bg: Color::Rgb(73, 72, 62),
            status_bar_fg: Color::Rgb(248, 248, 242),
            inline_code_fg: Color::Rgb(230, 219, 116),
            inline_code_bg: Color::Rgb(73, 72, 62),
            bold_fg: Color::Rgb(255, 255, 255),
            italic_fg: Color::Rgb(102, 217, 239),
            list_bullet: Color::Rgb(102, 217, 239),
            blockquote_border: Color::Rgb(117, 113, 94),
            blockquote_fg: Color::Rgb(117, 113, 94),
            code_fence: Color::Rgb(102, 217, 239),
        }
    }

    /// Gruvbox Dark - Retro groove color scheme
    pub fn gruvbox() -> Self {
        Self {
            name: "Gruvbox",
            background: Color::Rgb(40, 40, 40),
            foreground: Color::Rgb(235, 219, 178),
            heading_1: Color::Rgb(131, 165, 152), // Aqua
            heading_2: Color::Rgb(184, 187, 38),  // Green
            heading_3: Color::Rgb(250, 189, 47),  // Yellow
            heading_4: Color::Rgb(254, 128, 25),  // Orange
            heading_5: Color::Rgb(211, 134, 155), // Purple
            border_focused: Color::Rgb(184, 187, 38),
            border_unfocused: Color::Rgb(60, 56, 54),
            selection_bg: Color::Rgb(60, 56, 54),
            selection_fg: Color::Rgb(235, 219, 178),
            status_bar_bg: Color::Rgb(60, 56, 54),
            status_bar_fg: Color::Rgb(235, 219, 178),
            inline_code_fg: Color::Rgb(250, 189, 47),
            inline_code_bg: Color::Rgb(60, 56, 54),
            bold_fg: Color::Rgb(251, 241, 199),
            italic_fg: Color::Rgb(211, 134, 155),
            list_bullet: Color::Rgb(131, 165, 152),
            blockquote_border: Color::Rgb(146, 131, 116),
            blockquote_fg: Color::Rgb(146, 131, 116),
            code_fence: Color::Rgb(131, 165, 152),
        }
    }

    /// Tokyo Night - Modern dark theme celebrating Tokyo's neon lights at night
    pub fn tokyo_night() -> Self {
        Self {
            name: "Tokyo Night",
            background: Color::Rgb(26, 27, 38), // Very dark blue-black
            foreground: Color::Rgb(192, 202, 245), // Soft blue-white
            heading_1: Color::Rgb(122, 162, 247), // Blue
            heading_2: Color::Rgb(125, 207, 255), // Cyan
            heading_3: Color::Rgb(158, 206, 106), // Green
            heading_4: Color::Rgb(224, 175, 104), // Yellow
            heading_5: Color::Rgb(187, 154, 247), // Purple
            border_focused: Color::Rgb(122, 162, 247),
            border_unfocused: Color::Rgb(41, 46, 66),
            selection_bg: Color::Rgb(41, 46, 66),
            selection_fg: Color::Rgb(192, 202, 245),
            status_bar_bg: Color::Rgb(31, 35, 53),
            status_bar_fg: Color::Rgb(192, 202, 245),
            inline_code_fg: Color::Rgb(255, 158, 100), // Orange
            inline_code_bg: Color::Rgb(41, 46, 66),
            bold_fg: Color::Rgb(255, 255, 255),
            italic_fg: Color::Rgb(187, 154, 247),       // Purple
            list_bullet: Color::Rgb(125, 207, 255),     // Cyan
            blockquote_border: Color::Rgb(86, 95, 137), // Comment
            blockquote_fg: Color::Rgb(169, 177, 214),   // Fg dark
            code_fence: Color::Rgb(125, 207, 255),      // Cyan
        }
    }

    /// Catppuccin Mocha - Soothing pastel theme for cozy night coding
    pub fn catppuccin_mocha() -> Self {
        Self {
            name: "Catppuccin Mocha",
            background: Color::Rgb(30, 30, 46),    // Base
            foreground: Color::Rgb(205, 214, 244), // Text
            heading_1: Color::Rgb(137, 180, 250),  // Blue
            heading_2: Color::Rgb(137, 220, 235),  // Sky
            heading_3: Color::Rgb(166, 227, 161),  // Green
            heading_4: Color::Rgb(249, 226, 175),  // Yellow
            heading_5: Color::Rgb(203, 166, 247),  // Mauve
            border_focused: Color::Rgb(137, 180, 250),
            border_unfocused: Color::Rgb(69, 71, 90), // Surface 1
            selection_bg: Color::Rgb(69, 71, 90),     // Surface 1
            selection_fg: Color::Rgb(205, 214, 244),  // Text
            status_bar_bg: Color::Rgb(24, 24, 37),    // Mantle
            status_bar_fg: Color::Rgb(205, 214, 244), // Text
            inline_code_fg: Color::Rgb(250, 179, 135), // Peach
            inline_code_bg: Color::Rgb(49, 50, 68),   // Surface 0
            bold_fg: Color::Rgb(255, 255, 255),
            italic_fg: Color::Rgb(245, 194, 231),         // Pink
            list_bullet: Color::Rgb(148, 226, 213),       // Teal
            blockquote_border: Color::Rgb(108, 112, 134), // Overlay 0
            blockquote_fg: Color::Rgb(147, 153, 178),     // Overlay 2
            code_fence: Color::Rgb(116, 199, 236),        // Sapphire
        }
    }

    pub fn heading_color(&self, level: usize) -> Color {
        match level {
            1 => self.heading_1,
            2 => self.heading_2,
            3 => self.heading_3,
            4 => self.heading_4,
            _ => self.heading_5,
        }
    }

    pub fn border_style(&self, focused: bool) -> Style {
        if focused {
            Style::default().fg(self.border_focused)
        } else {
            Style::default().fg(self.border_unfocused)
        }
    }

    pub fn selection_style(&self) -> Style {
        Style::default()
            .bg(self.selection_bg)
            .fg(self.selection_fg)
            .add_modifier(Modifier::BOLD)
    }

    pub fn status_bar_style(&self) -> Style {
        Style::default()
            .bg(self.status_bar_bg)
            .fg(self.status_bar_fg)
    }

    pub fn inline_code_style(&self) -> Style {
        Style::default()
            .fg(self.inline_code_fg)
            .bg(self.inline_code_bg)
    }

    pub fn bold_style(&self) -> Style {
        Style::default()
            .fg(self.bold_fg)
            .add_modifier(Modifier::BOLD)
    }

    pub fn italic_style(&self) -> Style {
        Style::default()
            .fg(self.italic_fg)
            .add_modifier(Modifier::ITALIC)
    }

    pub fn text_style(&self) -> Style {
        Style::default().fg(self.foreground)
    }

    pub fn content_style(&self) -> Style {
        Style::default().fg(self.foreground).bg(self.background)
    }

    pub fn code_fence_style(&self) -> Style {
        Style::default().fg(self.code_fence)
    }
}
