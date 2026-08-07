use crossterm::style::Color;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub description: String,
    pub ui: UiColors,
    pub syntax: SyntaxColors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiColors {
    pub background: String,
    pub foreground: String,
    #[serde(default = "default_cursor")]
    pub cursor: String,
    #[serde(default = "default_cursor_line")]
    pub cursor_line: String,
    #[serde(default = "default_selection")]
    pub selection: String,
    #[serde(default = "default_line_number")]
    pub line_number: String,
    #[serde(default = "default_line_number_active")]
    pub line_number_active: String,
    #[serde(default = "default_gutter_bg")]
    pub gutter_bg: String,

    #[serde(default = "default_menu_bar_bg")]
    pub menu_bar_bg: String,
    #[serde(default = "default_menu_bar_fg")]
    pub menu_bar_fg: String,
    #[serde(default = "default_menu_selected_bg")]
    pub menu_selected_bg: String,
    #[serde(default = "default_menu_selected_fg")]
    pub menu_selected_fg: String,
    #[serde(default = "default_menu_shortcut_fg")]
    pub menu_shortcut_fg: String,
    #[serde(default = "default_menu_border")]
    pub menu_border: String,

    #[serde(default = "default_status_bar_bg")]
    pub status_bar_bg: String,
    #[serde(default = "default_status_bar_fg")]
    pub status_bar_fg: String,
    #[serde(default = "default_status_bar_accent_bg")]
    pub status_bar_accent_bg: String,
    #[serde(default = "default_status_bar_accent_fg")]
    pub status_bar_accent_fg: String,

    #[serde(default = "default_tab_active_bg")]
    pub tab_active_bg: String,
    #[serde(default = "default_tab_active_fg")]
    pub tab_active_fg: String,
    #[serde(default = "default_tab_inactive_bg")]
    pub tab_inactive_bg: String,
    #[serde(default = "default_tab_inactive_fg")]
    pub tab_inactive_fg: String,
    #[serde(default = "default_tab_modified")]
    pub tab_modified: String,

    #[serde(default = "default_dialog_bg")]
    pub dialog_bg: String,
    #[serde(default = "default_dialog_fg")]
    pub dialog_fg: String,
    #[serde(default = "default_dialog_border")]
    pub dialog_border: String,
    #[serde(default = "default_dialog_title")]
    pub dialog_title: String,
    #[serde(default = "default_dialog_input_bg")]
    pub dialog_input_bg: String,
    #[serde(default = "default_dialog_input_fg")]
    pub dialog_input_fg: String,
    #[serde(default = "default_dialog_button_bg")]
    pub dialog_button_bg: String,
    #[serde(default = "default_dialog_button_fg")]
    pub dialog_button_fg: String,
    #[serde(default = "default_dialog_button_active_bg")]
    pub dialog_button_active_bg: String,
    #[serde(default = "default_dialog_button_active_fg")]
    pub dialog_button_active_fg: String,

    #[serde(default = "default_search_match_bg")]
    pub search_match_bg: String,
    #[serde(default = "default_search_match_fg")]
    pub search_match_fg: String,
    #[serde(default = "default_search_match_active_bg")]
    pub search_match_active_bg: String,
    #[serde(default = "default_search_match_active_fg")]
    pub search_match_active_fg: String,

    #[serde(default = "default_finder_bg")]
    pub finder_bg: String,
    #[serde(default = "default_finder_fg")]
    pub finder_fg: String,
    #[serde(default = "default_finder_border")]
    pub finder_border: String,
    #[serde(default = "default_finder_match_fg")]
    pub finder_match_fg: String,
    #[serde(default = "default_finder_selected_bg")]
    pub finder_selected_bg: String,
    #[serde(default = "default_finder_selected_fg")]
    pub finder_selected_fg: String,
    #[serde(default = "default_finder_preview_bg")]
    pub finder_preview_bg: String,

    #[serde(default = "default_scrollbar_track")]
    pub scrollbar_track: String,
    #[serde(default = "default_scrollbar_thumb")]
    pub scrollbar_thumb: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxColors {
    #[serde(default = "default_keyword")]
    pub keyword: String,
    #[serde(default = "default_function")]
    pub function: String,
    #[serde(default = "default_type")]
    pub r#type: String,
    #[serde(default = "default_string")]
    pub string: String,
    #[serde(default = "default_comment")]
    pub comment: String,
    #[serde(default = "default_number")]
    pub number: String,
    #[serde(default = "default_operator")]
    pub operator: String,
    #[serde(default = "default_variable")]
    pub variable: String,
    #[serde(default = "default_constant")]
    pub constant: String,
    #[serde(default = "default_attribute")]
    pub attribute: String,
    #[serde(default = "default_tag")]
    pub tag: String,
    #[serde(default = "default_punctuation")]
    pub punctuation: String,
    #[serde(default = "default_error")]
    pub error: String,
}

// Fallback defaults
fn default_cursor() -> String { "#f8f8f0".into() }
fn default_cursor_line() -> String { "#3e3d32".into() }
fn default_selection() -> String { "#49483e".into() }
fn default_line_number() -> String { "#90908a".into() }
fn default_line_number_active() -> String { "#f8f8f2".into() }
fn default_gutter_bg() -> String { "#272822".into() }
fn default_menu_bar_bg() -> String { "#3e3d32".into() }
fn default_menu_bar_fg() -> String { "#f8f8f2".into() }
fn default_menu_selected_bg() -> String { "#a6e22e".into() }
fn default_menu_selected_fg() -> String { "#272822".into() }
fn default_menu_shortcut_fg() -> String { "#fd971f".into() }
fn default_menu_border() -> String { "#75715e".into() }
fn default_status_bar_bg() -> String { "#3e3d32".into() }
fn default_status_bar_fg() -> String { "#f8f8f2".into() }
fn default_status_bar_accent_bg() -> String { "#66d9ef".into() }
fn default_status_bar_accent_fg() -> String { "#272822".into() }
fn default_tab_active_bg() -> String { "#272822".into() }
fn default_tab_active_fg() -> String { "#f8f8f2".into() }
fn default_tab_inactive_bg() -> String { "#1e1f1c".into() }
fn default_tab_inactive_fg() -> String { "#75715e".into() }
fn default_tab_modified() -> String { "#fd971f".into() }
fn default_dialog_bg() -> String { "#272822".into() }
fn default_dialog_fg() -> String { "#f8f8f2".into() }
fn default_dialog_border() -> String { "#66d9ef".into() }
fn default_dialog_title() -> String { "#a6e22e".into() }
fn default_dialog_input_bg() -> String { "#3e3d32".into() }
fn default_dialog_input_fg() -> String { "#f8f8f2".into() }
fn default_dialog_button_bg() -> String { "#3e3d32".into() }
fn default_dialog_button_fg() -> String { "#f8f8f2".into() }
fn default_dialog_button_active_bg() -> String { "#a6e22e".into() }
fn default_dialog_button_active_fg() -> String { "#272822".into() }
fn default_search_match_bg() -> String { "#e6db74".into() }
fn default_search_match_fg() -> String { "#272822".into() }
fn default_search_match_active_bg() -> String { "#fd971f".into() }
fn default_search_match_active_fg() -> String { "#272822".into() }
fn default_finder_bg() -> String { "#272822".into() }
fn default_finder_fg() -> String { "#f8f8f2".into() }
fn default_finder_border() -> String { "#a6e22e".into() }
fn default_finder_match_fg() -> String { "#66d9ef".into() }
fn default_finder_selected_bg() -> String { "#49483e".into() }
fn default_finder_selected_fg() -> String { "#a6e22e".into() }
fn default_finder_preview_bg() -> String { "#1e1f1c".into() }
fn default_scrollbar_track() -> String { "#272822".into() }
fn default_scrollbar_thumb() -> String { "#75715e".into() }

fn default_keyword() -> String { "#f92672".into() }
fn default_function() -> String { "#a6e22e".into() }
fn default_type() -> String { "#66d9ef".into() }
fn default_string() -> String { "#e6db74".into() }
fn default_comment() -> String { "#75715e".into() }
fn default_number() -> String { "#ae81ff".into() }
fn default_operator() -> String { "#f92672".into() }
fn default_variable() -> String { "#f8f8f2".into() }
fn default_constant() -> String { "#ae81ff".into() }
fn default_attribute() -> String { "#a6e22e".into() }
fn default_tag() -> String { "#f92672".into() }
fn default_punctuation() -> String { "#f8f8f2".into() }
fn default_error() -> String { "#f92672".into() }

pub fn parse_color(hex_str: &str) -> Color {
    let s = hex_str.trim().trim_start_matches('#');
    if s.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&s[0..2], 16),
            u8::from_str_radix(&s[2..4], 16),
            u8::from_str_radix(&s[4..6], 16),
        ) {
            return Color::Rgb { r, g, b };
        }
    } else if s.len() == 3 {
        let r = u8::from_str_radix(&s[0..1], 16).unwrap_or(0) * 17;
        let g = u8::from_str_radix(&s[1..2], 16).unwrap_or(0) * 17;
        let b = u8::from_str_radix(&s[2..3], 16).unwrap_or(0) * 17;
        return Color::Rgb { r, g, b };
    }
    match hex_str.to_lowercase().as_str() {
        "black" => Color::Black,
        "dark_grey" | "darkgrey" => Color::DarkGrey,
        "red" | "dark_red" => Color::DarkRed,
        "green" | "dark_green" => Color::DarkGreen,
        "yellow" | "dark_yellow" => Color::DarkYellow,
        "blue" | "dark_blue" => Color::DarkBlue,
        "magenta" | "dark_magenta" => Color::DarkMagenta,
        "cyan" | "dark_cyan" => Color::DarkCyan,
        "white" => Color::White,
        "grey" | "gray" => Color::Grey,
        _ => Color::Reset,
    }
}

pub struct ThemeManager {
    pub themes: HashMap<String, Theme>,
    pub current_theme: Theme,
    pub theme_names: Vec<String>,
}

impl ThemeManager {
    pub fn new() -> Self {
        let mut manager = Self {
            themes: HashMap::new(),
            current_theme: Self::embedded_monokai(),
            theme_names: Vec::new(),
        };

        // Load built-in embedded themes
        manager.add_theme(Self::embedded_monokai());
        manager.add_theme(Self::embedded_dracula());
        manager.add_theme(Self::embedded_nord());
        manager.add_theme(Self::embedded_tokyo_night());
        manager.add_theme(Self::embedded_catppuccin_mocha());
        manager.add_theme(Self::embedded_gruvbox_dark());
        manager.add_theme(Self::embedded_solarized_dark());
        manager.add_theme(Self::embedded_one_dark());
        manager.add_theme(Self::embedded_github_dark());
        manager.add_theme(Self::embedded_classic_dos());
        manager.add_theme(Self::embedded_cyberpunk());

        // Discover and load external themes from ./themes and config directory
        manager.load_external_themes();

        manager.set_theme("Monokai");
        manager
    }

    pub fn add_theme(&mut self, theme: Theme) {
        let name = theme.name.clone();
        if !self.theme_names.contains(&name) {
            self.theme_names.push(name.clone());
        }
        self.themes.insert(name, theme);
    }

    pub fn set_theme(&mut self, name: &str) -> bool {
        if let Some(t) = self.themes.get(name) {
            self.current_theme = t.clone();
            true
        } else {
            // Case-insensitive search
            let found = self.themes.iter().find(|(k, _)| k.eq_ignore_ascii_case(name));
            if let Some((_, t)) = found {
                self.current_theme = t.clone();
                true
            } else {
                false
            }
        }
    }

    pub fn load_external_themes(&mut self) {
        let mut search_paths = vec![
            PathBuf::from("themes"),
            PathBuf::from("../themes"),
            PathBuf::from("../../themes"),
        ];

        if let Some(config_dir) = dirs::config_dir() {
            search_paths.push(config_dir.join("edit-ng").join("themes"));
        }

        for path in search_paths {
            if path.is_dir() {
                if let Ok(entries) = fs::read_dir(path) {
                    for entry in entries.flatten() {
                        let file_path = entry.path();
                        if file_path.extension().and_then(|s| s.to_str()) == Some("toml") {
                            if let Ok(content) = fs::read_to_string(&file_path) {
                                if let Ok(theme) = toml::from_str::<Theme>(&content) {
                                    self.add_theme(theme);
                                }
                            }
                        }
                    }
                }
            }
        }
        self.theme_names.sort();
    }

    pub fn embedded_monokai() -> Theme {
        toml::from_str(include_str!("../../../themes/monokai.toml"))
            .unwrap_or_else(|_| Theme {
                name: "Monokai".into(),
                author: "edit-ng".into(),
                description: "Vibrant Monokai".into(),
                ui: UiColors {
                    background: "#272822".into(),
                    foreground: "#f8f8f2".into(),
                    cursor: "#f8f8f0".into(),
                    cursor_line: "#3e3d32".into(),
                    selection: "#49483e".into(),
                    line_number: "#90908a".into(),
                    line_number_active: "#f8f8f2".into(),
                    gutter_bg: "#272822".into(),
                    menu_bar_bg: "#3e3d32".into(),
                    menu_bar_fg: "#f8f8f2".into(),
                    menu_selected_bg: "#a6e22e".into(),
                    menu_selected_fg: "#272822".into(),
                    menu_shortcut_fg: "#fd971f".into(),
                    menu_border: "#75715e".into(),
                    status_bar_bg: "#3e3d32".into(),
                    status_bar_fg: "#f8f8f2".into(),
                    status_bar_accent_bg: "#66d9ef".into(),
                    status_bar_accent_fg: "#272822".into(),
                    tab_active_bg: "#272822".into(),
                    tab_active_fg: "#f8f8f2".into(),
                    tab_inactive_bg: "#1e1f1c".into(),
                    tab_inactive_fg: "#75715e".into(),
                    tab_modified: "#fd971f".into(),
                    dialog_bg: "#272822".into(),
                    dialog_fg: "#f8f8f2".into(),
                    dialog_border: "#66d9ef".into(),
                    dialog_title: "#a6e22e".into(),
                    dialog_input_bg: "#3e3d32".into(),
                    dialog_input_fg: "#f8f8f2".into(),
                    dialog_button_bg: "#3e3d32".into(),
                    dialog_button_fg: "#f8f8f2".into(),
                    dialog_button_active_bg: "#a6e22e".into(),
                    dialog_button_active_fg: "#272822".into(),
                    search_match_bg: "#e6db74".into(),
                    search_match_fg: "#272822".into(),
                    search_match_active_bg: "#fd971f".into(),
                    search_match_active_fg: "#272822".into(),
                    finder_bg: "#272822".into(),
                    finder_fg: "#f8f8f2".into(),
                    finder_border: "#a6e22e".into(),
                    finder_match_fg: "#66d9ef".into(),
                    finder_selected_bg: "#49483e".into(),
                    finder_selected_fg: "#a6e22e".into(),
                    finder_preview_bg: "#1e1f1c".into(),
                    scrollbar_track: "#272822".into(),
                    scrollbar_thumb: "#75715e".into(),
                },
                syntax: SyntaxColors {
                    keyword: "#f92672".into(),
                    function: "#a6e22e".into(),
                    r#type: "#66d9ef".into(),
                    string: "#e6db74".into(),
                    comment: "#75715e".into(),
                    number: "#ae81ff".into(),
                    operator: "#f92672".into(),
                    variable: "#f8f8f2".into(),
                    constant: "#ae81ff".into(),
                    attribute: "#a6e22e".into(),
                    tag: "#f92672".into(),
                    punctuation: "#f8f8f2".into(),
                    error: "#f92672".into(),
                },
            })
    }

    pub fn embedded_dracula() -> Theme {
        toml::from_str(include_str!("../../../themes/dracula.toml")).unwrap_or_else(|_| Self::embedded_monokai())
    }

    pub fn embedded_nord() -> Theme {
        toml::from_str(include_str!("../../../themes/nord.toml")).unwrap_or_else(|_| Self::embedded_monokai())
    }

    pub fn embedded_tokyo_night() -> Theme {
        toml::from_str(include_str!("../../../themes/tokyo-night.toml")).unwrap_or_else(|_| Self::embedded_monokai())
    }

    pub fn embedded_catppuccin_mocha() -> Theme {
        toml::from_str(include_str!("../../../themes/catppuccin-mocha.toml")).unwrap_or_else(|_| Self::embedded_monokai())
    }

    pub fn embedded_gruvbox_dark() -> Theme {
        toml::from_str(include_str!("../../../themes/gruvbox-dark.toml")).unwrap_or_else(|_| Self::embedded_monokai())
    }

    pub fn embedded_solarized_dark() -> Theme {
        toml::from_str(include_str!("../../../themes/solarized-dark.toml")).unwrap_or_else(|_| Self::embedded_monokai())
    }

    pub fn embedded_one_dark() -> Theme {
        toml::from_str(include_str!("../../../themes/one-dark.toml")).unwrap_or_else(|_| Self::embedded_monokai())
    }

    pub fn embedded_github_dark() -> Theme {
        toml::from_str(include_str!("../../../themes/github-dark.toml")).unwrap_or_else(|_| Self::embedded_monokai())
    }

    pub fn embedded_classic_dos() -> Theme {
        toml::from_str(include_str!("../../../themes/classic-dos.toml")).unwrap_or_else(|_| Self::embedded_monokai())
    }

    pub fn embedded_cyberpunk() -> Theme {
        toml::from_str(include_str!("../../../themes/cyberpunk.toml")).unwrap_or_else(|_| Self::embedded_monokai())
    }
}
