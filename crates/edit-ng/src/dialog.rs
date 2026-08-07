use crate::fuzzy::FuzzyFinder;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmAction {
    CloseCurrentBuffer(usize),
    QuitApplication,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FindField {
    FindInput,
    ReplaceInput,
    MatchCase,
    WholeWord,
    UseRegex,
}

pub enum DialogState {
    OpenFile {
        input: String,
        cursor: usize,
    },
    SaveAs {
        input: String,
        cursor: usize,
    },
    ConfirmClose {
        action: ConfirmAction,
        selected_button: usize, // 0 = Save, 1 = Don't Save, 2 = Cancel
    },
    FindReplace {
        find_query: String,
        find_cursor: usize,
        replace_query: String,
        replace_cursor: usize,
        match_case: bool,
        whole_word: bool,
        use_regex: bool,
        focused_field: FindField,
    },
    GotoLine {
        input: String,
        cursor: usize,
    },
    ThemePicker {
        selected_index: usize,
        search_query: String,
    },
    LanguagePicker {
        selected_index: usize,
        search_query: String,
    },
    WordCount {
        lines: usize,
        words: usize,
        chars: usize,
        bytes: usize,
    },
    FuzzyFinder(FuzzyFinder),
    About,
    ShortcutsHelp,
}
