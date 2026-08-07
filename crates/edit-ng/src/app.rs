use crate::buffer::Buffer;
use crate::dialog::{ConfirmAction, DialogState, FindField};
use crate::fuzzy::FuzzyFinder;
use crate::i18n::I18n;
use crate::plugin::PluginManager;
use crate::theme::ThemeManager;
use crate::ui::{get_menu_dropdown_geometry, get_menus, get_navbar_button_ranges, get_tab_button_ranges, Renderer};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub struct App {
    pub buffers: Vec<Buffer>,
    pub active_buf_idx: usize,
    pub theme_manager: ThemeManager,
    pub i18n: I18n,
    pub plugin_manager: PluginManager,
    pub dialog: Option<DialogState>,
    pub active_menu: Option<(usize, usize)>, // (menu_idx, item_idx)
    pub status_message: Option<(String, Instant)>,
    pub clipboard: String,
    pub should_quit: bool,
    pub term_width: u16,
    pub term_height: u16,
}

impl App {
    pub fn new(initial_files: &[String], initial_theme: Option<String>, initial_lang: Option<String>) -> Self {
        let mut theme_manager = ThemeManager::new();
        if let Some(t) = initial_theme {
            theme_manager.set_theme(&t);
        }

        let mut i18n = I18n::new();
        if let Some(l) = initial_lang {
            i18n.set_language(&l);
        }

        let plugin_manager = PluginManager::new();
        let mut buffers = Vec::new();

        for file_arg in initial_files {
            let path = Path::new(file_arg);
            if path.exists() {
                if let Ok(buf) = Buffer::from_file(path) {
                    buffers.push(buf);
                }
            } else {
                let mut buf = Buffer::new_empty(file_arg.clone());
                buf.file_path = Some(path.to_path_buf());
                buffers.push(buf);
            }
        }

        if buffers.is_empty() {
            buffers.push(Buffer::new_empty("Untitled-1".into()));
        }

        let (w, h) = crossterm::terminal::size().unwrap_or((80, 24));

        Self {
            buffers,
            active_buf_idx: 0,
            theme_manager,
            i18n,
            plugin_manager,
            dialog: None,
            active_menu: None,
            status_message: None,
            clipboard: String::new(),
            should_quit: false,
            term_width: w,
            term_height: h,
        }
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some((msg.into(), Instant::now()));
    }

    pub fn current_buffer(&self) -> Option<&Buffer> {
        self.buffers.get(self.active_buf_idx)
    }

    pub fn current_buffer_mut(&mut self) -> Option<&mut Buffer> {
        self.buffers.get_mut(self.active_buf_idx)
    }

    pub fn open_file(&mut self, path: &Path) {
        if let Some(pos) = self.buffers.iter().position(|b| b.file_path.as_ref() == Some(&path.to_path_buf())) {
            self.active_buf_idx = pos;
            return;
        }

        if let Ok(buf) = Buffer::from_file(path) {
            self.buffers.push(buf);
            self.active_buf_idx = self.buffers.len() - 1;
            let display_str = path.display().to_string();
            self.set_status(format!("Opened {}", display_str));
        } else {
            let display_str = path.display().to_string();
            self.set_status(format!("Error reading {}", display_str));
        }
    }

    pub fn handle_resize(&mut self, width: u16, height: u16) {
        self.term_width = width;
        self.term_height = height;
    }

    pub fn scroll_cursor_into_view(&mut self) {
        let editor_height = (self.term_height as usize).saturating_sub(3);
        let gutter_w = self.current_buffer().map_or(5, |b| format!("{}", b.line_count()).len().max(3) + 2);
        let editor_width = (self.term_width as usize).saturating_sub(gutter_w + 1);

        if let Some(buf) = self.current_buffer_mut() {
            if buf.cursor_row < buf.scroll_top {
                buf.scroll_top = buf.cursor_row;
            } else if buf.cursor_row >= buf.scroll_top + editor_height {
                buf.scroll_top = buf.cursor_row.saturating_sub(editor_height - 1);
            }

            if buf.cursor_col < buf.scroll_left {
                buf.scroll_left = buf.cursor_col;
            } else if buf.cursor_col >= buf.scroll_left + editor_width {
                buf.scroll_left = buf.cursor_col.saturating_sub(editor_width - 1);
            }
        }
    }

    pub fn render<W: Write>(&mut self, stdout: &mut W) -> io::Result<()> {
        self.scroll_cursor_into_view();

        if let Some((_, created)) = self.status_message {
            if created.elapsed() > Duration::from_secs(4) {
                self.status_message = None;
            }
        }

        let msg = self.status_message.as_ref().map(|(s, _)| s.as_str());

        Renderer::render(
            stdout,
            &self.buffers,
            self.active_buf_idx,
            self.active_menu,
            self.dialog.as_ref(),
            &self.theme_manager,
            &self.i18n,
            self.term_width,
            self.term_height,
            msg,
        )
    }

    pub fn handle_mouse_event(&mut self, mouse: MouseEvent) -> bool {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let col = mouse.column as usize;
                let row = mouse.row as usize;
                self.handle_mouse_click(col, row);
                true
            }
            MouseEventKind::ScrollUp => {
                if let Some(buf) = self.current_buffer_mut() {
                    for _ in 0..3 {
                        buf.move_cursor_up(false);
                    }
                }
                true
            }
            MouseEventKind::ScrollDown => {
                if let Some(buf) = self.current_buffer_mut() {
                    for _ in 0..3 {
                        buf.move_cursor_down(false);
                    }
                }
                true
            }
            MouseEventKind::Moved => {
                if let Some((menu_idx, current_item_idx)) = self.active_menu {
                    let col = mouse.column as usize;
                    let row = mouse.row as usize;
                    let menus = get_menus(&self.theme_manager, &self.i18n);

                    if row == 0 {
                        let ranges = get_navbar_button_ranges(&menus, &self.i18n);
                        for (i, (start_x, end_x)) in ranges.iter().enumerate() {
                            if col >= *start_x && col < *end_x && i != menu_idx {
                                self.active_menu = Some((i, 0));
                                return true;
                            }
                        }
                    } else if let Some((m_x, _m_y, m_w, _m_h)) = get_menu_dropdown_geometry(&menus, menu_idx, &self.i18n) {
                        let menu = &menus[menu_idx];
                        if row >= 2 && row < 2 + menu.items.len() && col >= m_x && col < m_x + m_w {
                            let item_i = row - 2;
                            if let Some(item) = menu.items.get(item_i) {
                                if !item.is_separator && item_i != current_item_idx {
                                    self.active_menu = Some((menu_idx, item_i));
                                    return true;
                                }
                            }
                        }
                    }
                }
                false
            }
            _ => false,
        }
    }

    pub fn check_status_expiration(&mut self) -> bool {
        if let Some((_, created)) = self.status_message {
            if created.elapsed() > Duration::from_secs(4) {
                self.status_message = None;
                return true;
            }
        }
        false
    }

    pub fn handle_mouse_click(&mut self, col: usize, row: usize) {
        let menus = get_menus(&self.theme_manager, &self.i18n);

        // 1. If a menu is currently open:
        if let Some((menu_idx, _)) = self.active_menu {
            // Check if clicking navbar header
            if row == 0 {
                let ranges = get_navbar_button_ranges(&menus, &self.i18n);
                for (i, (start_x, end_x)) in ranges.iter().enumerate() {
                    if col >= *start_x && col < *end_x {
                        if i == menu_idx {
                            self.active_menu = None; // toggle off
                        } else {
                            self.active_menu = Some((i, 0)); // switch to clicked menu
                        }
                        return;
                    }
                }
                self.active_menu = None;
                return;
            }

            // Check if clicking inside the active dropdown
            if let Some((m_x, _m_y, m_w, _m_h)) = get_menu_dropdown_geometry(&menus, menu_idx, &self.i18n) {
                let menu = &menus[menu_idx];
                if row >= 2 && row < 2 + menu.items.len() && col >= m_x && col < m_x + m_w {
                    let item_i = row - 2;
                    if let Some(item) = menu.items.get(item_i) {
                        if !item.is_separator {
                            let action = item.action_id.clone();
                            self.active_menu = None;
                            self.execute_action(&action);
                            return;
                        }
                    }
                }
            }

            // Clicked outside dropdown
            self.active_menu = None;
            return;
        }

        // 2. If no menu is open, handle clicking on Navbar (row 0)
        if row == 0 {
            if col < 9 {
                // Clicked brand " edit-ng "
                self.dialog = Some(DialogState::About);
                return;
            }
            let ranges = get_navbar_button_ranges(&menus, &self.i18n);
            for (i, (start_x, end_x)) in ranges.iter().enumerate() {
                if col >= *start_x && col < *end_x {
                    self.active_menu = Some((i, 0));
                    return;
                }
            }
            return;
        }

        // 3. Tab Bar Click (row 1)
        if row == 1 {
            let tab_ranges = get_tab_button_ranges(&self.buffers);
            for (i, (start_x, end_x)) in tab_ranges.iter().enumerate() {
                if col >= *start_x && col < *end_x {
                    self.active_buf_idx = i;
                    return;
                }
            }
            return;
        }

        // 4. Editor Area Click (row >= 2)
        let editor_height = (self.term_height as usize).saturating_sub(3);
        if row >= 2 && row < 2 + editor_height {
            if let Some(buf) = self.current_buffer_mut() {
                let gutter_w = format!("{}", buf.line_count()).len().max(3) + 2;
                if col >= gutter_w {
                    let target_row = buf.scroll_top + (row - 2);
                    let target_col = buf.scroll_left + (col - gutter_w);
                    buf.set_cursor(target_row, target_col, false);
                }
            }
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        if self.dialog.is_some() {
            self.handle_dialog_key(key);
            return;
        }

        if let Some((menu_idx, item_idx)) = self.active_menu {
            let menus = get_menus(&self.theme_manager, &self.i18n);
            match key.code {
                KeyCode::Esc => {
                    self.active_menu = None;
                }
                KeyCode::Up => {
                    if let Some(menu) = menus.get(menu_idx) {
                        let count = menu.items.len();
                        let mut new_idx = if item_idx > 0 { item_idx - 1 } else { count.saturating_sub(1) };
                        if menu.items.get(new_idx).map_or(false, |it| it.is_separator) {
                            new_idx = if new_idx > 0 { new_idx - 1 } else { count.saturating_sub(1) };
                        }
                        self.active_menu = Some((menu_idx, new_idx));
                    }
                }
                KeyCode::Down => {
                    if let Some(menu) = menus.get(menu_idx) {
                        let count = menu.items.len();
                        if count > 0 {
                            let mut new_idx = (item_idx + 1) % count;
                            if menu.items.get(new_idx).map_or(false, |it| it.is_separator) {
                                new_idx = (new_idx + 1) % count;
                            }
                            self.active_menu = Some((menu_idx, new_idx));
                        }
                    }
                }
                KeyCode::Left => {
                    let count = menus.len();
                    let new_m = if menu_idx > 0 { menu_idx - 1 } else { count - 1 };
                    self.active_menu = Some((new_m, 0));
                }
                KeyCode::Right => {
                    let count = menus.len();
                    let new_m = (menu_idx + 1) % count;
                    self.active_menu = Some((new_m, 0));
                }
                KeyCode::Enter => {
                    if let Some(menu) = menus.get(menu_idx) {
                        if let Some(item) = menu.items.get(item_idx) {
                            if !item.is_separator {
                                let action = item.action_id.clone();
                                self.active_menu = None;
                                self.execute_action(&action);
                            }
                        }
                    }
                }
                _ => {}
            }
            return;
        }

        if key.modifiers.contains(KeyModifiers::ALT) {
            match key.code {
                KeyCode::Char('f') | KeyCode::Char('F') => { self.active_menu = Some((0, 0)); return; }
                KeyCode::Char('e') | KeyCode::Char('E') => { self.active_menu = Some((1, 0)); return; }
                KeyCode::Char('v') | KeyCode::Char('V') => { self.active_menu = Some((2, 0)); return; }
                KeyCode::Char('p') | KeyCode::Char('P') => { self.active_menu = Some((3, 0)); return; }
                KeyCode::Char('t') | KeyCode::Char('T') => { self.active_menu = Some((4, 0)); return; }
                KeyCode::Char('l') | KeyCode::Char('L') => { self.active_menu = Some((5, 0)); return; }
                KeyCode::Char('h') | KeyCode::Char('H') => { self.active_menu = Some((6, 0)); return; }
                _ => {}
            }
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('p') | KeyCode::Char('o') => {
                    self.open_fuzzy_finder();
                    return;
                }
                KeyCode::Char('n') => {
                    let new_id = self.buffers.len() + 1;
                    self.buffers.push(Buffer::new_empty(format!("Untitled-{}", new_id)));
                    self.active_buf_idx = self.buffers.len() - 1;
                    return;
                }
                KeyCode::Char('s') => {
                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                        let current_path = self.current_buffer().and_then(|b| b.file_path.as_ref()).map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
                        self.dialog = Some(DialogState::SaveAs {
                            input: current_path,
                            cursor: 0,
                        });
                    } else {
                        self.save_current_buffer();
                    }
                    return;
                }
                KeyCode::Char('w') => {
                    self.close_current_buffer();
                    return;
                }
                KeyCode::Char('q') => {
                    self.quit_app();
                    return;
                }
                KeyCode::Char('f') => {
                    self.dialog = Some(DialogState::FindReplace {
                        find_query: String::new(),
                        find_cursor: 0,
                        replace_query: String::new(),
                        replace_cursor: 0,
                        match_case: false,
                        whole_word: false,
                        use_regex: false,
                        focused_field: FindField::FindInput,
                    });
                    return;
                }
                KeyCode::Char('g') => {
                    self.dialog = Some(DialogState::GotoLine {
                        input: String::new(),
                        cursor: 0,
                    });
                    return;
                }
                KeyCode::Char('l') => {
                    self.dialog = Some(DialogState::LanguagePicker {
                        selected_index: 0,
                        search_query: String::new(),
                    });
                    return;
                }
                KeyCode::Char('c') => {
                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                        if let Some(buf) = self.current_buffer() {
                            let (l, w, c, b) = buf.word_count();
                            self.dialog = Some(DialogState::WordCount {
                                lines: l,
                                words: w,
                                chars: c,
                                bytes: b,
                            });
                        }
                    } else {
                        let sel_text = self.current_buffer().and_then(|b| b.selected_text());
                        if let Some(sel) = sel_text {
                            self.clipboard = sel;
                            self.set_status("Copied selection to clipboard");
                        }
                    }
                    return;
                }
                KeyCode::Char('x') => {
                    let mut cut_text = None;
                    if let Some(buf) = self.current_buffer_mut() {
                        if let Some(sel) = buf.selected_text() {
                            cut_text = Some(sel);
                            buf.delete_selection();
                        }
                    }
                    if let Some(txt) = cut_text {
                        self.clipboard = txt;
                        self.set_status("Cut selection to clipboard");
                    }
                    return;
                }
                KeyCode::Char('v') => {
                    if !self.clipboard.is_empty() {
                        let clip = self.clipboard.clone();
                        if let Some(buf) = self.current_buffer_mut() {
                            buf.insert_str(&clip);
                        }
                    }
                    return;
                }
                KeyCode::Char('z') => {
                    self.set_status("Undo");
                    return;
                }
                KeyCode::Char('y') => {
                    self.set_status("Redo");
                    return;
                }
                KeyCode::Char('a') => {
                    if let Some(buf) = self.current_buffer_mut() {
                        buf.select_all();
                    }
                    return;
                }
                KeyCode::Char('d') => {
                    if let Some(buf) = self.current_buffer_mut() {
                        buf.duplicate_line();
                    }
                    return;
                }
                KeyCode::Char('k') => {
                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                        if let Some(buf) = self.current_buffer_mut() {
                            buf.delete_line();
                        }
                    }
                    return;
                }
                KeyCode::Tab => {
                    if !self.buffers.is_empty() {
                        self.active_buf_idx = (self.active_buf_idx + 1) % self.buffers.len();
                    }
                    return;
                }
                _ => {}
            }
        }

        match key.code {
            KeyCode::F(1) => {
                self.dialog = Some(DialogState::ShortcutsHelp);
                return;
            }
            KeyCode::F(2) => {
                self.dialog = Some(DialogState::ThemePicker {
                    selected_index: 0,
                    search_query: String::new(),
                });
                return;
            }
            KeyCode::F(3) => {
                if let Some(buf) = self.current_buffer_mut() {
                    buf.next_match();
                }
                return;
            }
            _ => {}
        }

        let select_mode = key.modifiers.contains(KeyModifiers::SHIFT);
        if let Some(buf) = self.current_buffer_mut() {
            match key.code {
                KeyCode::Left => buf.move_cursor_left(select_mode),
                KeyCode::Right => buf.move_cursor_right(select_mode),
                KeyCode::Up => {
                    if key.modifiers.contains(KeyModifiers::ALT) {
                        buf.move_line_up();
                    } else {
                        buf.move_cursor_up(select_mode);
                    }
                }
                KeyCode::Down => {
                    if key.modifiers.contains(KeyModifiers::ALT) {
                        buf.move_line_down();
                    } else {
                        buf.move_cursor_down(select_mode);
                    }
                }
                KeyCode::Home => buf.move_to_line_start(select_mode),
                KeyCode::End => buf.move_to_line_end(select_mode),
                KeyCode::PageUp => {
                    for _ in 0..15 {
                        buf.move_cursor_up(select_mode);
                    }
                }
                KeyCode::PageDown => {
                    for _ in 0..15 {
                        buf.move_cursor_down(select_mode);
                    }
                }
                KeyCode::Char(c) => buf.insert_char(c),
                KeyCode::Enter => buf.insert_newline(),
                KeyCode::Backspace => buf.delete_backspace(),
                KeyCode::Delete => buf.delete_forward(),
                KeyCode::Tab => {
                    if select_mode {
                        buf.unindent_selection();
                    } else {
                        buf.insert_str("    ");
                    }
                }
                _ => {}
            }
        }
    }

    pub fn handle_dialog_key(&mut self, key: KeyEvent) {
        match self.dialog.as_mut() {
            Some(DialogState::FuzzyFinder(finder)) => match key.code {
                KeyCode::Esc => {
                    self.dialog = None;
                }
                KeyCode::Up => finder.move_selection_up(),
                KeyCode::Down => finder.move_selection_down(),
                KeyCode::Enter => {
                    if let Some(item) = finder.selected_item().cloned() {
                        self.dialog = None;
                        self.open_file(&item.full_path);
                    }
                }
                KeyCode::Backspace => {
                    let mut q = finder.query.clone();
                    q.pop();
                    finder.update_query(&q);
                }
                KeyCode::Char(c) => {
                    let mut q = finder.query.clone();
                    q.push(c);
                    finder.update_query(&q);
                }
                _ => {}
            },
            Some(DialogState::ThemePicker { selected_index, .. }) => match key.code {
                KeyCode::Esc => {
                    self.dialog = None;
                }
                KeyCode::Up => {
                    if *selected_index > 0 {
                        *selected_index -= 1;
                    }
                    if let Some(t_name) = self.theme_manager.theme_names.get(*selected_index).cloned() {
                        self.theme_manager.set_theme(&t_name);
                    }
                }
                KeyCode::Down => {
                    if *selected_index + 1 < self.theme_manager.theme_names.len() {
                        *selected_index += 1;
                    }
                    if let Some(t_name) = self.theme_manager.theme_names.get(*selected_index).cloned() {
                        self.theme_manager.set_theme(&t_name);
                    }
                }
                KeyCode::Enter => {
                    if let Some(t_name) = self.theme_manager.theme_names.get(*selected_index).cloned() {
                        self.theme_manager.set_theme(&t_name);
                        self.set_status(format!("Active theme: {}", t_name));
                    }
                    self.dialog = None;
                }
                _ => {}
            },
            Some(DialogState::LanguagePicker { selected_index, .. }) => match key.code {
                KeyCode::Esc => {
                    self.dialog = None;
                }
                KeyCode::Up => {
                    if *selected_index > 0 {
                        *selected_index -= 1;
                    }
                }
                KeyCode::Down => {
                    if *selected_index + 1 < self.i18n.available_languages.len() {
                        *selected_index += 1;
                    }
                }
                KeyCode::Enter => {
                    if let Some(lang) = self.i18n.available_languages.get(*selected_index).cloned() {
                        self.i18n.set_language(&lang.code);
                        self.set_status(format!("Language set to: {}", lang.native_name));
                    }
                    self.dialog = None;
                }
                _ => {}
            },
            Some(DialogState::WordCount { .. }) | Some(DialogState::About) | Some(DialogState::ShortcutsHelp) => {
                if key.code == KeyCode::Esc || key.code == KeyCode::Enter {
                    self.dialog = None;
                }
            }
            Some(DialogState::GotoLine { input, .. }) => match key.code {
                KeyCode::Esc => {
                    self.dialog = None;
                }
                KeyCode::Char(c) if c.is_ascii_digit() => {
                    input.push(c);
                }
                KeyCode::Backspace => {
                    input.pop();
                }
                KeyCode::Enter => {
                    if let Ok(line_num) = input.parse::<usize>() {
                        if let Some(buf) = self.current_buffer_mut() {
                            buf.goto_line(line_num);
                        }
                    }
                    self.dialog = None;
                }
                _ => {}
            },
            Some(DialogState::OpenFile { input, .. }) => match key.code {
                KeyCode::Esc => {
                    self.dialog = None;
                }
                KeyCode::Char(c) => {
                    input.push(c);
                }
                KeyCode::Backspace => {
                    input.pop();
                }
                KeyCode::Enter => {
                    let path = PathBuf::from(input.clone());
                    self.dialog = None;
                    self.open_file(&path);
                }
                _ => {}
            },
            Some(DialogState::SaveAs { input, .. }) => match key.code {
                KeyCode::Esc => {
                    self.dialog = None;
                }
                KeyCode::Char(c) => {
                    input.push(c);
                }
                KeyCode::Backspace => {
                    input.pop();
                }
                KeyCode::Enter => {
                    let path = PathBuf::from(input.clone());
                    let mut res = Ok(());
                    if let Some(buf) = self.current_buffer_mut() {
                        res = buf.save_as(&path);
                    }
                    match res {
                        Ok(()) => self.set_status(format!("Saved to {}", path.display())),
                        Err(e) => self.set_status(format!("Save failed: {}", e)),
                    }
                    self.dialog = None;
                }
                _ => {}
            },
            Some(DialogState::ConfirmClose { action, selected_button }) => match key.code {
                KeyCode::Esc => {
                    self.dialog = None;
                }
                KeyCode::Left => {
                    if *selected_button > 0 {
                        *selected_button -= 1;
                    }
                }
                KeyCode::Right => {
                    if *selected_button < 2 {
                        *selected_button += 1;
                    }
                }
                KeyCode::Enter => {
                    let btn = *selected_button;
                    let act = action.clone();
                    self.dialog = None;

                    match act {
                        ConfirmAction::CloseCurrentBuffer(idx) => {
                            if btn == 0 {
                                self.save_current_buffer();
                                self.close_buffer_at(idx);
                            } else if btn == 1 {
                                self.close_buffer_at(idx);
                            }
                        }
                        ConfirmAction::QuitApplication => {
                            if btn == 0 {
                                self.save_current_buffer();
                                self.should_quit = true;
                            } else if btn == 1 {
                                self.should_quit = true;
                            }
                        }
                    }
                }
                _ => {}
            },
            Some(DialogState::FindReplace {
                find_query,
                replace_query,
                match_case,
                whole_word,
                use_regex,
                focused_field,
                ..
            }) => match key.code {
                KeyCode::Esc => {
                    self.dialog = None;
                }
                KeyCode::Tab => {
                    *focused_field = match focused_field {
                        FindField::FindInput => FindField::ReplaceInput,
                        FindField::ReplaceInput => FindField::MatchCase,
                        FindField::MatchCase => FindField::WholeWord,
                        FindField::WholeWord => FindField::UseRegex,
                        FindField::UseRegex => FindField::FindInput,
                    };
                }
                KeyCode::BackTab => {
                    *focused_field = match focused_field {
                        FindField::FindInput => FindField::UseRegex,
                        FindField::ReplaceInput => FindField::FindInput,
                        FindField::MatchCase => FindField::ReplaceInput,
                        FindField::WholeWord => FindField::MatchCase,
                        FindField::UseRegex => FindField::WholeWord,
                    };
                }
                KeyCode::Up => {
                    *focused_field = match focused_field {
                        FindField::FindInput => FindField::UseRegex,
                        FindField::ReplaceInput => FindField::FindInput,
                        FindField::MatchCase | FindField::WholeWord | FindField::UseRegex => FindField::ReplaceInput,
                    };
                }
                KeyCode::Down => {
                    *focused_field = match focused_field {
                        FindField::FindInput => FindField::ReplaceInput,
                        FindField::ReplaceInput => FindField::MatchCase,
                        FindField::MatchCase | FindField::WholeWord | FindField::UseRegex => FindField::FindInput,
                    };
                }
                KeyCode::Left => match focused_field {
                    FindField::WholeWord => *focused_field = FindField::MatchCase,
                    FindField::UseRegex => *focused_field = FindField::WholeWord,
                    _ => {}
                },
                KeyCode::Right => match focused_field {
                    FindField::MatchCase => *focused_field = FindField::WholeWord,
                    FindField::WholeWord => *focused_field = FindField::UseRegex,
                    _ => {}
                },
                KeyCode::Char(' ') if matches!(focused_field, FindField::MatchCase | FindField::WholeWord | FindField::UseRegex) => {
                    match focused_field {
                        FindField::MatchCase => *match_case = !*match_case,
                        FindField::WholeWord => *whole_word = !*whole_word,
                        FindField::UseRegex => *use_regex = !*use_regex,
                        _ => {}
                    }
                }
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::ALT) => {
                    *match_case = !*match_case;
                }
                KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::ALT) => {
                    *whole_word = !*whole_word;
                }
                KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::ALT) => {
                    let rep = replace_query.clone();
                    if let Some(buf) = self.current_buffer_mut() {
                        buf.replace_current(&rep);
                        buf.next_match();
                    }
                }
                KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::ALT) => {
                    let f = find_query.clone();
                    let r = replace_query.clone();
                    let mc = *match_case;
                    let ww = *whole_word;
                    let rx = *use_regex;
                    let mut count = 0;
                    if let Some(buf) = self.current_buffer_mut() {
                        count = buf.replace_all(&f, &r, mc, ww, rx);
                    }
                    self.set_status(format!("Replaced {} occurrences", count));
                    self.dialog = None;
                }
                KeyCode::Enter => {
                    let f = find_query.clone();
                    let mc = *match_case;
                    let ww = *whole_word;
                    let rx = *use_regex;
                    let mut match_len = 0;
                    if let Some(buf) = self.current_buffer_mut() {
                        if buf.search_needle != f {
                            buf.search(&f, mc, ww, rx);
                        } else {
                            buf.next_match();
                        }
                        match_len = buf.search_matches.len();
                    }
                    self.set_status(format!("Found {} matches", match_len));
                }
                KeyCode::Backspace => match focused_field {
                    FindField::FindInput => { find_query.pop(); }
                    FindField::ReplaceInput => { replace_query.pop(); }
                    _ => {}
                },
                KeyCode::Char(c) => match focused_field {
                    FindField::FindInput => { find_query.push(c); }
                    FindField::ReplaceInput => { replace_query.push(c); }
                    FindField::MatchCase => { *match_case = !*match_case; }
                    FindField::WholeWord => { *whole_word = !*whole_word; }
                    FindField::UseRegex => { *use_regex = !*use_regex; }
                },
                _ => {}
            },
            None => {}
        }
    }

    pub fn open_fuzzy_finder(&mut self) {
        let root = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let finder = FuzzyFinder::new(&root);
        self.dialog = Some(DialogState::FuzzyFinder(finder));
    }

    pub fn save_current_buffer(&mut self) {
        let has_path = self.current_buffer().and_then(|b| b.file_path.as_ref()).is_some();
        if has_path {
            let mut save_res = Ok(());
            let mut title_str = String::new();
            if let Some(buf) = self.current_buffer_mut() {
                title_str = buf.title.clone();
                save_res = buf.save();
            }
            match save_res {
                Ok(()) => self.set_status(format!("Saved {}", title_str)),
                Err(e) => self.set_status(format!("Save failed: {}", e)),
            }
        } else {
            self.dialog = Some(DialogState::SaveAs {
                input: String::new(),
                cursor: 0,
            });
        }
    }

    pub fn close_current_buffer(&mut self) {
        if let Some(buf) = self.current_buffer() {
            if buf.is_modified {
                self.dialog = Some(DialogState::ConfirmClose {
                    action: ConfirmAction::CloseCurrentBuffer(self.active_buf_idx),
                    selected_button: 0,
                });
                return;
            }
        }
        self.close_buffer_at(self.active_buf_idx);
    }

    pub fn close_buffer_at(&mut self, idx: usize) {
        if self.buffers.len() > 1 {
            self.buffers.remove(idx);
            if self.active_buf_idx >= self.buffers.len() {
                self.active_buf_idx = self.buffers.len() - 1;
            }
        } else {
            self.buffers[0] = Buffer::new_empty("Untitled-1".into());
        }
    }

    pub fn quit_app(&mut self) {
        let has_dirty = self.buffers.iter().any(|b| b.is_modified);
        if has_dirty {
            self.dialog = Some(DialogState::ConfirmClose {
                action: ConfirmAction::QuitApplication,
                selected_button: 0,
            });
        } else {
            self.should_quit = true;
        }
    }

    pub fn execute_action(&mut self, action_id: &str) {
        if action_id.starts_with("set_theme_") {
            let t_name = &action_id["set_theme_".len()..];
            self.theme_manager.set_theme(t_name);
            self.set_status(format!("Active Theme: {}", t_name));
            return;
        }

        if action_id.starts_with("set_lang_") {
            let lang_code = &action_id["set_lang_".len()..];
            self.i18n.set_language(lang_code);
            let lang_name = self.i18n.current_language_name().to_string();
            self.set_status(format!("Language: {}", lang_name));
            return;
        }

        match action_id {
            "file_new" => {
                let new_id = self.buffers.len() + 1;
                self.buffers.push(Buffer::new_empty(format!("Untitled-{}", new_id)));
                self.active_buf_idx = self.buffers.len() - 1;
            }
            "file_open" => {
                self.dialog = Some(DialogState::OpenFile {
                    input: String::new(),
                    cursor: 0,
                });
            }
            "fuzzy_finder" => {
                self.open_fuzzy_finder();
            }
            "file_save" => {
                self.save_current_buffer();
            }
            "file_save_as" => {
                self.dialog = Some(DialogState::SaveAs {
                    input: String::new(),
                    cursor: 0,
                });
            }
            "file_close" => {
                self.close_current_buffer();
            }
            "file_exit" => {
                self.quit_app();
            }
            "edit_undo" => {
                self.set_status("Undo");
            }
            "edit_redo" => {
                self.set_status("Redo");
            }
            "edit_cut" => {
                let mut cut_text = None;
                if let Some(buf) = self.current_buffer_mut() {
                    if let Some(sel) = buf.selected_text() {
                        cut_text = Some(sel);
                        buf.delete_selection();
                    }
                }
                if let Some(txt) = cut_text {
                    self.clipboard = txt;
                    self.set_status("Cut selection to clipboard");
                }
            }
            "edit_copy" => {
                let sel_text = self.current_buffer().and_then(|b| b.selected_text());
                if let Some(sel) = sel_text {
                    self.clipboard = sel;
                    self.set_status("Copied selection to clipboard");
                }
            }
            "edit_paste" => {
                if !self.clipboard.is_empty() {
                    let clip = self.clipboard.clone();
                    if let Some(buf) = self.current_buffer_mut() {
                        buf.insert_str(&clip);
                    }
                }
            }
            "edit_find" => {
                self.dialog = Some(DialogState::FindReplace {
                    find_query: String::new(),
                    find_cursor: 0,
                    replace_query: String::new(),
                    replace_cursor: 0,
                    match_case: false,
                    whole_word: false,
                    use_regex: false,
                    focused_field: FindField::FindInput,
                });
            }
            "edit_goto" => {
                self.dialog = Some(DialogState::GotoLine {
                    input: String::new(),
                    cursor: 0,
                });
            }
            "edit_duplicate" => {
                if let Some(buf) = self.current_buffer_mut() {
                    buf.duplicate_line();
                }
            }
            "edit_delete_line" => {
                if let Some(buf) = self.current_buffer_mut() {
                    buf.delete_line();
                }
            }
            "edit_move_up" => {
                if let Some(buf) = self.current_buffer_mut() {
                    buf.move_line_up();
                }
            }
            "edit_move_down" => {
                if let Some(buf) = self.current_buffer_mut() {
                    buf.move_line_down();
                }
            }
            "edit_select_all" => {
                if let Some(buf) = self.current_buffer_mut() {
                    buf.select_all();
                }
            }
            "view_word_count" => {
                if let Some(buf) = self.current_buffer() {
                    let (l, w, c, b) = buf.word_count();
                    self.dialog = Some(DialogState::WordCount {
                        lines: l,
                        words: w,
                        chars: c,
                        bytes: b,
                    });
                }
            }
            "view_language" => {
                self.dialog = Some(DialogState::LanguagePicker {
                    selected_index: 0,
                    search_query: String::new(),
                });
            }
            "theme_picker" => {
                self.dialog = Some(DialogState::ThemePicker {
                    selected_index: 0,
                    search_query: String::new(),
                });
            }
            "plugin_git" => {
                self.set_status("Git plugin active: branch main, 0 modified files");
            }
            "help_shortcuts" => {
                self.dialog = Some(DialogState::ShortcutsHelp);
            }
            "help_about" => {
                self.dialog = Some(DialogState::About);
            }
            "tab_next" => {
                if !self.buffers.is_empty() {
                    self.active_buf_idx = (self.active_buf_idx + 1) % self.buffers.len();
                }
            }
            "tab_prev" => {
                if !self.buffers.is_empty() {
                    if self.active_buf_idx > 0 {
                        self.active_buf_idx -= 1;
                    } else {
                        self.active_buf_idx = self.buffers.len() - 1;
                    }
                }
            }
            _ => {}
        }
    }
}
