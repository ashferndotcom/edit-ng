use ignore::WalkBuilder;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct FuzzyMatchItem {
    pub display_path: String,
    pub full_path: PathBuf,
    pub file_name: String,
    pub score: i64,
    pub matched_indices: Vec<usize>,
    pub file_size: u64,
}

pub struct FuzzyFinder {
    pub query: String,
    pub all_files: Vec<PathBuf>,
    pub filtered_items: Vec<FuzzyMatchItem>,
    pub selected_index: usize,
    pub root_dir: PathBuf,
    pub preview_content: Option<Vec<String>>,
    pub preview_file_path: Option<PathBuf>,
}

impl FuzzyFinder {
    pub fn new(root_dir: &Path) -> Self {
        let mut finder = Self {
            query: String::new(),
            all_files: Vec::new(),
            filtered_items: Vec::new(),
            selected_index: 0,
            root_dir: root_dir.to_path_buf(),
            preview_content: None,
            preview_file_path: None,
        };
        finder.refresh_files();
        finder
    }

    pub fn refresh_files(&mut self) {
        let mut files = Vec::new();
        let walker = WalkBuilder::new(&self.root_dir)
            .hidden(false)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .filter_entry(|entry| {
                if let Some(name) = entry.file_name().to_str() {
                    if name == ".git" || name == "target" || name == "node_modules" || name == ".cache" {
                        return false;
                    }
                }
                true
            })
            .build();

        for result in walker {
            if let Ok(entry) = result {
                if entry.file_type().map_or(false, |ft| ft.is_file()) {
                    files.push(entry.into_path());
                }
            }
        }

        self.all_files = files;
        self.update_query("");
    }

    pub fn update_query(&mut self, query: &str) {
        self.query = query.to_string();
        let q_lower = query.to_lowercase();
        let mut items = Vec::new();

        for file_path in &self.all_files {
            let rel_path = file_path
                .strip_prefix(&self.root_dir)
                .unwrap_or(file_path)
                .to_string_lossy()
                .to_string();

            let file_name = file_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            let file_size = fs::metadata(file_path).map(|m| m.len()).unwrap_or(0);

            if q_lower.is_empty() {
                items.push(FuzzyMatchItem {
                    display_path: rel_path,
                    full_path: file_path.clone(),
                    file_name,
                    score: 0,
                    matched_indices: Vec::new(),
                    file_size,
                });
            } else if let Some((score, matched_indices)) = fuzzy_match(&q_lower, &rel_path) {
                items.push(FuzzyMatchItem {
                    display_path: rel_path,
                    full_path: file_path.clone(),
                    file_name,
                    score,
                    matched_indices,
                    file_size,
                });
            }
        }

        if !q_lower.is_empty() {
            items.sort_by(|a, b| b.score.cmp(&a.score));
        }

        self.filtered_items = items;
        self.selected_index = 0;
        self.update_preview();
    }

    pub fn move_selection_up(&mut self) {
        if !self.filtered_items.is_empty() {
            if self.selected_index > 0 {
                self.selected_index -= 1;
            } else {
                self.selected_index = self.filtered_items.len() - 1;
            }
            self.update_preview();
        }
    }

    pub fn move_selection_down(&mut self) {
        if !self.filtered_items.is_empty() {
            if self.selected_index + 1 < self.filtered_items.len() {
                self.selected_index += 1;
            } else {
                self.selected_index = 0;
            }
            self.update_preview();
        }
    }

    pub fn selected_item(&self) -> Option<&FuzzyMatchItem> {
        self.filtered_items.get(self.selected_index)
    }

    pub fn update_preview(&mut self) {
        if let Some(item) = self.filtered_items.get(self.selected_index) {
            if self.preview_file_path.as_ref() == Some(&item.full_path) {
                return;
            }
            self.preview_file_path = Some(item.full_path.clone());

            // Read up to 80 lines for preview
            if item.file_size < 2 * 1024 * 1024 {
                if let Ok(content) = fs::read_to_string(&item.full_path) {
                    let lines: Vec<String> = content.lines().take(80).map(|s| s.to_string()).collect();
                    self.preview_content = Some(lines);
                    return;
                }
            }
            self.preview_content = Some(vec!["[Binary or inaccessible file content]".into()]);
        } else {
            self.preview_content = None;
            self.preview_file_path = None;
        }
    }
}

pub fn fuzzy_match(pattern: &str, target: &str) -> Option<(i64, Vec<usize>)> {
    if pattern.is_empty() {
        return Some((0, Vec::new()));
    }

    let pattern_chars: Vec<char> = pattern.to_lowercase().chars().collect();
    let target_chars: Vec<char> = target.chars().collect();
    let target_lower: Vec<char> = target.to_lowercase().chars().collect();

    let mut pattern_idx = 0;
    let mut matched_indices = Vec::new();
    let mut score: i64 = 0;
    let mut consecutive_bonus: i64 = 0;
    let mut last_match_idx: Option<usize> = None;

    let target_len = target_chars.len();
    let pattern_len = pattern_chars.len();

    // Find the start of the filename in the path
    let filename_start = target.rfind('/').or_else(|| target.rfind('\\')).map(|p| p + 1).unwrap_or(0);

    for (t_idx, &t_char) in target_lower.iter().enumerate() {
        if pattern_idx < pattern_len && t_char == pattern_chars[pattern_idx] {
            matched_indices.push(t_idx);
            let mut char_score: i64 = 10;

            // Consecutive match bonus
            if let Some(prev) = last_match_idx {
                if prev + 1 == t_idx {
                    consecutive_bonus += 20;
                    char_score += consecutive_bonus;
                } else {
                    consecutive_bonus = 0;
                }
            }

            // Word boundary bonus
            if t_idx == 0 || t_idx == filename_start {
                char_score += 35; // Start of string or filename
            } else {
                let prev_char = target_chars[t_idx - 1];
                if prev_char == '/' || prev_char == '\\' || prev_char == '_' || prev_char == '-' || prev_char == '.' {
                    char_score += 25;
                } else if prev_char.is_lowercase() && target_chars[t_idx].is_uppercase() {
                    char_score += 20; // CamelCase boundary
                }
            }

            // Filename bonus
            if t_idx >= filename_start {
                char_score += 15;
            }

            // Exact case match bonus
            if target_chars[t_idx] == pattern.chars().nth(pattern_idx).unwrap_or('\0') {
                char_score += 5;
            }

            score += char_score;
            last_match_idx = Some(t_idx);
            pattern_idx += 1;
        }
    }

    if pattern_idx == pattern_len {
        // Penalty for length difference
        let len_diff = (target_len.saturating_sub(pattern_len)) as i64;
        score -= len_diff / 2;
        Some((score, matched_indices))
    } else {
        None
    }
}
