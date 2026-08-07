use crate::syntax::{Language, SyntaxHighlighter};
use regex::Regex;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineEnding {
    Lf,
    Crlf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndentType {
    Spaces(usize),
    Tabs,
}

#[derive(Debug, Clone)]
pub struct SearchMatch {
    pub row: usize,
    pub start_col: usize,
    pub end_col: usize,
}

#[derive(Debug, Clone)]
pub enum UndoAction {
    InsertText {
        row: usize,
        col: usize,
        text: String,
    },
    DeleteText {
        row: usize,
        col: usize,
        text: String,
    },
    Group(Vec<UndoAction>),
}

pub struct Buffer {
    pub lines: Vec<String>,
    pub file_path: Option<PathBuf>,
    pub title: String,
    pub is_modified: bool,
    pub read_only: bool,

    // Cursor position: (row, col)
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub preferred_col: usize,

    // Selection anchor: (row, col)
    pub selection_anchor: Option<(usize, usize)>,

    // Viewport scrolling
    pub scroll_top: usize,
    pub scroll_left: usize,

    // File settings
    pub line_ending: LineEnding,
    pub indent_type: IndentType,
    pub syntax_highlighter: SyntaxHighlighter,

    // Undo / Redo
    pub undo_stack: Vec<UndoAction>,
    pub redo_stack: Vec<UndoAction>,

    // Search state
    pub search_needle: String,
    pub search_matches: Vec<SearchMatch>,
    pub active_match_index: usize,
}

impl Buffer {
    pub fn new_empty(title: String) -> Self {
        let mut buffer = Self {
            lines: vec![String::new()],
            file_path: None,
            title,
            is_modified: false,
            read_only: false,
            cursor_row: 0,
            cursor_col: 0,
            preferred_col: 0,
            selection_anchor: None,
            scroll_top: 0,
            scroll_left: 0,
            line_ending: LineEnding::Lf,
            indent_type: IndentType::Spaces(4),
            syntax_highlighter: SyntaxHighlighter::new(Language::PlainText),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            search_needle: String::new(),
            search_matches: Vec::new(),
            active_match_index: 0,
        };
        buffer.refresh_syntax();
        buffer
    }

    pub fn from_file(path: &Path) -> io::Result<Self> {
        let content = fs::read_to_string(path)?;
        let line_ending = if content.contains("\r\n") {
            LineEnding::Crlf
        } else {
            LineEnding::Lf
        };

        let raw_lines: Vec<&str> = content.split('\n').collect();
        let mut lines = Vec::new();
        for l in raw_lines {
            let clean = l.trim_end_matches('\r').to_string();
            lines.push(clean);
        }
        if lines.is_empty() {
            lines.push(String::new());
        }

        let title = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".into());

        let lang = Language::from_path(path);

        let mut buffer = Self {
            lines,
            file_path: Some(path.to_path_buf()),
            title,
            is_modified: false,
            read_only: false,
            cursor_row: 0,
            cursor_col: 0,
            preferred_col: 0,
            selection_anchor: None,
            scroll_top: 0,
            scroll_left: 0,
            line_ending,
            indent_type: IndentType::Spaces(4),
            syntax_highlighter: SyntaxHighlighter::new(lang),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            search_needle: String::new(),
            search_matches: Vec::new(),
            active_match_index: 0,
        };

        buffer.refresh_syntax();
        Ok(buffer)
    }

    pub fn save(&mut self) -> io::Result<()> {
        if let Some(path) = self.file_path.clone() {
            self.save_as(&path)
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "No file path set"))
        }
    }

    pub fn save_as(&mut self, path: &Path) -> io::Result<()> {
        let separator = match self.line_ending {
            LineEnding::Lf => "\n",
            LineEnding::Crlf => "\r\n",
        };

        let mut content = self.lines.join(separator);
        content.push_str(separator);

        fs::write(path, content)?;
        self.file_path = Some(path.to_path_buf());
        self.title = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".into());
        self.is_modified = false;
        self.syntax_highlighter.set_language(Language::from_path(path));
        self.refresh_syntax();
        Ok(())
    }

    pub fn refresh_syntax(&mut self) {
        let full_text = self.lines.join("\n");
        self.syntax_highlighter.update_tree(&full_text);
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn current_line(&self) -> &str {
        self.lines.get(self.cursor_row).map(|s| s.as_str()).unwrap_or("")
    }

    pub fn line_len(&self, row: usize) -> usize {
        self.lines.get(row).map(|l| l.chars().count()).unwrap_or(0)
    }

    // --- Cursor & Selection ---

    pub fn clamp_cursor(&mut self) {
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        if self.cursor_row >= self.lines.len() {
            self.cursor_row = self.lines.len() - 1;
        }
        let line_len = self.line_len(self.cursor_row);
        if self.cursor_col > line_len {
            self.cursor_col = line_len;
        }
    }

    pub fn set_cursor(&mut self, row: usize, col: usize, keep_selection: bool) {
        if !keep_selection {
            self.selection_anchor = None;
        } else if self.selection_anchor.is_none() {
            self.selection_anchor = Some((self.cursor_row, self.cursor_col));
        }

        self.cursor_row = row;
        self.cursor_col = col;
        self.clamp_cursor();
        self.preferred_col = self.cursor_col;
    }

    pub fn move_cursor_left(&mut self, select: bool) {
        if select && self.selection_anchor.is_none() {
            self.selection_anchor = Some((self.cursor_row, self.cursor_col));
        } else if !select {
            self.selection_anchor = None;
        }

        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.line_len(self.cursor_row);
        }
        self.preferred_col = self.cursor_col;
    }

    pub fn move_cursor_right(&mut self, select: bool) {
        if select && self.selection_anchor.is_none() {
            self.selection_anchor = Some((self.cursor_row, self.cursor_col));
        } else if !select {
            self.selection_anchor = None;
        }

        let cur_len = self.line_len(self.cursor_row);
        if self.cursor_col < cur_len {
            self.cursor_col += 1;
        } else if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            self.cursor_col = 0;
        }
        self.preferred_col = self.cursor_col;
    }

    pub fn move_cursor_up(&mut self, select: bool) {
        if select && self.selection_anchor.is_none() {
            self.selection_anchor = Some((self.cursor_row, self.cursor_col));
        } else if !select {
            self.selection_anchor = None;
        }

        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            let target_len = self.line_len(self.cursor_row);
            self.cursor_col = self.preferred_col.min(target_len);
        } else {
            self.cursor_col = 0;
            self.preferred_col = 0;
        }
    }

    pub fn move_cursor_down(&mut self, select: bool) {
        if select && self.selection_anchor.is_none() {
            self.selection_anchor = Some((self.cursor_row, self.cursor_col));
        } else if !select {
            self.selection_anchor = None;
        }

        if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            let target_len = self.line_len(self.cursor_row);
            self.cursor_col = self.preferred_col.min(target_len);
        } else {
            let target_len = self.line_len(self.cursor_row);
            self.cursor_col = target_len;
            self.preferred_col = target_len;
        }
    }

    pub fn move_to_line_start(&mut self, select: bool) {
        if select && self.selection_anchor.is_none() {
            self.selection_anchor = Some((self.cursor_row, self.cursor_col));
        } else if !select {
            self.selection_anchor = None;
        }
        let line = self.current_line();
        let first_non_space = line.chars().position(|c| !c.is_whitespace()).unwrap_or(0);
        if self.cursor_col == first_non_space {
            self.cursor_col = 0;
        } else {
            self.cursor_col = first_non_space;
        }
        self.preferred_col = self.cursor_col;
    }

    pub fn move_to_line_end(&mut self, select: bool) {
        if select && self.selection_anchor.is_none() {
            self.selection_anchor = Some((self.cursor_row, self.cursor_col));
        } else if !select {
            self.selection_anchor = None;
        }
        self.cursor_col = self.line_len(self.cursor_row);
        self.preferred_col = self.cursor_col;
    }

    pub fn move_to_top(&mut self, select: bool) {
        if select && self.selection_anchor.is_none() {
            self.selection_anchor = Some((self.cursor_row, self.cursor_col));
        } else if !select {
            self.selection_anchor = None;
        }
        self.cursor_row = 0;
        self.cursor_col = 0;
        self.preferred_col = 0;
    }

    pub fn move_to_bottom(&mut self, select: bool) {
        if select && self.selection_anchor.is_none() {
            self.selection_anchor = Some((self.cursor_row, self.cursor_col));
        } else if !select {
            self.selection_anchor = None;
        }
        self.cursor_row = self.lines.len().saturating_sub(1);
        self.cursor_col = self.line_len(self.cursor_row);
        self.preferred_col = self.cursor_col;
    }

    pub fn select_all(&mut self) {
        self.selection_anchor = Some((0, 0));
        self.cursor_row = self.lines.len().saturating_sub(1);
        self.cursor_col = self.line_len(self.cursor_row);
        self.preferred_col = self.cursor_col;
    }

    pub fn selection_range(&self) -> Option<((usize, usize), (usize, usize))> {
        if let Some((anchor_row, anchor_col)) = self.selection_anchor {
            if anchor_row == self.cursor_row && anchor_col == self.cursor_col {
                return None;
            }
            if (anchor_row, anchor_col) < (self.cursor_row, self.cursor_col) {
                Some(((anchor_row, anchor_col), (self.cursor_row, self.cursor_col)))
            } else {
                Some(((self.cursor_row, self.cursor_col), (anchor_row, anchor_col)))
            }
        } else {
            None
        }
    }

    pub fn selected_text(&self) -> Option<String> {
        let (start, end) = self.selection_range()?;
        let mut result = String::new();

        if start.0 == end.0 {
            let line = self.lines.get(start.0)?;
            let chars: Vec<char> = line.chars().collect();
            let slice: String = chars[start.1.min(chars.len())..end.1.min(chars.len())].iter().collect();
            return Some(slice);
        }

        for r in start.0..=end.0 {
            if let Some(line) = self.lines.get(r) {
                let chars: Vec<char> = line.chars().collect();
                if r == start.0 {
                    let slice: String = chars[start.1.min(chars.len())..].iter().collect();
                    result.push_str(&slice);
                    result.push('\n');
                } else if r == end.0 {
                    let slice: String = chars[..end.1.min(chars.len())].iter().collect();
                    result.push_str(&slice);
                } else {
                    result.push_str(line);
                    result.push('\n');
                }
            }
        }

        Some(result)
    }

    // --- Text Editing Operations ---

    pub fn insert_char(&mut self, c: char) {
        self.delete_selection();
        self.clamp_cursor();

        let mut chars: Vec<char> = self.lines[self.cursor_row].chars().collect();
        chars.insert(self.cursor_col, c);
        self.lines[self.cursor_row] = chars.iter().collect();

        self.undo_stack.push(UndoAction::InsertText {
            row: self.cursor_row,
            col: self.cursor_col,
            text: c.to_string(),
        });
        self.redo_stack.clear();

        self.cursor_col += 1;
        self.preferred_col = self.cursor_col;
        self.is_modified = true;
        self.refresh_syntax();
    }

    pub fn insert_str(&mut self, s: &str) {
        if s.is_empty() {
            return;
        }
        self.delete_selection();

        let insert_lines: Vec<&str> = s.split('\n').collect();
        if insert_lines.len() == 1 {
            let row = self.cursor_row;
            let col = self.cursor_col;
            let mut chars: Vec<char> = self.lines[self.cursor_row].chars().collect();
            for (idx, ch) in s.chars().enumerate() {
                chars.insert(self.cursor_col + idx, ch);
            }
            self.lines[self.cursor_row] = chars.iter().collect();
            self.cursor_col += s.chars().count();

            self.undo_stack.push(UndoAction::InsertText {
                row,
                col,
                text: s.to_string(),
            });
            self.redo_stack.clear();
        } else {
            let cur_line = self.lines[self.cursor_row].clone();
            let chars: Vec<char> = cur_line.chars().collect();
            let prefix: String = chars[..self.cursor_col.min(chars.len())].iter().collect();
            let suffix: String = chars[self.cursor_col.min(chars.len())..].iter().collect();

            let mut new_lines = Vec::new();
            for (i, &l) in insert_lines.iter().enumerate() {
                if i == 0 {
                    let mut first = prefix.clone();
                    first.push_str(l.trim_end_matches('\r'));
                    new_lines.push(first);
                } else if i == insert_lines.len() - 1 {
                    let mut last = l.trim_end_matches('\r').to_string();
                    last.push_str(&suffix);
                    new_lines.push(last);
                } else {
                    new_lines.push(l.trim_end_matches('\r').to_string());
                }
            }

            self.lines.remove(self.cursor_row);
            for (offset, line) in new_lines.into_iter().enumerate() {
                self.lines.insert(self.cursor_row + offset, line);
            }

            self.cursor_row += insert_lines.len() - 1;
            self.cursor_col = insert_lines.last().map(|l| l.trim_end_matches('\r').chars().count()).unwrap_or(0);
        }

        self.preferred_col = self.cursor_col;
        self.is_modified = true;
        self.refresh_syntax();
    }

    pub fn undo(&mut self) {
        if let Some(action) = self.undo_stack.pop() {
            self.apply_undo_action(action.clone(), false);
            self.redo_stack.push(action);
            self.is_modified = true;
            self.refresh_syntax();
        }
    }

    pub fn redo(&mut self) {
        if let Some(action) = self.redo_stack.pop() {
            self.apply_undo_action(action.clone(), true);
            self.undo_stack.push(action);
            self.is_modified = true;
            self.refresh_syntax();
        }
    }

    fn apply_undo_action(&mut self, action: UndoAction, is_redo: bool) {
        match action {
            UndoAction::InsertText { row, col, text } => {
                if is_redo {
                    if let Some(line) = self.lines.get_mut(row) {
                        let mut chars: Vec<char> = line.chars().collect();
                        let insert_idx = col.min(chars.len());
                        for (i, ch) in text.chars().enumerate() {
                            chars.insert(insert_idx + i, ch);
                        }
                        *line = chars.into_iter().collect();
                    }
                    self.cursor_row = row;
                    self.cursor_col = col + text.chars().count();
                } else {
                    if let Some(line) = self.lines.get_mut(row) {
                        let mut chars: Vec<char> = line.chars().collect();
                        let count = text.chars().count();
                        let start = col.min(chars.len());
                        let end = (start + count).min(chars.len());
                        chars.drain(start..end);
                        *line = chars.into_iter().collect();
                    }
                    self.cursor_row = row;
                    self.cursor_col = col;
                }
            }
            UndoAction::DeleteText { row, col, text } => {
                if is_redo {
                    if let Some(line) = self.lines.get_mut(row) {
                        let mut chars: Vec<char> = line.chars().collect();
                        let count = text.chars().count();
                        let start = col.min(chars.len());
                        let end = (start + count).min(chars.len());
                        chars.drain(start..end);
                        *line = chars.into_iter().collect();
                    }
                    self.cursor_row = row;
                    self.cursor_col = col;
                } else {
                    if let Some(line) = self.lines.get_mut(row) {
                        let mut chars: Vec<char> = line.chars().collect();
                        let insert_idx = col.min(chars.len());
                        for (i, ch) in text.chars().enumerate() {
                            chars.insert(insert_idx + i, ch);
                        }
                        *line = chars.into_iter().collect();
                    }
                    self.cursor_row = row;
                    self.cursor_col = col + text.chars().count();
                }
            }
            UndoAction::Group(actions) => {
                if is_redo {
                    for act in actions {
                        self.apply_undo_action(act, true);
                    }
                } else {
                    for act in actions.into_iter().rev() {
                        self.apply_undo_action(act, false);
                    }
                }
            }
        }
        self.preferred_col = self.cursor_col;
    }

    pub fn insert_newline(&mut self) {
        self.delete_selection();
        self.clamp_cursor();

        let cur_line = self.lines[self.cursor_row].clone();
        let chars: Vec<char> = cur_line.chars().collect();

        // Calculate auto-indent from current line
        let indent: String = cur_line.chars().take_while(|c| c.is_whitespace()).collect();

        let left: String = chars[..self.cursor_col].iter().collect();
        let right: String = chars[self.cursor_col..].iter().collect();

        self.lines[self.cursor_row] = left;
        let mut next_line = indent.clone();
        next_line.push_str(&right);
        self.lines.insert(self.cursor_row + 1, next_line);

        self.cursor_row += 1;
        self.cursor_col = indent.chars().count();
        self.preferred_col = self.cursor_col;
        self.is_modified = true;
        self.refresh_syntax();
    }

    pub fn delete_backspace(&mut self) {
        if self.delete_selection() {
            return;
        }

        if self.cursor_col > 0 {
            let mut chars: Vec<char> = self.lines[self.cursor_row].chars().collect();
            let deleted = chars.remove(self.cursor_col - 1);
            self.lines[self.cursor_row] = chars.iter().collect();
            self.cursor_col -= 1;
            self.preferred_col = self.cursor_col;

            self.undo_stack.push(UndoAction::DeleteText {
                row: self.cursor_row,
                col: self.cursor_col,
                text: deleted.to_string(),
            });
            self.redo_stack.clear();
            self.is_modified = true;
            self.refresh_syntax();
        } else if self.cursor_row > 0 {
            let current = self.lines.remove(self.cursor_row);
            self.cursor_row -= 1;
            self.cursor_col = self.line_len(self.cursor_row);
            self.preferred_col = self.cursor_col;
            self.lines[self.cursor_row].push_str(&current);
            self.is_modified = true;
            self.refresh_syntax();
        }
    }

    pub fn delete_forward(&mut self) {
        if self.delete_selection() {
            return;
        }

        let cur_len = self.line_len(self.cursor_row);
        if self.cursor_col < cur_len {
            let mut chars: Vec<char> = self.lines[self.cursor_row].chars().collect();
            chars.remove(self.cursor_col);
            self.lines[self.cursor_row] = chars.iter().collect();
            self.is_modified = true;
            self.refresh_syntax();
        } else if self.cursor_row + 1 < self.lines.len() {
            let next_line = self.lines.remove(self.cursor_row + 1);
            self.lines[self.cursor_row].push_str(&next_line);
            self.is_modified = true;
            self.refresh_syntax();
        }
    }

    pub fn delete_selection(&mut self) -> bool {
        let (start, end) = match self.selection_range() {
            Some(range) => range,
            None => return false,
        };

        if start.0 == end.0 {
            let mut chars: Vec<char> = self.lines[start.0].chars().collect();
            chars.drain(start.1..end.1);
            self.lines[start.0] = chars.iter().collect();
        } else {
            let first_prefix: String = self.lines[start.0].chars().take(start.1).collect();
            let last_suffix: String = self.lines[end.0].chars().skip(end.1).collect();

            let mut merged = first_prefix;
            merged.push_str(&last_suffix);

            self.lines.drain(start.0..=end.0);
            self.lines.insert(start.0, merged);
        }

        self.cursor_row = start.0;
        self.cursor_col = start.1;
        self.preferred_col = self.cursor_col;
        self.selection_anchor = None;
        self.is_modified = true;
        self.refresh_syntax();
        true
    }

    pub fn delete_line(&mut self) {
        if self.lines.len() > 1 {
            self.lines.remove(self.cursor_row);
            if self.cursor_row >= self.lines.len() {
                self.cursor_row = self.lines.len() - 1;
            }
        } else {
            self.lines[0].clear();
            self.cursor_row = 0;
            self.cursor_col = 0;
        }
        self.clamp_cursor();
        self.is_modified = true;
        self.refresh_syntax();
    }

    pub fn duplicate_line(&mut self) {
        let line = self.current_line().to_string();
        self.lines.insert(self.cursor_row + 1, line);
        self.cursor_row += 1;
        self.is_modified = true;
        self.refresh_syntax();
    }

    pub fn move_line_up(&mut self) {
        if self.cursor_row > 0 {
            self.lines.swap(self.cursor_row, self.cursor_row - 1);
            self.cursor_row -= 1;
            self.is_modified = true;
            self.refresh_syntax();
        }
    }

    pub fn move_line_down(&mut self) {
        if self.cursor_row + 1 < self.lines.len() {
            self.lines.swap(self.cursor_row, self.cursor_row + 1);
            self.cursor_row += 1;
            self.is_modified = true;
            self.refresh_syntax();
        }
    }

    pub fn indent_selection(&mut self) {
        let (start_row, end_row) = if let Some(((s_row, _), (e_row, _))) = self.selection_range() {
            (s_row, e_row)
        } else {
            (self.cursor_row, self.cursor_row)
        };

        let indent_str = "    ";
        for r in start_row..=end_row {
            if let Some(line) = self.lines.get_mut(r) {
                line.insert_str(0, indent_str);
            }
        }
        self.cursor_col += 4;
        self.preferred_col = self.cursor_col;
        self.is_modified = true;
        self.refresh_syntax();
    }

    pub fn unindent_selection(&mut self) {
        let (start_row, end_row) = if let Some(((s_row, _), (e_row, _))) = self.selection_range() {
            (s_row, e_row)
        } else {
            (self.cursor_row, self.cursor_row)
        };

        for r in start_row..=end_row {
            if let Some(line) = self.lines.get_mut(r) {
                if line.starts_with("    ") {
                    line.drain(0..4);
                } else if line.starts_with('\t') {
                    line.drain(0..1);
                } else {
                    let spaces = line.chars().take_while(|c| *c == ' ').count();
                    line.drain(0..spaces.min(4));
                }
            }
        }
        self.cursor_col = self.cursor_col.saturating_sub(4);
        self.preferred_col = self.cursor_col;
        self.is_modified = true;
        self.refresh_syntax();
    }

    pub fn goto_line(&mut self, line: usize) {
        let target = line.saturating_sub(1).min(self.lines.len().saturating_sub(1));
        self.cursor_row = target;
        self.cursor_col = 0;
        self.preferred_col = 0;
        self.selection_anchor = None;
    }

    // --- Search & Replace ---

    pub fn search(&mut self, needle: &str, match_case: bool, whole_word: bool, use_regex: bool) {
        self.search_needle = needle.to_string();
        self.search_matches.clear();

        if needle.is_empty() {
            return;
        }

        let pattern = if use_regex {
            if match_case {
                Regex::new(needle)
            } else {
                Regex::new(&format!("(?i){}", needle))
            }
        } else {
            let escaped = regex::escape(needle);
            let p = if whole_word {
                format!(r"\b{}\b", escaped)
            } else {
                escaped
            };
            if match_case {
                Regex::new(&p)
            } else {
                Regex::new(&format!("(?i){}", p))
            }
        };

        if let Ok(re) = pattern {
            for (row, line) in self.lines.iter().enumerate() {
                for mat in re.find_iter(line) {
                    self.search_matches.push(SearchMatch {
                        row,
                        start_col: mat.start(),
                        end_col: mat.end(),
                    });
                }
            }
        }

        if !self.search_matches.is_empty() {
            self.active_match_index = 0;
            // Jump to first match at or after cursor
            for (idx, m) in self.search_matches.iter().enumerate() {
                if m.row >= self.cursor_row {
                    self.active_match_index = idx;
                    break;
                }
            }
            let m = &self.search_matches[self.active_match_index];
            self.cursor_row = m.row;
            self.cursor_col = m.start_col;
            self.preferred_col = self.cursor_col;
        }
    }

    pub fn next_match(&mut self) {
        if !self.search_matches.is_empty() {
            self.active_match_index = (self.active_match_index + 1) % self.search_matches.len();
            let m = &self.search_matches[self.active_match_index];
            self.cursor_row = m.row;
            self.cursor_col = m.start_col;
            self.preferred_col = self.cursor_col;
        }
    }

    pub fn prev_match(&mut self) {
        if !self.search_matches.is_empty() {
            if self.active_match_index > 0 {
                self.active_match_index -= 1;
            } else {
                self.active_match_index = self.search_matches.len() - 1;
            }
            let m = &self.search_matches[self.active_match_index];
            self.cursor_row = m.row;
            self.cursor_col = m.start_col;
            self.preferred_col = self.cursor_col;
        }
    }

    pub fn replace_current(&mut self, replacement: &str) {
        if let Some(m) = self.search_matches.get(self.active_match_index) {
            let row = m.row;
            let start = m.start_col;
            let end = m.end_col;

            if let Some(line) = self.lines.get_mut(row) {
                let prefix: String = line.chars().take(start).collect();
                let suffix: String = line.chars().skip(end).collect();
                *line = format!("{}{}{}", prefix, replacement, suffix);
            }
            self.is_modified = true;
            self.refresh_syntax();
        }
    }

    pub fn replace_all(&mut self, needle: &str, replacement: &str, match_case: bool, whole_word: bool, use_regex: bool) -> usize {
        self.search(needle, match_case, whole_word, use_regex);
        let count = self.search_matches.len();
        if count == 0 {
            return 0;
        }

        // Replace from bottom-up so character indices don't shift earlier matches on same line
        for m in self.search_matches.iter().rev() {
            if let Some(line) = self.lines.get_mut(m.row) {
                let prefix: String = line.chars().take(m.start_col).collect();
                let suffix: String = line.chars().skip(m.end_col).collect();
                *line = format!("{}{}{}", prefix, replacement, suffix);
            }
        }

        self.is_modified = true;
        self.search_matches.clear();
        self.refresh_syntax();
        count
    }

    pub fn word_count(&self) -> (usize, usize, usize, usize) {
        let lines = self.lines.len();
        let mut words = 0;
        let mut chars = 0;
        let mut bytes = 0;

        for l in &self.lines {
            words += l.split_whitespace().count();
            chars += l.chars().count();
            bytes += l.len();
        }

        (lines, words, chars, bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_insert_and_delete() {
        let mut buf = Buffer::new_empty("test.txt".to_string());
        buf.insert_char('H');
        buf.insert_char('i');
        assert_eq!(buf.lines[0], "Hi");
        assert_eq!(buf.cursor_col, 2);

        buf.delete_backspace();
        assert_eq!(buf.lines[0], "H");
        assert_eq!(buf.cursor_col, 1);
    }

    #[test]
    fn test_buffer_undo_redo() {
        let mut buf = Buffer::new_empty("test.txt".to_string());
        buf.insert_str("Hello World");
        assert_eq!(buf.lines[0], "Hello World");

        buf.undo();
        assert_eq!(buf.lines[0], "");

        buf.redo();
        assert_eq!(buf.lines[0], "Hello World");
    }

    #[test]
    fn test_buffer_search_and_replace() {
        let mut buf = Buffer::new_empty("test.txt".to_string());
        buf.insert_str("apple banana apple orange apple");
        buf.search("apple", true, false, false);
        assert_eq!(buf.search_matches.len(), 3);

        let replaced = buf.replace_all("apple", "pear", true, false, false);
        assert_eq!(replaced, 3);
        assert_eq!(buf.lines[0], "pear banana pear orange pear");
    }

    #[test]
    fn test_buffer_word_count() {
        let mut buf = Buffer::new_empty("test.txt".to_string());
        buf.insert_str("Line one\nLine two with five words\nThree");
        let (lines, words, chars, _bytes) = buf.word_count();
        assert_eq!(lines, 3);
        assert_eq!(words, 8);
        assert_eq!(chars, 37);
    }

    #[test]
    fn test_buffer_duplicate_and_delete_line() {
        let mut buf = Buffer::new_empty("test.txt".to_string());
        buf.insert_str("First line\nSecond line");
        buf.set_cursor(0, 0, false);
        buf.duplicate_line();
        assert_eq!(buf.lines.len(), 3);
        assert_eq!(buf.lines[0], "First line");
        assert_eq!(buf.lines[1], "First line");

        buf.delete_line();
        assert_eq!(buf.lines.len(), 2);
        assert_eq!(buf.lines[0], "First line");
        assert_eq!(buf.lines[1], "Second line");
    }
}
