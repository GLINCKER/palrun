//! Theme support for the TUI.
//!
//! Provides customizable color themes including built-in themes
//! (Dracula, Nord, Solarized, Catppuccin) and custom theme support.

use ratatui::style::Color;

/// A complete color theme for the TUI.
///
/// Themes are runtime-only - configuration happens through the config file
/// with hex color strings which are parsed into Theme at startup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Theme {
    /// Theme name for display and configuration
    pub name: String,
    /// Primary accent color (headers, selected items, active elements)
    pub primary: Color,
    /// Secondary accent color (command prompts, success indicators)
    pub secondary: Color,
    /// Tertiary accent color (highlights, warnings)
    pub accent: Color,
    /// Highlight color for search matches
    pub highlight: Color,
    /// Main text color
    pub text: Color,
    /// Dimmed text color (descriptions, secondary info)
    pub text_dim: Color,
    /// Muted text color (placeholders, hints)
    pub text_muted: Color,
    /// Background color (Reset uses terminal default)
    pub background: Color,
    /// Selected item background
    pub selected_bg: Color,
    /// Border color
    pub border: Color,
    /// Success indicator color
    pub success: Color,
    /// Warning indicator color
    pub warning: Color,
    /// Error indicator color
    pub error: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::default_theme()
    }
}

impl Theme {
    /// Default theme - works well on both light and dark terminals.
    pub fn default_theme() -> Self {
        Self {
            name: "default".to_string(),
            primary: Color::Rgb(99, 102, 241),    // Indigo
            secondary: Color::Rgb(16, 185, 129),  // Emerald
            accent: Color::Rgb(251, 146, 60),     // Orange
            highlight: Color::Rgb(250, 204, 21),  // Yellow
            text: Color::White,
            text_dim: Color::Rgb(156, 163, 175),  // Gray-400
            text_muted: Color::Rgb(107, 114, 128), // Gray-500
            background: Color::Reset,
            selected_bg: Color::Rgb(55, 65, 81),  // Gray-700
            border: Color::Rgb(75, 85, 99),       // Gray-600
            success: Color::Rgb(34, 197, 94),     // Green
            warning: Color::Rgb(234, 179, 8),     // Yellow
            error: Color::Rgb(239, 68, 68),       // Red
        }
    }

    /// Dracula theme - dark purple and pink.
    pub fn dracula() -> Self {
        Self {
            name: "dracula".to_string(),
            primary: Color::Rgb(189, 147, 249),   // Purple
            secondary: Color::Rgb(80, 250, 123),  // Green
            accent: Color::Rgb(255, 121, 198),    // Pink
            highlight: Color::Rgb(241, 250, 140), // Yellow
            text: Color::Rgb(248, 248, 242),      // Foreground
            text_dim: Color::Rgb(189, 147, 249),  // Purple (dimmed)
            text_muted: Color::Rgb(98, 114, 164), // Comment
            background: Color::Rgb(40, 42, 54),   // Background
            selected_bg: Color::Rgb(68, 71, 90),  // Current Line
            border: Color::Rgb(68, 71, 90),       // Selection
            success: Color::Rgb(80, 250, 123),    // Green
            warning: Color::Rgb(255, 184, 108),   // Orange
            error: Color::Rgb(255, 85, 85),       // Red
        }
    }

    /// Nord theme - arctic, bluish colors.
    pub fn nord() -> Self {
        Self {
            name: "nord".to_string(),
            primary: Color::Rgb(136, 192, 208),   // Nord8 (Frost)
            secondary: Color::Rgb(163, 190, 140), // Nord14 (Aurora Green)
            accent: Color::Rgb(208, 135, 112),    // Nord12 (Aurora Orange)
            highlight: Color::Rgb(235, 203, 139), // Nord13 (Aurora Yellow)
            text: Color::Rgb(236, 239, 244),      // Nord6 (Snow Storm)
            text_dim: Color::Rgb(216, 222, 233),  // Nord5
            text_muted: Color::Rgb(76, 86, 106),  // Nord3 (Polar Night)
            background: Color::Rgb(46, 52, 64),   // Nord0
            selected_bg: Color::Rgb(59, 66, 82),  // Nord1
            border: Color::Rgb(67, 76, 94),       // Nord2
            success: Color::Rgb(163, 190, 140),   // Nord14
            warning: Color::Rgb(235, 203, 139),   // Nord13
            error: Color::Rgb(191, 97, 106),      // Nord11
        }
    }

    /// Solarized Dark theme.
    pub fn solarized_dark() -> Self {
        Self {
            name: "solarized-dark".to_string(),
            primary: Color::Rgb(38, 139, 210),    // Blue
            secondary: Color::Rgb(133, 153, 0),   // Green
            accent: Color::Rgb(203, 75, 22),      // Orange
            highlight: Color::Rgb(181, 137, 0),   // Yellow
            text: Color::Rgb(147, 161, 161),      // Base1
            text_dim: Color::Rgb(101, 123, 131),  // Base00
            text_muted: Color::Rgb(88, 110, 117), // Base01
            background: Color::Rgb(0, 43, 54),    // Base03
            selected_bg: Color::Rgb(7, 54, 66),   // Base02
            border: Color::Rgb(88, 110, 117),     // Base01
            success: Color::Rgb(133, 153, 0),     // Green
            warning: Color::Rgb(181, 137, 0),     // Yellow
            error: Color::Rgb(220, 50, 47),       // Red
        }
    }

    /// Solarized Light theme.
    pub fn solarized_light() -> Self {
        Self {
            name: "solarized-light".to_string(),
            primary: Color::Rgb(38, 139, 210),    // Blue
            secondary: Color::Rgb(133, 153, 0),   // Green
            accent: Color::Rgb(203, 75, 22),      // Orange
            highlight: Color::Rgb(181, 137, 0),   // Yellow
            text: Color::Rgb(88, 110, 117),       // Base01
            text_dim: Color::Rgb(101, 123, 131),  // Base00
            text_muted: Color::Rgb(147, 161, 161), // Base1
            background: Color::Rgb(253, 246, 227), // Base3
            selected_bg: Color::Rgb(238, 232, 213), // Base2
            border: Color::Rgb(147, 161, 161),    // Base1
            success: Color::Rgb(133, 153, 0),     // Green
            warning: Color::Rgb(181, 137, 0),     // Yellow
            error: Color::Rgb(220, 50, 47),       // Red
        }
    }

    /// Catppuccin Mocha theme - warm, pastel colors.
    pub fn catppuccin_mocha() -> Self {
        Self {
            name: "catppuccin-mocha".to_string(),
            primary: Color::Rgb(137, 180, 250),   // Blue
            secondary: Color::Rgb(166, 227, 161), // Green
            accent: Color::Rgb(250, 179, 135),    // Peach
            highlight: Color::Rgb(249, 226, 175), // Yellow
            text: Color::Rgb(205, 214, 244),      // Text
            text_dim: Color::Rgb(166, 173, 200),  // Subtext1
            text_muted: Color::Rgb(127, 132, 156), // Overlay1
            background: Color::Rgb(30, 30, 46),   // Base
            selected_bg: Color::Rgb(49, 50, 68),  // Surface0
            border: Color::Rgb(69, 71, 90),       // Surface1
            success: Color::Rgb(166, 227, 161),   // Green
            warning: Color::Rgb(249, 226, 175),   // Yellow
            error: Color::Rgb(243, 139, 168),     // Red
        }
    }

    /// Catppuccin Latte theme - light pastel colors.
    pub fn catppuccin_latte() -> Self {
        Self {
            name: "catppuccin-latte".to_string(),
            primary: Color::Rgb(30, 102, 245),    // Blue
            secondary: Color::Rgb(64, 160, 43),   // Green
            accent: Color::Rgb(254, 100, 11),     // Peach
            highlight: Color::Rgb(223, 142, 29),  // Yellow
            text: Color::Rgb(76, 79, 105),        // Text
            text_dim: Color::Rgb(92, 95, 119),    // Subtext1
            text_muted: Color::Rgb(140, 143, 161), // Overlay1
            background: Color::Rgb(239, 241, 245), // Base
            selected_bg: Color::Rgb(220, 224, 232), // Surface0
            border: Color::Rgb(188, 192, 204),    // Surface1
            success: Color::Rgb(64, 160, 43),     // Green
            warning: Color::Rgb(223, 142, 29),    // Yellow
            error: Color::Rgb(210, 15, 57),       // Red
        }
    }

    /// Tokyo Night theme - dark blue and purple.
    pub fn tokyo_night() -> Self {
        Self {
            name: "tokyo-night".to_string(),
            primary: Color::Rgb(122, 162, 247),   // Blue
            secondary: Color::Rgb(158, 206, 106), // Green
            accent: Color::Rgb(255, 158, 100),    // Orange
            highlight: Color::Rgb(224, 175, 104), // Yellow
            text: Color::Rgb(169, 177, 214),      // Foreground
            text_dim: Color::Rgb(86, 95, 137),    // Comment
            text_muted: Color::Rgb(65, 72, 104),  // Dark5
            background: Color::Rgb(26, 27, 38),   // Background
            selected_bg: Color::Rgb(41, 46, 66),  // Selection
            border: Color::Rgb(41, 46, 66),       // Selection
            success: Color::Rgb(158, 206, 106),   // Green
            warning: Color::Rgb(224, 175, 104),   // Yellow
            error: Color::Rgb(247, 118, 142),     // Red
        }
    }

    /// Gruvbox Dark theme - retro, earthy colors.
    pub fn gruvbox_dark() -> Self {
        Self {
            name: "gruvbox-dark".to_string(),
            primary: Color::Rgb(131, 165, 152),   // Aqua
            secondary: Color::Rgb(184, 187, 38),  // Green
            accent: Color::Rgb(254, 128, 25),     // Orange
            highlight: Color::Rgb(250, 189, 47),  // Yellow
            text: Color::Rgb(235, 219, 178),      // Foreground
            text_dim: Color::Rgb(168, 153, 132),  // Gray
            text_muted: Color::Rgb(146, 131, 116), // Dark Gray
            background: Color::Rgb(40, 40, 40),   // Background
            selected_bg: Color::Rgb(60, 56, 54),  // BG1
            border: Color::Rgb(80, 73, 69),       // BG2
            success: Color::Rgb(184, 187, 38),    // Green
            warning: Color::Rgb(250, 189, 47),    // Yellow
            error: Color::Rgb(251, 73, 52),       // Red
        }
    }

    /// One Dark theme - Atom's default dark theme.
    pub fn one_dark() -> Self {
        Self {
            name: "one-dark".to_string(),
            primary: Color::Rgb(97, 175, 239),    // Blue
            secondary: Color::Rgb(152, 195, 121), // Green
            accent: Color::Rgb(209, 154, 102),    // Orange
            highlight: Color::Rgb(229, 192, 123), // Yellow
            text: Color::Rgb(171, 178, 191),      // Foreground
            text_dim: Color::Rgb(92, 99, 112),    // Comment
            text_muted: Color::Rgb(76, 82, 99),   // Gutter
            background: Color::Rgb(40, 44, 52),   // Background
            selected_bg: Color::Rgb(62, 68, 81),  // Selection
            border: Color::Rgb(62, 68, 81),       // Selection
            success: Color::Rgb(152, 195, 121),   // Green
            warning: Color::Rgb(229, 192, 123),   // Yellow
            error: Color::Rgb(224, 108, 117),     // Red
        }
    }

    /// High Contrast theme - Maximum readability for accessibility.
    pub fn high_contrast() -> Self {
        Self {
            name: "high-contrast".to_string(),
            primary: Color::Cyan,                  // Bright cyan
            secondary: Color::Green,               // Bright green
            accent: Color::Yellow,                 // Bright yellow
            highlight: Color::Yellow,              // Bright yellow
            text: Color::White,                    // Pure white
            text_dim: Color::LightCyan,            // Light cyan
            text_muted: Color::Gray,               // Gray
            background: Color::Black,              // Pure black
            selected_bg: Color::Blue,              // High contrast selection
            border: Color::White,                  // White borders
            success: Color::LightGreen,            // Bright green
            warning: Color::LightYellow,           // Bright yellow
            error: Color::LightRed,                // Bright red
        }
    }

    /// Get a theme by name (case-insensitive).
    pub fn by_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "default" => Some(Self::default_theme()),
            "dracula" => Some(Self::dracula()),
            "nord" => Some(Self::nord()),
            "solarized-dark" | "solarized_dark" => Some(Self::solarized_dark()),
            "solarized-light" | "solarized_light" => Some(Self::solarized_light()),
            "catppuccin-mocha" | "catppuccin_mocha" | "catppuccin" => Some(Self::catppuccin_mocha()),
            "catppuccin-latte" | "catppuccin_latte" => Some(Self::catppuccin_latte()),
            "tokyo-night" | "tokyo_night" => Some(Self::tokyo_night()),
            "gruvbox-dark" | "gruvbox_dark" | "gruvbox" => Some(Self::gruvbox_dark()),
            "one-dark" | "one_dark" => Some(Self::one_dark()),
            "high-contrast" | "high_contrast" => Some(Self::high_contrast()),
            _ => None,
        }
    }

    /// List all available built-in theme names.
    pub fn available_themes() -> Vec<&'static str> {
        vec![
            "default",
            "dracula",
            "nord",
            "solarized-dark",
            "solarized-light",
            "catppuccin-mocha",
            "catppuccin-latte",
            "tokyo-night",
            "gruvbox-dark",
            "one-dark",
            "high-contrast",
        ]
    }
}

/// Parse a hex color string (#RRGGBB or RRGGBB) into a Color.
pub fn parse_hex_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

    Some(Color::Rgb(r, g, b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_theme() {
        let theme = Theme::default();
        assert_eq!(theme.name, "default");
    }

    #[test]
    fn test_theme_by_name() {
        assert!(Theme::by_name("dracula").is_some());
        assert!(Theme::by_name("DRACULA").is_some());
        assert!(Theme::by_name("Nord").is_some());
        assert!(Theme::by_name("solarized-dark").is_some());
        assert!(Theme::by_name("solarized_dark").is_some());
        assert!(Theme::by_name("unknown-theme").is_none());
    }

    #[test]
    fn test_available_themes() {
        let themes = Theme::available_themes();
        assert!(themes.contains(&"default"));
        assert!(themes.contains(&"dracula"));
        assert!(themes.contains(&"nord"));
        assert!(themes.len() >= 8);
    }

    #[test]
    fn test_parse_hex_color() {
        assert_eq!(parse_hex_color("#FF0000"), Some(Color::Rgb(255, 0, 0)));
        assert_eq!(parse_hex_color("00FF00"), Some(Color::Rgb(0, 255, 0)));
        assert_eq!(parse_hex_color("#0000FF"), Some(Color::Rgb(0, 0, 255)));
        assert_eq!(parse_hex_color("#282a36"), Some(Color::Rgb(40, 42, 54)));
        assert_eq!(parse_hex_color("invalid"), None);
        assert_eq!(parse_hex_color("#FFF"), None);
    }

    #[test]
    fn test_all_builtin_themes_valid() {
        for name in Theme::available_themes() {
            let theme = Theme::by_name(name).unwrap_or_else(|| panic!("Theme {} should exist", name));
            assert!(!theme.name.is_empty());
        }
    }

    #[test]
    fn test_theme_colors_different() {
        let dracula = Theme::dracula();
        let nord = Theme::nord();

        // Themes should have different primary colors
        assert_ne!(dracula.primary, nord.primary);
        assert_ne!(dracula.background, nord.background);
    }
}
