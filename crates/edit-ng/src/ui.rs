use crate::buffer::Buffer;
use crate::dialog::{DialogState, FindField};
use crate::i18n::I18n;
use crate::syntax::TokenType;
use crate::theme::{parse_color, Theme, ThemeManager};
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::style::{ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::QueueableCommand;
use std::io::{self, Write};

#[derive(Clone, Debug)]
pub struct MenuDefinition {
    pub title: String,
    pub shortcut_key: char,
    pub items: Vec<MenuItem>,
}

#[derive(Clone, Debug)]
pub struct MenuItem {
    pub label: String,
    pub shortcut: String,
    pub action_id: String,
    pub is_separator: bool,
}

impl MenuItem {
    pub fn new(label: impl Into<String>, shortcut: impl Into<String>, action_id: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            shortcut: shortcut.into(),
            action_id: action_id.into(),
            is_separator: false,
        }
    }

    pub fn separator() -> Self {
        Self {
            label: String::new(),
            shortcut: String::new(),
            action_id: String::new(),
            is_separator: true,
        }
    }
}

pub fn get_menus(theme_manager: &ThemeManager, i18n: &I18n) -> Vec<MenuDefinition> {
    // 1. File Menu
    let file_menu = MenuDefinition {
        title: "File".into(),
        shortcut_key: 'F',
        items: vec![
            MenuItem::new("New", "Ctrl+N", "file_new"),
            MenuItem::new("Open...", "Ctrl+O", "file_open"),
            MenuItem::new("Quick Open (Fuzzy)...", "Ctrl+P", "fuzzy_finder"),
            MenuItem::separator(),
            MenuItem::new("Save", "Ctrl+S", "file_save"),
            MenuItem::new("Save As...", "Ctrl+Shift+S", "file_save_as"),
            MenuItem::separator(),
            MenuItem::new("Close File", "Ctrl+W", "file_close"),
            MenuItem::new("Exit", "Ctrl+Q", "file_exit"),
        ],
    };

    // 2. Edit Menu
    let edit_menu = MenuDefinition {
        title: "Edit".into(),
        shortcut_key: 'E',
        items: vec![
            MenuItem::new("Undo", "Ctrl+Z", "edit_undo"),
            MenuItem::new("Redo", "Ctrl+Y", "edit_redo"),
            MenuItem::separator(),
            MenuItem::new("Cut", "Ctrl+X", "edit_cut"),
            MenuItem::new("Copy", "Ctrl+C", "edit_copy"),
            MenuItem::new("Paste", "Ctrl+V", "edit_paste"),
            MenuItem::separator(),
            MenuItem::new("Find & Replace...", "Ctrl+F", "edit_find"),
            MenuItem::new("Go to Line...", "Ctrl+G", "edit_goto"),
            MenuItem::separator(),
            MenuItem::new("Duplicate Line", "Ctrl+D", "edit_duplicate"),
            MenuItem::new("Delete Line", "Ctrl+Shift+K", "edit_delete_line"),
            MenuItem::new("Move Line Up", "Alt+Up", "edit_move_up"),
            MenuItem::new("Move Line Down", "Alt+Down", "edit_move_down"),
            MenuItem::separator(),
            MenuItem::new("Select All", "Ctrl+A", "edit_select_all"),
        ],
    };

    // 3. View Menu
    let view_menu = MenuDefinition {
        title: "View".into(),
        shortcut_key: 'V',
        items: vec![
            MenuItem::new("Document Statistics...", "Ctrl+Shift+C", "view_word_count"),
            MenuItem::new("Select Language...", "Ctrl+L", "view_language"),
            MenuItem::new("Select Theme...", "F2", "theme_picker"),
            MenuItem::separator(),
            MenuItem::new("Next Tab", "Ctrl+Tab", "tab_next"),
            MenuItem::new("Previous Tab", "Ctrl+Shift+Tab", "tab_prev"),
        ],
    };

    // 4. Plugins Menu
    let plugins_menu = MenuDefinition {
        title: "Plugins".into(),
        shortcut_key: 'P',
        items: vec![
            MenuItem::new("Fuzzy File Finder", "Ctrl+P", "fuzzy_finder"),
            MenuItem::new("Git Status & Gutter Diff", "", "plugin_git"),
            MenuItem::new("Document Statistics", "Ctrl+Shift+C", "view_word_count"),
        ],
    };

    // 5. Themes Menu (Dynamic listing with ✓ on current theme)
    let mut theme_items = vec![
        MenuItem::new("Select Color Theme...", "F2", "theme_picker"),
        MenuItem::separator(),
    ];
    let current_theme_name = &theme_manager.current_theme.name;
    for name in &theme_manager.theme_names {
        let is_current = name == current_theme_name;
        let prefix = if is_current { "✓ " } else { "  " };
        let label = format!("{}{}", prefix, name);
        let action = format!("set_theme_{}", name);
        theme_items.push(MenuItem::new(label, "", action));
    }
    let themes_menu = MenuDefinition {
        title: "Themes".into(),
        shortcut_key: 'T',
        items: theme_items,
    };

    // 6. Language Menu (Dynamic listing with ✓ on current language)
    let mut lang_items = vec![
        MenuItem::new("Select Language...", "Ctrl+L", "view_language"),
        MenuItem::separator(),
    ];
    let current_lang_code = i18n.current_language();
    for lang in i18n.available_languages.iter().take(12) {
        let is_current = lang.code.eq_ignore_ascii_case(current_lang_code);
        let prefix = if is_current { "✓ " } else { "  " };
        let label = format!("{}{:<12} ({})", prefix, lang.native_name, lang.name);
        let action = format!("set_lang_{}", lang.code);
        lang_items.push(MenuItem::new(label, "", action));
    }
    let language_menu = MenuDefinition {
        title: "Language".into(),
        shortcut_key: 'L',
        items: lang_items,
    };

    // 7. Help Menu
    let help_menu = MenuDefinition {
        title: "Help".into(),
        shortcut_key: 'H',
        items: vec![
            MenuItem::new("Keyboard Shortcuts", "F1", "help_shortcuts"),
            MenuItem::new("About edit-ng", "", "help_about"),
        ],
    };

    vec![
        file_menu,
        edit_menu,
        view_menu,
        plugins_menu,
        themes_menu,
        language_menu,
        help_menu,
    ]
}

pub fn get_navbar_button_ranges(menus: &[MenuDefinition], i18n: &I18n) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut x = 9; // brand width: " edit-ng "
    for menu in menus {
        let t = i18n.t(&menu.title);
        let len = t.chars().count() + 2;
        ranges.push((x, x + len));
        x += len;
    }
    ranges
}

pub fn get_menu_dropdown_geometry(menus: &[MenuDefinition], menu_idx: usize, i18n: &I18n) -> Option<(usize, usize, usize, usize)> {
    let menu = menus.get(menu_idx)?;
    let mut menu_x = 9;
    for i in 0..menu_idx {
        let t = i18n.t(&menus[i].title);
        menu_x += t.chars().count() + 2;
    }

    let mut max_label_len = 10;
    let mut max_shortcut_len = 0;
    for item in &menu.items {
        if !item.is_separator {
            let l_len = if item.action_id.starts_with("set_theme_") || item.action_id.starts_with("set_lang_") {
                item.label.chars().count()
            } else {
                i18n.t(&item.label).chars().count()
            };
            max_label_len = max_label_len.max(l_len);
            max_shortcut_len = max_shortcut_len.max(item.shortcut.chars().count());
        }
    }

    let inner_width = max_label_len + max_shortcut_len + 4;
    let dropdown_width = inner_width + 2;
    let dropdown_height = menu.items.len() + 2;

    Some((menu_x, 1, dropdown_width, dropdown_height))
}

pub fn get_tab_button_ranges(buffers: &[Buffer]) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut x = 0;
    for (i, buf) in buffers.iter().enumerate() {
        let mod_mark = if buf.is_modified { " ●" } else { "" };
        let title = format!(" {} {}{} ", if i == 0 { "▎" } else { " " }, buf.title, mod_mark);
        let len = title.chars().count();
        ranges.push((x, x + len));
        x += len;
    }
    ranges
}

pub struct Renderer;

impl Renderer {
    pub fn render<W: Write>(
        stdout: &mut W,
        buffers: &[Buffer],
        active_buf_idx: usize,
        active_menu: Option<(usize, usize)>, // (menu_index, item_index)
        dialog: Option<&DialogState>,
        theme_manager: &ThemeManager,
        i18n: &I18n,
        term_width: u16,
        term_height: u16,
        status_msg: Option<&str>,
    ) -> io::Result<()> {
        let theme = &theme_manager.current_theme;
        let width = term_width as usize;
        let height = term_height as usize;

        if width < 20 || height < 6 {
            return Ok(());
        }

        let menus = get_menus(theme_manager, i18n);

        stdout.queue(Hide)?;

        // 1. Draw Menu Bar (row 0)
        Self::draw_menu_bar(stdout, &menus, active_menu, theme, i18n, width)?;

        // 2. Draw Tab Bar (row 1)
        Self::draw_tab_bar(stdout, buffers, active_buf_idx, theme, width)?;

        // 3. Draw Editor Area (rows 2 .. height - 2)
        let editor_height = height.saturating_sub(3);
        if let Some(buf) = buffers.get(active_buf_idx) {
            Self::draw_editor(stdout, buf, theme, width, editor_height, 2)?;
        }

        // 4. Draw Status Bar (row height - 1)
        Self::draw_status_bar(stdout, buffers.get(active_buf_idx), theme, i18n, width, height - 1, status_msg)?;

        // 5. Draw Active Menu Dropdown Overlay if open
        if let Some((menu_idx, item_idx)) = active_menu {
            Self::draw_menu_dropdown(stdout, &menus, menu_idx, item_idx, theme, i18n, width, height)?;
        }

        // 6. Draw Modal Dialog Overlay if open
        if let Some(d) = dialog {
            Self::draw_dialog(stdout, d, theme_manager, i18n, width, height)?;
        }

        // 7. Place Cursor
        if dialog.is_none() && active_menu.is_none() {
            if let Some(buf) = buffers.get(active_buf_idx) {
                let gutter_w = format!("{}", buf.line_count()).len().max(3) + 2;
                if buf.cursor_row >= buf.scroll_top && buf.cursor_row < buf.scroll_top + editor_height {
                    let screen_y = 2 + (buf.cursor_row - buf.scroll_top);
                    if buf.cursor_col >= buf.scroll_left && buf.cursor_col < buf.scroll_left + (width.saturating_sub(gutter_w)) {
                        let screen_x = gutter_w + (buf.cursor_col - buf.scroll_left);
                        stdout.queue(MoveTo(screen_x as u16, screen_y as u16))?;
                        stdout.queue(Show)?;
                    }
                }
            }
        }

        stdout.queue(ResetColor)?;
        stdout.flush()?;
        Ok(())
    }

    fn draw_menu_bar<W: Write>(
        stdout: &mut W,
        menus: &[MenuDefinition],
        active_menu: Option<(usize, usize)>,
        theme: &Theme,
        i18n: &I18n,
        width: usize,
    ) -> io::Result<()> {
        stdout.queue(MoveTo(0, 0))?;
        stdout.queue(SetBackgroundColor(parse_color(&theme.ui.menu_bar_bg)))?;
        stdout.queue(SetForegroundColor(parse_color(&theme.ui.menu_bar_fg)))?;

        let mut x = 0;

        let brand = " edit-ng ";
        stdout.queue(SetForegroundColor(parse_color(&theme.ui.menu_shortcut_fg)))?;
        stdout.write_all(brand.as_bytes())?;
        x += brand.len();

        for (m_idx, menu) in menus.iter().enumerate() {
            let is_active = active_menu.map_or(false, |(m, _)| m == m_idx);
            let translated = i18n.t(&menu.title);
            let item_str = format!(" {} ", translated);

            if is_active {
                stdout.queue(SetBackgroundColor(parse_color(&theme.ui.menu_selected_bg)))?;
                stdout.queue(SetForegroundColor(parse_color(&theme.ui.menu_selected_fg)))?;
            } else {
                stdout.queue(SetBackgroundColor(parse_color(&theme.ui.menu_bar_bg)))?;
                stdout.queue(SetForegroundColor(parse_color(&theme.ui.menu_bar_fg)))?;
            }

            stdout.write_all(item_str.as_bytes())?;
            x += item_str.len();
        }

        stdout.queue(SetBackgroundColor(parse_color(&theme.ui.menu_bar_bg)))?;
        if x < width {
            let fill = " ".repeat(width - x);
            stdout.write_all(fill.as_bytes())?;
        }

        Ok(())
    }

    fn draw_tab_bar<W: Write>(
        stdout: &mut W,
        buffers: &[Buffer],
        active_idx: usize,
        theme: &Theme,
        width: usize,
    ) -> io::Result<()> {
        stdout.queue(MoveTo(0, 1))?;
        stdout.queue(SetBackgroundColor(parse_color(&theme.ui.tab_inactive_bg)))?;

        let mut x = 0;
        for (i, buf) in buffers.iter().enumerate() {
            let is_active = i == active_idx;
            let mod_mark = if buf.is_modified { " ●" } else { "" };
            let title = format!(" {} {}{} ", if is_active { "▎" } else { " " }, buf.title, mod_mark);

            if is_active {
                stdout.queue(SetBackgroundColor(parse_color(&theme.ui.tab_active_bg)))?;
                stdout.queue(SetForegroundColor(parse_color(&theme.ui.tab_active_fg)))?;
            } else {
                stdout.queue(SetBackgroundColor(parse_color(&theme.ui.tab_inactive_bg)))?;
                stdout.queue(SetForegroundColor(parse_color(&theme.ui.tab_inactive_fg)))?;
            }

            stdout.write_all(title.as_bytes())?;
            x += title.chars().count();
        }

        stdout.queue(SetBackgroundColor(parse_color(&theme.ui.tab_inactive_bg)))?;
        if x < width {
            let fill = " ".repeat(width - x);
            stdout.write_all(fill.as_bytes())?;
        }

        Ok(())
    }

    fn draw_editor<W: Write>(
        stdout: &mut W,
        buffer: &Buffer,
        theme: &Theme,
        width: usize,
        height: usize,
        start_y: usize,
    ) -> io::Result<()> {
        let gutter_digits = format!("{}", buffer.line_count()).len().max(3);
        let gutter_width = gutter_digits + 2;
        let content_width = width.saturating_sub(gutter_width + 1);

        let sel_range = buffer.selection_range();

        for screen_row in 0..height {
            let y = start_y + screen_row;
            let file_row = buffer.scroll_top + screen_row;

            stdout.queue(MoveTo(0, y as u16))?;

            if file_row < buffer.line_count() {
                let is_current_line = file_row == buffer.cursor_row;

                // Line Number Gutter
                stdout.queue(SetBackgroundColor(parse_color(&theme.ui.gutter_bg)))?;
                if is_current_line {
                    stdout.queue(SetForegroundColor(parse_color(&theme.ui.line_number_active)))?;
                } else {
                    stdout.queue(SetForegroundColor(parse_color(&theme.ui.line_number)))?;
                }

                let gutter_str = format!("{:>width$} │", file_row + 1, width = gutter_digits);
                stdout.write_all(gutter_str.as_bytes())?;

                // Line Background
                let line_bg = if is_current_line {
                    parse_color(&theme.ui.cursor_line)
                } else {
                    parse_color(&theme.ui.background)
                };
                stdout.queue(SetBackgroundColor(line_bg))?;

                // Syntax Highlights & Content
                let line_text = &buffer.lines[file_row];
                let spans = buffer.syntax_highlighter.highlight_line(file_row, line_text);
                let chars: Vec<char> = line_text.chars().collect();

                let col = buffer.scroll_left;
                let end_col = buffer.scroll_left + content_width;

                for c_idx in col..end_col {
                    if c_idx < chars.len() {
                        let ch = chars[c_idx];

                        let is_selected = sel_range.map_or(false, |((s_r, s_c), (e_r, e_c))| {
                            if file_row > s_r && file_row < e_r {
                                true
                            } else if file_row == s_r && file_row == e_r {
                                c_idx >= s_c && c_idx < e_c
                            } else if file_row == s_r {
                                c_idx >= s_c
                            } else if file_row == e_r {
                                c_idx < e_c
                            } else {
                                false
                            }
                        });

                        let search_match = buffer.search_matches.iter().enumerate().find(|(_, m)| {
                            m.row == file_row && c_idx >= m.start_col && c_idx < m.end_col
                        });

                        if is_selected {
                            stdout.queue(SetBackgroundColor(parse_color(&theme.ui.selection)))?;
                            stdout.queue(SetForegroundColor(parse_color(&theme.ui.foreground)))?;
                        } else if let Some((m_idx, _)) = search_match {
                            if m_idx == buffer.active_match_index {
                                stdout.queue(SetBackgroundColor(parse_color(&theme.ui.search_match_active_bg)))?;
                                stdout.queue(SetForegroundColor(parse_color(&theme.ui.search_match_active_fg)))?;
                            } else {
                                stdout.queue(SetBackgroundColor(parse_color(&theme.ui.search_match_bg)))?;
                                stdout.queue(SetForegroundColor(parse_color(&theme.ui.search_match_fg)))?;
                            }
                        } else {
                            stdout.queue(SetBackgroundColor(line_bg))?;
                            let token_type = spans
                                .iter()
                                .find(|s| c_idx >= s.start_col && c_idx < s.end_col)
                                .map(|s| s.token_type)
                                .unwrap_or(TokenType::Normal);
                            stdout.queue(SetForegroundColor(token_type.to_color(theme)))?;
                        }

                        let mut buf_c = [0u8; 4];
                        stdout.write_all(ch.encode_utf8(&mut buf_c).as_bytes())?;
                    } else {
                        stdout.queue(SetBackgroundColor(line_bg))?;
                        stdout.write_all(b" ")?;
                    }
                }

                // Scrollbar
                let scrollbar_thumb_y = if buffer.line_count() > height {
                    (buffer.scroll_top * height) / buffer.line_count()
                } else {
                    0
                };
                if screen_row == scrollbar_thumb_y {
                    stdout.queue(SetBackgroundColor(parse_color(&theme.ui.scrollbar_thumb)))?;
                } else {
                    stdout.queue(SetBackgroundColor(parse_color(&theme.ui.scrollbar_track)))?;
                }
                stdout.write_all(b" ")?;
            } else {
                stdout.queue(SetBackgroundColor(parse_color(&theme.ui.gutter_bg)))?;
                stdout.queue(SetForegroundColor(parse_color(&theme.ui.line_number)))?;
                let empty_gutter = format!("{:>width$} │", "~", width = gutter_digits);
                stdout.write_all(empty_gutter.as_bytes())?;

                stdout.queue(SetBackgroundColor(parse_color(&theme.ui.background)))?;
                let fill = " ".repeat(content_width + 1);
                stdout.write_all(fill.as_bytes())?;
            }
        }

        Ok(())
    }

    fn draw_status_bar<W: Write>(
        stdout: &mut W,
        buffer: Option<&Buffer>,
        theme: &Theme,
        i18n: &I18n,
        width: usize,
        y: usize,
        status_msg: Option<&str>,
    ) -> io::Result<()> {
        stdout.queue(MoveTo(0, y as u16))?;
        stdout.queue(SetBackgroundColor(parse_color(&theme.ui.status_bar_bg)))?;
        stdout.queue(SetForegroundColor(parse_color(&theme.ui.status_bar_fg)))?;

        let left_part = if let Some(buf) = buffer {
            let mod_flag = if buf.is_modified { " [+]" } else { "" };
            let ro_flag = if buf.read_only { " [RO]" } else { "" };
            format!(" ⎇ main │ {}{}{} ", buf.title, mod_flag, ro_flag)
        } else {
            " edit-ng ".to_string()
        };

        let right_part = if let Some(buf) = buffer {
            let lang_name = buf.syntax_highlighter.language().name();
            let encoding = "UTF-8";
            let indent_str = match buf.indent_type {
                crate::buffer::IndentType::Spaces(s) => format!("Spaces: {}", s),
                crate::buffer::IndentType::Tabs => "Tabs".into(),
            };
            format!(
                " Ln {}, Col {} │ {} │ {} │ {} │ {} │ {} ",
                buf.cursor_row + 1,
                buf.cursor_col + 1,
                indent_str,
                encoding,
                lang_name,
                theme.name,
                i18n.current_language_name(),
            )
        } else {
            format!(" {} │ {} ", theme.name, i18n.current_language_name())
        };

        let msg = status_msg.unwrap_or("");
        let left_len = left_part.chars().count();
        let right_len = right_part.chars().count();

        stdout.queue(SetBackgroundColor(parse_color(&theme.ui.status_bar_accent_bg)))?;
        stdout.queue(SetForegroundColor(parse_color(&theme.ui.status_bar_accent_fg)))?;
        stdout.write_all(b" NORMAL ")?;

        stdout.queue(SetBackgroundColor(parse_color(&theme.ui.status_bar_bg)))?;
        stdout.queue(SetForegroundColor(parse_color(&theme.ui.status_bar_fg)))?;
        stdout.write_all(left_part.as_bytes())?;

        let used_len = 8 + left_len + right_len;
        if used_len < width {
            let space_available = width - used_len;
            if !msg.is_empty() {
                stdout.queue(SetForegroundColor(parse_color(&theme.ui.status_bar_accent_bg)))?;
                let disp_msg = format!(" {} ", msg);
                stdout.write_all(disp_msg.as_bytes())?;
                let remaining = space_available.saturating_sub(disp_msg.chars().count());
                stdout.queue(SetForegroundColor(parse_color(&theme.ui.status_bar_fg)))?;
                stdout.write_all(" ".repeat(remaining).as_bytes())?;
            } else {
                stdout.write_all(" ".repeat(space_available).as_bytes())?;
            }
        }

        stdout.write_all(right_part.as_bytes())?;
        Ok(())
    }

    fn draw_menu_dropdown<W: Write>(
        stdout: &mut W,
        menus: &[MenuDefinition],
        menu_idx: usize,
        selected_item_idx: usize,
        theme: &Theme,
        i18n: &I18n,
        _width: usize,
        _height: usize,
    ) -> io::Result<()> {
        let menu = match menus.get(menu_idx) {
            Some(m) => m,
            None => return Ok(()),
        };

        let mut menu_x = 9;
        for i in 0..menu_idx {
            let t = i18n.t(&menus[i].title);
            menu_x += t.chars().count() + 2;
        }

        let mut max_label_len = 10;
        let mut max_shortcut_len = 0;
        for item in &menu.items {
            if !item.is_separator {
                let l_len = if item.action_id.starts_with("set_theme_") || item.action_id.starts_with("set_lang_") {
                    item.label.chars().count()
                } else {
                    i18n.t(&item.label).chars().count()
                };
                max_label_len = max_label_len.max(l_len);
                max_shortcut_len = max_shortcut_len.max(item.shortcut.chars().count());
            }
        }

        let inner_width = max_label_len + max_shortcut_len + 4;

        // Top Border
        stdout.queue(MoveTo(menu_x as u16, 1))?;
        stdout.queue(SetBackgroundColor(parse_color(&theme.ui.menu_bar_bg)))?;
        stdout.queue(SetForegroundColor(parse_color(&theme.ui.menu_border)))?;
        stdout.write_all(format!("┌{}┐", "─".repeat(inner_width)).as_bytes())?;

        for (item_i, item) in menu.items.iter().enumerate() {
            let y = 2 + item_i;
            stdout.queue(MoveTo(menu_x as u16, y as u16))?;

            if item.is_separator {
                stdout.queue(SetBackgroundColor(parse_color(&theme.ui.menu_bar_bg)))?;
                stdout.queue(SetForegroundColor(parse_color(&theme.ui.menu_border)))?;
                stdout.write_all(format!("├{}┤", "─".repeat(inner_width)).as_bytes())?;
            } else {
                let is_selected = item_i == selected_item_idx;
                let label_text = if item.action_id.starts_with("set_theme_") || item.action_id.starts_with("set_lang_") {
                    item.label.as_str()
                } else {
                    i18n.t(&item.label)
                };
                let shortcut_text = &item.shortcut;

                stdout.queue(SetForegroundColor(parse_color(&theme.ui.menu_border)))?;
                stdout.write_all("│".as_bytes())?;

                if is_selected {
                    stdout.queue(SetBackgroundColor(parse_color(&theme.ui.menu_selected_bg)))?;
                    stdout.queue(SetForegroundColor(parse_color(&theme.ui.menu_selected_fg)))?;
                } else {
                    stdout.queue(SetBackgroundColor(parse_color(&theme.ui.menu_bar_bg)))?;
                    stdout.queue(SetForegroundColor(parse_color(&theme.ui.menu_bar_fg)))?;
                }

                let pad = inner_width.saturating_sub(label_text.chars().count() + shortcut_text.chars().count() + 2);
                let line_str = format!(" {}{}{} ", label_text, " ".repeat(pad), shortcut_text);
                stdout.write_all(line_str.as_bytes())?;

                stdout.queue(SetBackgroundColor(parse_color(&theme.ui.menu_bar_bg)))?;
                stdout.queue(SetForegroundColor(parse_color(&theme.ui.menu_border)))?;
                stdout.write_all("│".as_bytes())?;
            }
        }

        // Bottom Border
        let bottom_y = 2 + menu.items.len();
        stdout.queue(MoveTo(menu_x as u16, bottom_y as u16))?;
        stdout.queue(SetBackgroundColor(parse_color(&theme.ui.menu_bar_bg)))?;
        stdout.queue(SetForegroundColor(parse_color(&theme.ui.menu_border)))?;
        stdout.write_all(format!("└{}┘", "─".repeat(inner_width)).as_bytes())?;

        Ok(())
    }

    fn draw_dialog<W: Write>(
        stdout: &mut W,
        dialog: &DialogState,
        theme_manager: &ThemeManager,
        i18n: &I18n,
        term_width: usize,
        term_height: usize,
    ) -> io::Result<()> {
        let theme = &theme_manager.current_theme;
        match dialog {
            DialogState::FuzzyFinder(finder) => {
                Self::draw_fuzzy_finder(stdout, finder, theme, i18n, term_width, term_height)?;
            }
            DialogState::ThemePicker { selected_index, search_query } => {
                Self::draw_theme_picker(stdout, theme_manager, *selected_index, search_query, term_width, term_height, i18n)?;
            }
            DialogState::LanguagePicker { selected_index, search_query } => {
                Self::draw_language_picker(stdout, i18n, *selected_index, search_query, theme, term_width, term_height)?;
            }
            DialogState::WordCount { lines, words, chars, bytes } => {
                Self::draw_word_count(stdout, *lines, *words, *chars, *bytes, theme, i18n, term_width, term_height)?;
            }
            DialogState::GotoLine { input, cursor } => {
                Self::draw_input_dialog(stdout, "Go to Line", "Line number:", input, *cursor, theme, i18n, term_width, term_height)?;
            }
            DialogState::OpenFile { input, cursor } => {
                Self::draw_input_dialog(stdout, "Open File", "File path:", input, *cursor, theme, i18n, term_width, term_height)?;
            }
            DialogState::SaveAs { input, cursor } => {
                Self::draw_input_dialog(stdout, "Save As", "Save to path:", input, *cursor, theme, i18n, term_width, term_height)?;
            }
            DialogState::ConfirmClose { selected_button, .. } => {
                Self::draw_confirm_dialog(stdout, "Unsaved Changes", "Save changes to file before closing?", *selected_button, theme, i18n, term_width, term_height)?;
            }
            DialogState::FindReplace { find_query, replace_query, match_case, whole_word, use_regex, focused_field, .. } => {
                Self::draw_find_replace(stdout, find_query, replace_query, *match_case, *whole_word, *use_regex, focused_field, theme, i18n, term_width, term_height)?;
            }
            DialogState::About => {
                Self::draw_about_dialog(stdout, theme, i18n, term_width, term_height)?;
            }
            DialogState::ShortcutsHelp => {
                Self::draw_shortcuts_help(stdout, theme, i18n, term_width, term_height)?;
            }
        }
        Ok(())
    }

    fn draw_fuzzy_finder<W: Write>(
        stdout: &mut W,
        finder: &crate::fuzzy::FuzzyFinder,
        theme: &Theme,
        i18n: &I18n,
        width: usize,
        height: usize,
    ) -> io::Result<()> {
        let dialog_w = (width * 85 / 100).max(60).min(width.saturating_sub(4));
        let dialog_h = (height * 80 / 100).max(16).min(height.saturating_sub(4));
        let start_x = (width.saturating_sub(dialog_w)) / 2;
        let start_y = (height.saturating_sub(dialog_h)) / 2;

        let left_w = dialog_w * 48 / 100;
        let right_w = dialog_w.saturating_sub(left_w + 3);

        stdout.queue(SetBackgroundColor(parse_color(&theme.ui.finder_bg)))?;
        stdout.queue(SetForegroundColor(parse_color(&theme.ui.finder_border)))?;

        // Box Frame
        stdout.queue(MoveTo(start_x as u16, start_y as u16))?;
        let title = format!(" 🔍 {} ({}) ", i18n.t("FuzzyFinder"), finder.filtered_items.len());
        let top_pad = dialog_w.saturating_sub(title.chars().count() + 2);
        stdout.write_all(format!("╭{}{}{}╮", title, "─".repeat(top_pad), "").as_bytes())?;

        for y_offset in 1..dialog_h - 1 {
            stdout.queue(MoveTo(start_x as u16, (start_y + y_offset) as u16))?;
            stdout.queue(SetForegroundColor(parse_color(&theme.ui.finder_border)))?;
            stdout.write_all("│".as_bytes())?;

            if y_offset == 1 {
                // Search Input Line
                stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_input_bg)))?;
                stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_input_fg)))?;
                let prompt_text = if finder.query.is_empty() {
                    i18n.t("FuzzyFinderPrompt")
                } else {
                    &finder.query
                };
                let input_pad = (dialog_w - 2).saturating_sub(prompt_text.chars().count() + 2);
                let input_str = format!(" > {}{} ", prompt_text, " ".repeat(input_pad));
                stdout.write_all(input_str.as_bytes())?;
            } else if y_offset == 2 {
                // Divider under input
                stdout.queue(SetBackgroundColor(parse_color(&theme.ui.finder_bg)))?;
                stdout.queue(SetForegroundColor(parse_color(&theme.ui.finder_border)))?;
                stdout.write_all(format!("├{}┬{}┤", "─".repeat(left_w), "─".repeat(right_w + 1)).as_bytes())?;
                continue;
            } else {
                let item_index = y_offset.saturating_sub(3);

                // Left side: Results List
                if let Some(item) = finder.filtered_items.get(item_index) {
                    let is_selected = item_index == finder.selected_index;
                    if is_selected {
                        stdout.queue(SetBackgroundColor(parse_color(&theme.ui.finder_selected_bg)))?;
                        stdout.queue(SetForegroundColor(parse_color(&theme.ui.finder_selected_fg)))?;
                    } else {
                        stdout.queue(SetBackgroundColor(parse_color(&theme.ui.finder_bg)))?;
                        stdout.queue(SetForegroundColor(parse_color(&theme.ui.finder_fg)))?;
                    }

                    let disp_path = &item.display_path;
                    let chars: Vec<char> = disp_path.chars().collect();
                    let mut rendered_len = 0;

                    stdout.write_all(if is_selected { b"> " } else { b"  " })?;
                    rendered_len += 2;

                    for (c_i, &ch) in chars.iter().enumerate() {
                        if rendered_len + 1 >= left_w {
                            break;
                        }
                        if item.matched_indices.contains(&c_i) {
                            stdout.queue(SetForegroundColor(parse_color(&theme.ui.finder_match_fg)))?;
                        } else if is_selected {
                            stdout.queue(SetForegroundColor(parse_color(&theme.ui.finder_selected_fg)))?;
                        } else {
                            stdout.queue(SetForegroundColor(parse_color(&theme.ui.finder_fg)))?;
                        }
                        let mut buf_c = [0u8; 4];
                        stdout.write_all(ch.encode_utf8(&mut buf_c).as_bytes())?;
                        rendered_len += 1;
                    }

                    if rendered_len < left_w {
                        let fill = " ".repeat(left_w - rendered_len);
                        stdout.write_all(fill.as_bytes())?;
                    }
                } else {
                    stdout.queue(SetBackgroundColor(parse_color(&theme.ui.finder_bg)))?;
                    stdout.write_all(" ".repeat(left_w).as_bytes())?;
                }

                // Middle Split Border
                stdout.queue(SetBackgroundColor(parse_color(&theme.ui.finder_bg)))?;
                stdout.queue(SetForegroundColor(parse_color(&theme.ui.finder_border)))?;
                stdout.write_all("│".as_bytes())?;

                // Right side: Live Preview
                stdout.queue(SetBackgroundColor(parse_color(&theme.ui.finder_preview_bg)))?;
                stdout.queue(SetForegroundColor(parse_color(&theme.ui.finder_fg)))?;

                let preview_line_idx = y_offset.saturating_sub(3);
                if let Some(preview_lines) = &finder.preview_content {
                    if let Some(p_line) = preview_lines.get(preview_line_idx) {
                        let p_str: String = p_line.chars().take(right_w.saturating_sub(2)).collect();
                        let pad = right_w.saturating_sub(p_str.chars().count());
                        stdout.write_all(format!(" {}{}", p_str, " ".repeat(pad)).as_bytes())?;
                    } else {
                        stdout.write_all(" ".repeat(right_w + 1).as_bytes())?;
                    }
                } else {
                    stdout.write_all(" ".repeat(right_w + 1).as_bytes())?;
                }
            }

            stdout.queue(SetBackgroundColor(parse_color(&theme.ui.finder_bg)))?;
            stdout.queue(SetForegroundColor(parse_color(&theme.ui.finder_border)))?;
            stdout.write_all("│".as_bytes())?;
        }

        // Bottom Border
        stdout.queue(MoveTo(start_x as u16, (start_y + dialog_h - 1) as u16))?;
        stdout.queue(SetBackgroundColor(parse_color(&theme.ui.finder_bg)))?;
        stdout.queue(SetForegroundColor(parse_color(&theme.ui.finder_border)))?;
        stdout.write_all(format!("╰{}┴{}╯", "─".repeat(left_w), "─".repeat(right_w + 1)).as_bytes())?;

        Ok(())
    }

    fn draw_theme_picker<W: Write>(
        stdout: &mut W,
        theme_manager: &ThemeManager,
        selected_index: usize,
        _search_query: &str,
        width: usize,
        height: usize,
        i18n: &I18n,
    ) -> io::Result<()> {
        let dialog_w = 46.min(width.saturating_sub(4));
        let dialog_h = 16.min(height.saturating_sub(4));
        let start_x = (width.saturating_sub(dialog_w)) / 2;
        let start_y = (height.saturating_sub(dialog_h)) / 2;

        let theme = &theme_manager.current_theme;
        stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_bg)))?;
        stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_border)))?;

        stdout.queue(MoveTo(start_x as u16, start_y as u16))?;
        let title = format!(" 🎨 {} ", i18n.t("ThemeSelectTitle"));
        let pad = dialog_w.saturating_sub(title.chars().count() + 2);
        stdout.write_all(format!("╭{}{}{}╮", title, "─".repeat(pad), "").as_bytes())?;

        for y_offset in 1..dialog_h - 1 {
            stdout.queue(MoveTo(start_x as u16, (start_y + y_offset) as u16))?;
            stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_border)))?;
            stdout.write_all("│".as_bytes())?;

            let item_idx = y_offset - 1;
            if let Some(name) = theme_manager.theme_names.get(item_idx) {
                let is_selected = item_idx == selected_index;
                let is_current = name == &theme.name;

                if is_selected {
                    stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_button_active_bg)))?;
                    stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_button_active_fg)))?;
                } else {
                    stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_bg)))?;
                    stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_fg)))?;
                }

                let check = if is_current { " ✓ " } else { "   " };
                let row_str = format!("{}{:<28}", check, name);
                let row_pad = (dialog_w - 2).saturating_sub(row_str.chars().count());
                stdout.write_all(format!("{}{}", row_str, " ".repeat(row_pad)).as_bytes())?;
            } else {
                stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_bg)))?;
                stdout.write_all(" ".repeat(dialog_w - 2).as_bytes())?;
            }

            stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_bg)))?;
            stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_border)))?;
            stdout.write_all("│".as_bytes())?;
        }

        stdout.queue(MoveTo(start_x as u16, (start_y + dialog_h - 1) as u16))?;
        stdout.write_all(format!("╰{}╯", "─".repeat(dialog_w - 2)).as_bytes())?;
        Ok(())
    }

    fn draw_language_picker<W: Write>(
        stdout: &mut W,
        i18n: &I18n,
        selected_index: usize,
        _search_query: &str,
        theme: &Theme,
        width: usize,
        height: usize,
    ) -> io::Result<()> {
        let dialog_w = 46.min(width.saturating_sub(4));
        let dialog_h = 18.min(height.saturating_sub(4));
        let start_x = (width.saturating_sub(dialog_w)) / 2;
        let start_y = (height.saturating_sub(dialog_h)) / 2;

        stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_bg)))?;
        stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_border)))?;

        stdout.queue(MoveTo(start_x as u16, start_y as u16))?;
        let title = " 🌐 Select Language / भाषा ";
        let pad = dialog_w.saturating_sub(title.chars().count() + 2);
        stdout.write_all(format!("╭{}{}{}╮", title, "─".repeat(pad), "").as_bytes())?;

        for y_offset in 1..dialog_h - 1 {
            stdout.queue(MoveTo(start_x as u16, (start_y + y_offset) as u16))?;
            stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_border)))?;
            stdout.write_all("│".as_bytes())?;

            let item_idx = y_offset - 1;
            if let Some(lang) = i18n.available_languages.get(item_idx) {
                let is_selected = item_idx == selected_index;
                let is_current = lang.code.eq_ignore_ascii_case(i18n.current_language());

                if is_selected {
                    stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_button_active_bg)))?;
                    stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_button_active_fg)))?;
                } else {
                    stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_bg)))?;
                    stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_fg)))?;
                }

                let check = if is_current { " ✓ " } else { "   " };
                let row_str = format!("{}{:<18} ({})", check, lang.native_name, lang.name);
                let row_pad = (dialog_w - 2).saturating_sub(row_str.chars().count());
                stdout.write_all(format!("{}{}", row_str, " ".repeat(row_pad)).as_bytes())?;
            } else {
                stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_bg)))?;
                stdout.write_all(" ".repeat(dialog_w - 2).as_bytes())?;
            }

            stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_bg)))?;
            stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_border)))?;
            stdout.write_all("│".as_bytes())?;
        }

        stdout.queue(MoveTo(start_x as u16, (start_y + dialog_h - 1) as u16))?;
        stdout.write_all(format!("╰{}╯", "─".repeat(dialog_w - 2)).as_bytes())?;
        Ok(())
    }

    fn draw_word_count<W: Write>(
        stdout: &mut W,
        lines: usize,
        words: usize,
        chars: usize,
        bytes: usize,
        theme: &Theme,
        i18n: &I18n,
        width: usize,
        height: usize,
    ) -> io::Result<()> {
        let dialog_w = 40.min(width.saturating_sub(4));
        let dialog_h = 9.min(height.saturating_sub(4));
        let start_x = (width.saturating_sub(dialog_w)) / 2;
        let start_y = (height.saturating_sub(dialog_h)) / 2;

        stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_bg)))?;
        stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_border)))?;

        stdout.queue(MoveTo(start_x as u16, start_y as u16))?;
        let title = format!(" 📊 {} ", i18n.t("WordCountTitle"));
        let pad = dialog_w.saturating_sub(title.chars().count() + 2);
        stdout.write_all(format!("╭{}{}{}╮", title, "─".repeat(pad), "").as_bytes())?;

        let rows = [
            format!("  Lines:        {:>12}", lines),
            format!("  Words:        {:>12}", words),
            format!("  Characters:   {:>12}", chars),
            format!("  UTF-8 Bytes:  {:>12}", bytes),
        ];

        for y_offset in 1..dialog_h - 1 {
            stdout.queue(MoveTo(start_x as u16, (start_y + y_offset) as u16))?;
            stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_bg)))?;
            stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_border)))?;
            stdout.write_all("│".as_bytes())?;

            if y_offset >= 1 && y_offset <= 4 {
                let row_str = &rows[y_offset - 1];
                stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_fg)))?;
                let pad = (dialog_w - 2).saturating_sub(row_str.chars().count());
                stdout.write_all(format!("{}{}", row_str, " ".repeat(pad)).as_bytes())?;
            } else if y_offset == 6 {
                let btn_str = "[ Close (Esc) ]";
                let pad = (dialog_w - 2).saturating_sub(btn_str.len());
                let l_pad = pad / 2;
                let r_pad = pad - l_pad;
                stdout.write_all(" ".repeat(l_pad).as_bytes())?;
                stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_button_active_bg)))?;
                stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_button_active_fg)))?;
                stdout.write_all(btn_str.as_bytes())?;
                stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_bg)))?;
                stdout.write_all(" ".repeat(r_pad).as_bytes())?;
            } else {
                stdout.write_all(" ".repeat(dialog_w - 2).as_bytes())?;
            }

            stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_bg)))?;
            stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_border)))?;
            stdout.write_all("│".as_bytes())?;
        }

        stdout.queue(MoveTo(start_x as u16, (start_y + dialog_h - 1) as u16))?;
        stdout.write_all(format!("╰{}╯", "─".repeat(dialog_w - 2)).as_bytes())?;

        Ok(())
    }

    fn draw_input_dialog<W: Write>(
        stdout: &mut W,
        title: &str,
        label: &str,
        input: &str,
        _cursor: usize,
        theme: &Theme,
        _i18n: &I18n,
        width: usize,
        height: usize,
    ) -> io::Result<()> {
        let dialog_w = 54.min(width.saturating_sub(4));
        let dialog_h = 8.min(height.saturating_sub(4));
        let start_x = (width.saturating_sub(dialog_w)) / 2;
        let start_y = (height.saturating_sub(dialog_h)) / 2;

        stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_bg)))?;
        stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_border)))?;

        stdout.queue(MoveTo(start_x as u16, start_y as u16))?;
        let title_str = format!(" 📄 {} ", title);
        let pad = dialog_w.saturating_sub(title_str.chars().count() + 2);
        stdout.write_all(format!("╭{}{}{}╮", title_str, "─".repeat(pad), "").as_bytes())?;

        for y_offset in 1..dialog_h - 1 {
            stdout.queue(MoveTo(start_x as u16, (start_y + y_offset) as u16))?;
            stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_bg)))?;
            stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_border)))?;
            stdout.write_all("│".as_bytes())?;

            match y_offset {
                1 => {
                    // Label line
                    stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_fg)))?;
                    let l_pad = (dialog_w - 2).saturating_sub(label.chars().count() + 2);
                    stdout.write_all(format!("  {}{}", label, " ".repeat(l_pad)).as_bytes())?;
                }
                2 => {
                    // Input Box line
                    stdout.write_all(b"  ")?;
                    stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_input_bg)))?;
                    stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_input_fg)))?;
                    let input_disp = format!(" {}", input);
                    let in_pad = (dialog_w - 6).saturating_sub(input_disp.chars().count());
                    stdout.write_all(format!("{}{}", input_disp, " ".repeat(in_pad)).as_bytes())?;
                    stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_bg)))?;
                    stdout.write_all(b"  ")?;
                }
                3 => {
                    stdout.write_all(" ".repeat(dialog_w - 2).as_bytes())?;
                }
                4 => {
                    // Buttons line
                    let btns = "  [ Enter: Confirm ]          [ Esc: Cancel ]";
                    let b_pad = (dialog_w - 2).saturating_sub(btns.chars().count());
                    stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_fg)))?;
                    stdout.write_all(format!("{}{}", btns, " ".repeat(b_pad)).as_bytes())?;
                }
                _ => {
                    stdout.write_all(" ".repeat(dialog_w - 2).as_bytes())?;
                }
            }

            stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_bg)))?;
            stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_border)))?;
            stdout.write_all("│".as_bytes())?;
        }

        stdout.queue(MoveTo(start_x as u16, (start_y + dialog_h - 1) as u16))?;
        stdout.write_all(format!("╰{}╯", "─".repeat(dialog_w - 2)).as_bytes())?;

        Ok(())
    }

    fn draw_confirm_dialog<W: Write>(
        stdout: &mut W,
        title: &str,
        message: &str,
        selected_btn: usize,
        theme: &Theme,
        _i18n: &I18n,
        width: usize,
        height: usize,
    ) -> io::Result<()> {
        let dialog_w = 56.min(width.saturating_sub(4));
        let dialog_h = 8.min(height.saturating_sub(4));
        let start_x = (width.saturating_sub(dialog_w)) / 2;
        let start_y = (height.saturating_sub(dialog_h)) / 2;

        stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_bg)))?;
        stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_border)))?;

        stdout.queue(MoveTo(start_x as u16, start_y as u16))?;
        let title_str = format!(" ⚠️ {} ", title);
        let pad = dialog_w.saturating_sub(title_str.chars().count() + 2);
        stdout.write_all(format!("╭{}{}{}╮", title_str, "─".repeat(pad), "").as_bytes())?;

        for y_offset in 1..dialog_h - 1 {
            stdout.queue(MoveTo(start_x as u16, (start_y + y_offset) as u16))?;
            stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_bg)))?;
            stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_border)))?;
            stdout.write_all("│".as_bytes())?;

            match y_offset {
                1 => {
                    stdout.write_all(" ".repeat(dialog_w - 2).as_bytes())?;
                }
                2 => {
                    stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_fg)))?;
                    let m_pad = (dialog_w - 2).saturating_sub(message.chars().count() + 4);
                    stdout.write_all(format!("    {}{}", message, " ".repeat(m_pad)).as_bytes())?;
                }
                3 => {
                    stdout.write_all(" ".repeat(dialog_w - 2).as_bytes())?;
                }
                4 => {
                    stdout.write_all(b"    ")?;
                    let btns = ["  Save  ", "  Don't Save  ", "  Cancel  "];
                    let mut used = 4;
                    for (b_i, b_text) in btns.iter().enumerate() {
                        if b_i == selected_btn {
                            stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_button_active_bg)))?;
                            stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_button_active_fg)))?;
                        } else {
                            stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_button_bg)))?;
                            stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_button_fg)))?;
                        }
                        stdout.write_all(format!("[{}]", b_text).as_bytes())?;
                        stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_bg)))?;
                        stdout.write_all(b"  ")?;
                        used += b_text.chars().count() + 4;
                    }
                    let pad = (dialog_w - 2).saturating_sub(used);
                    stdout.write_all(" ".repeat(pad).as_bytes())?;
                }
                _ => {
                    stdout.write_all(" ".repeat(dialog_w - 2).as_bytes())?;
                }
            }

            stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_bg)))?;
            stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_border)))?;
            stdout.write_all("│".as_bytes())?;
        }

        stdout.queue(MoveTo(start_x as u16, (start_y + dialog_h - 1) as u16))?;
        stdout.write_all(format!("╰{}╯", "─".repeat(dialog_w - 2)).as_bytes())?;

        Ok(())
    }

    fn draw_find_replace<W: Write>(
        stdout: &mut W,
        find_query: &str,
        replace_query: &str,
        match_case: bool,
        whole_word: bool,
        use_regex: bool,
        focused_field: &FindField,
        theme: &Theme,
        i18n: &I18n,
        width: usize,
        height: usize,
    ) -> io::Result<()> {
        let dialog_w = 64.min(width.saturating_sub(4));
        let dialog_h = 12.min(height.saturating_sub(4));
        let start_x = (width.saturating_sub(dialog_w)) / 2;
        let start_y = (height.saturating_sub(dialog_h)) / 2;

        stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_bg)))?;
        stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_border)))?;

        // Row 0: Top Border
        stdout.queue(MoveTo(start_x as u16, start_y as u16))?;
        let title_str = format!(" 🔎 {} ", i18n.t("FindReplaceTitle"));
        let pad = dialog_w.saturating_sub(title_str.chars().count() + 2);
        stdout.write_all(format!("╭{}{}{}╮", title_str, "─".repeat(pad), "").as_bytes())?;

        for y_offset in 1..dialog_h - 1 {
            stdout.queue(MoveTo(start_x as u16, (start_y + y_offset) as u16))?;
            stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_bg)))?;
            stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_border)))?;
            stdout.write_all("│".as_bytes())?;

            match y_offset {
                1 => {
                    // Spacer row
                    stdout.write_all(" ".repeat(dialog_w - 2).as_bytes())?;
                }
                2 => {
                    // Find Input Row
                    let label = "  Find:    ";
                    stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_fg)))?;
                    stdout.write_all(label.as_bytes())?;

                    let is_focus = *focused_field == FindField::FindInput;
                    if is_focus {
                        stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_button_active_bg)))?;
                        stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_button_active_fg)))?;
                    } else {
                        stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_input_bg)))?;
                        stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_input_fg)))?;
                    }
                    let f_disp = format!(" {}", find_query);
                    let input_w = (dialog_w - 2).saturating_sub(label.chars().count() + 2);
                    let f_pad = input_w.saturating_sub(f_disp.chars().count());
                    stdout.write_all(format!("{}{}", f_disp, " ".repeat(f_pad)).as_bytes())?;
                    stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_bg)))?;
                    stdout.write_all(b"  ")?;
                }
                3 => {
                    // Spacer row
                    stdout.write_all(" ".repeat(dialog_w - 2).as_bytes())?;
                }
                4 => {
                    // Replace Input Row
                    let label = "  Replace: ";
                    stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_fg)))?;
                    stdout.write_all(label.as_bytes())?;

                    let is_focus = *focused_field == FindField::ReplaceInput;
                    if is_focus {
                        stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_button_active_bg)))?;
                        stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_button_active_fg)))?;
                    } else {
                        stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_input_bg)))?;
                        stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_input_fg)))?;
                    }
                    let r_disp = format!(" {}", replace_query);
                    let input_w = (dialog_w - 2).saturating_sub(label.chars().count() + 2);
                    let r_pad = input_w.saturating_sub(r_disp.chars().count());
                    stdout.write_all(format!("{}{}", r_disp, " ".repeat(r_pad)).as_bytes())?;
                    stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_bg)))?;
                    stdout.write_all(b"  ")?;
                }
                5 => {
                    // Spacer row
                    stdout.write_all(" ".repeat(dialog_w - 2).as_bytes())?;
                }
                6 => {
                    // Options Row: Match Case, Whole Word, Regex
                    let c_box = if match_case { "[x] Match Case (Alt+C)" } else { "[ ] Match Case (Alt+C)" };
                    let w_box = if whole_word { "[x] Whole Word (Alt+W)" } else { "[ ] Whole Word (Alt+W)" };
                    let r_box = if use_regex { "[x] Regex" } else { "[ ] Regex" };

                    stdout.write_all(b"  ")?;
                    let items = [
                        (c_box, *focused_field == FindField::MatchCase),
                        (w_box, *focused_field == FindField::WholeWord),
                        (r_box, *focused_field == FindField::UseRegex),
                    ];
                    let mut used = 2;
                    for (text, is_foc) in items {
                        if is_foc {
                            stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_button_active_bg)))?;
                            stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_button_active_fg)))?;
                        } else {
                            stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_button_bg)))?;
                            stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_button_fg)))?;
                        }
                        stdout.write_all(format!(" {} ", text).as_bytes())?;
                        stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_bg)))?;
                        stdout.write_all(b"  ")?;
                        used += text.chars().count() + 4;
                    }
                    let pad = (dialog_w - 2).saturating_sub(used);
                    stdout.write_all(" ".repeat(pad).as_bytes())?;
                }
                7 => {
                    // Spacer row
                    stdout.write_all(" ".repeat(dialog_w - 2).as_bytes())?;
                }
                8 => {
                    // Shortcuts / Actions row
                    stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_fg)))?;
                    let action_text = "  [Enter] Next Match   [Alt+R] Replace   [Alt+A] All   [Esc]";
                    let pad = (dialog_w - 2).saturating_sub(action_text.chars().count());
                    stdout.write_all(format!("{}{}", action_text, " ".repeat(pad)).as_bytes())?;
                }
                _ => {
                    stdout.write_all(" ".repeat(dialog_w - 2).as_bytes())?;
                }
            }

            stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_bg)))?;
            stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_border)))?;
            stdout.write_all("│".as_bytes())?;
        }

        // Bottom Border
        stdout.queue(MoveTo(start_x as u16, (start_y + dialog_h - 1) as u16))?;
        stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_bg)))?;
        stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_border)))?;
        stdout.write_all(format!("╰{}╯", "─".repeat(dialog_w - 2)).as_bytes())?;

        Ok(())
    }

    fn draw_about_dialog<W: Write>(
        stdout: &mut W,
        theme: &Theme,
        _i18n: &I18n,
        width: usize,
        height: usize,
    ) -> io::Result<()> {
        let dialog_w = 54.min(width.saturating_sub(4));
        let lines = [
            "  edit-ng v0.1.0 (Rust Edition)",
            "  Next-Gen Modeless TUI Text Editor",
            "",
            "  • Powered by Tree-sitter AST Highlighting",
            "  • Sub-millisecond Fuzzy File Finder",
            "  • 11 Handcrafted Themes (.toml)",
            "  • 35+ Localized Languages (i18n)",
            "  • Extensible Plugin Architecture",
            "",
            "  [ Press Esc to Close ]",
        ];
        let dialog_h = (lines.len() + 2).min(height.saturating_sub(4));
        let start_x = (width.saturating_sub(dialog_w)) / 2;
        let start_y = (height.saturating_sub(dialog_h)) / 2;

        stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_bg)))?;
        stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_border)))?;

        stdout.queue(MoveTo(start_x as u16, start_y as u16))?;
        let title_str = " 🚀 About edit-ng ";
        let pad = dialog_w.saturating_sub(title_str.chars().count() + 2);
        stdout.write_all(format!("╭{}{}{}╮", title_str, "─".repeat(pad), "").as_bytes())?;

        for y_offset in 1..dialog_h - 1 {
            stdout.queue(MoveTo(start_x as u16, (start_y + y_offset) as u16))?;
            stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_bg)))?;
            stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_border)))?;
            stdout.write_all("│".as_bytes())?;

            let l_idx = y_offset - 1;
            if let Some(l) = lines.get(l_idx) {
                stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_fg)))?;
                let l_pad = (dialog_w - 2).saturating_sub(l.chars().count());
                stdout.write_all(format!("{}{}", l, " ".repeat(l_pad)).as_bytes())?;
            } else {
                stdout.write_all(" ".repeat(dialog_w - 2).as_bytes())?;
            }

            stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_bg)))?;
            stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_border)))?;
            stdout.write_all("│".as_bytes())?;
        }

        stdout.queue(MoveTo(start_x as u16, (start_y + dialog_h - 1) as u16))?;
        stdout.write_all(format!("╰{}╯", "─".repeat(dialog_w - 2)).as_bytes())?;
        Ok(())
    }

    fn draw_shortcuts_help<W: Write>(
        stdout: &mut W,
        theme: &Theme,
        _i18n: &I18n,
        width: usize,
        height: usize,
    ) -> io::Result<()> {
        let dialog_w = 58.min(width.saturating_sub(4));
        let shortcuts = [
            ("Ctrl+P / Ctrl+O", "Quick Open Fuzzy File Finder"),
            ("Ctrl+N / Ctrl+S", "New File / Save File"),
            ("Ctrl+W / Ctrl+Q", "Close Buffer / Exit Editor"),
            ("Ctrl+F", "Find & Replace"),
            ("Ctrl+G", "Go to Line Number"),
            ("Ctrl+Z / Ctrl+Y", "Undo / Redo"),
            ("Ctrl+A", "Select All"),
            ("Ctrl+D", "Duplicate Line"),
            ("Ctrl+Shift+K", "Delete Line"),
            ("Alt+Up / Alt+Down", "Move Line Up / Down"),
            ("F2", "Select Color Theme"),
            ("Ctrl+Shift+C", "Document Word Count Statistics"),
        ];
        let dialog_h = (shortcuts.len() + 2).min(height.saturating_sub(4));
        let start_x = (width.saturating_sub(dialog_w)) / 2;
        let start_y = (height.saturating_sub(dialog_h)) / 2;

        stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_bg)))?;
        stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_border)))?;

        stdout.queue(MoveTo(start_x as u16, start_y as u16))?;
        let title_str = " ⌨️ Keyboard Shortcuts ";
        let pad = dialog_w.saturating_sub(title_str.chars().count() + 2);
        stdout.write_all(format!("╭{}{}{}╮", title_str, "─".repeat(pad), "").as_bytes())?;

        for y_offset in 1..dialog_h - 1 {
            stdout.queue(MoveTo(start_x as u16, (start_y + y_offset) as u16))?;
            stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_bg)))?;
            stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_border)))?;
            stdout.write_all("│".as_bytes())?;

            let s_idx = y_offset - 1;
            if let Some((k, desc)) = shortcuts.get(s_idx) {
                stdout.queue(SetForegroundColor(parse_color(&theme.ui.menu_shortcut_fg)))?;
                stdout.write_all(format!("  {:<18}", k).as_bytes())?;
                stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_fg)))?;
                let d_str = format!(" {}", desc);
                let d_pad = (dialog_w - 22).saturating_sub(d_str.chars().count());
                stdout.write_all(format!("{}{}", d_str, " ".repeat(d_pad)).as_bytes())?;
            } else {
                stdout.write_all(" ".repeat(dialog_w - 2).as_bytes())?;
            }

            stdout.queue(SetBackgroundColor(parse_color(&theme.ui.dialog_bg)))?;
            stdout.queue(SetForegroundColor(parse_color(&theme.ui.dialog_border)))?;
            stdout.write_all("│".as_bytes())?;
        }

        stdout.queue(MoveTo(start_x as u16, (start_y + dialog_h - 1) as u16))?;
        stdout.write_all(format!("╰{}╯", "─".repeat(dialog_w - 2)).as_bytes())?;
        Ok(())
    }
}
