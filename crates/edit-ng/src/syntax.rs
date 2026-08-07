use crate::theme::{parse_color, Theme};
use crossterm::style::Color;
use std::path::Path;
use tree_sitter::{Node, Parser, Tree};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Rust,
    C,
    Cpp,
    Python,
    JavaScript,
    TypeScript,
    Json,
    Toml,
    Markdown,
    Html,
    Css,
    Bash,
    Go,
    Sql,
    PlainText,
}

impl Language {
    pub fn name(&self) -> &'static str {
        match self {
            Language::Rust => "Rust",
            Language::C => "C",
            Language::Cpp => "C++",
            Language::Python => "Python",
            Language::JavaScript => "JavaScript",
            Language::TypeScript => "TypeScript",
            Language::Json => "JSON",
            Language::Toml => "TOML",
            Language::Markdown => "Markdown",
            Language::Html => "HTML",
            Language::Css => "CSS",
            Language::Bash => "Shell",
            Language::Go => "Go",
            Language::Sql => "SQL",
            Language::PlainText => "Plain Text",
        }
    }

    pub fn from_path(path: &Path) -> Self {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            match ext.to_lowercase().as_str() {
                "rs" => Language::Rust,
                "c" | "h" => Language::C,
                "cpp" | "cxx" | "cc" | "hpp" | "hxx" => Language::Cpp,
                "py" | "pyw" => Language::Python,
                "js" | "mjs" | "cjs" | "jsx" => Language::JavaScript,
                "ts" | "mts" | "cts" | "tsx" => Language::TypeScript,
                "json" => Language::Json,
                "toml" => Language::Toml,
                "md" | "markdown" => Language::Markdown,
                "html" | "htm" => Language::Html,
                "css" | "scss" | "sass" | "less" => Language::Css,
                "sh" | "bash" | "zsh" => Language::Bash,
                "go" => Language::Go,
                "sql" => Language::Sql,
                _ => Language::PlainText,
            }
        } else if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            match file_name.to_lowercase().as_str() {
                "cargo.lock" => Language::Toml,
                "dockerfile" => Language::Bash,
                "makefile" => Language::Bash,
                _ => Language::PlainText,
            }
        } else {
            Language::PlainText
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Normal,
    Keyword,
    Function,
    Type,
    String,
    Comment,
    Number,
    Operator,
    Variable,
    Constant,
    Attribute,
    Tag,
    Punctuation,
    Error,
}

impl TokenType {
    pub fn to_color(self, theme: &Theme) -> Color {
        match self {
            TokenType::Normal => parse_color(&theme.ui.foreground),
            TokenType::Keyword => parse_color(&theme.syntax.keyword),
            TokenType::Function => parse_color(&theme.syntax.function),
            TokenType::Type => parse_color(&theme.syntax.r#type),
            TokenType::String => parse_color(&theme.syntax.string),
            TokenType::Comment => parse_color(&theme.syntax.comment),
            TokenType::Number => parse_color(&theme.syntax.number),
            TokenType::Operator => parse_color(&theme.syntax.operator),
            TokenType::Variable => parse_color(&theme.syntax.variable),
            TokenType::Constant => parse_color(&theme.syntax.constant),
            TokenType::Attribute => parse_color(&theme.syntax.attribute),
            TokenType::Tag => parse_color(&theme.syntax.tag),
            TokenType::Punctuation => parse_color(&theme.syntax.punctuation),
            TokenType::Error => parse_color(&theme.syntax.error),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HighlightSpan {
    pub start_col: usize,
    pub end_col: usize,
    pub token_type: TokenType,
}

pub struct SyntaxHighlighter {
    language: Language,
    ts_parser: Option<Parser>,
    ts_tree: Option<Tree>,
}

impl SyntaxHighlighter {
    pub fn new(language: Language) -> Self {
        let mut highlighter = Self {
            language,
            ts_parser: None,
            ts_tree: None,
        };

        if language == Language::Rust {
            let mut parser = Parser::new();
            if parser.set_language(&tree_sitter_rust::LANGUAGE.into()).is_ok() {
                highlighter.ts_parser = Some(parser);
            }
        }

        highlighter
    }

    pub fn set_language(&mut self, language: Language) {
        self.language = language;
        if language == Language::Rust {
            let mut parser = Parser::new();
            if parser.set_language(&tree_sitter_rust::LANGUAGE.into()).is_ok() {
                self.ts_parser = Some(parser);
            } else {
                self.ts_parser = None;
            }
        } else {
            self.ts_parser = None;
        }
        self.ts_tree = None;
    }

    pub fn language(&self) -> Language {
        self.language
    }

    pub fn update_tree(&mut self, full_text: &str) {
        if let Some(parser) = &mut self.ts_parser {
            self.ts_tree = parser.parse(full_text, None);
        }
    }

    pub fn highlight_line(&self, line_index: usize, line: &str) -> Vec<HighlightSpan> {
        if line.is_empty() {
            return Vec::new();
        }

        // 1. Try Tree-sitter for Rust
        if self.language == Language::Rust {
            if let Some(tree) = &self.ts_tree {
                let mut spans = Vec::new();
                let root_node = tree.root_node();
                self.collect_ts_spans(root_node, line_index, &mut spans);
                if !spans.is_empty() {
                    spans.sort_by_key(|s| s.start_col);
                    return spans;
                }
            }
        }

        // 2. High-precision rule-based tokenizer for all supported languages
        self.tokenize_line_regex(line)
    }

    fn collect_ts_spans(&self, node: Node, target_line: usize, spans: &mut Vec<HighlightSpan>) {
        let start_pos = node.start_position();
        let end_pos = node.end_position();

        if start_pos.row > target_line || end_pos.row < target_line {
            return;
        }

        let kind = node.kind();
        let token_type = match kind {
            "use" | "fn" | "let" | "mut" | "struct" | "enum" | "impl" | "trait" | "type"
            | "if" | "else" | "match" | "for" | "while" | "loop" | "return" | "break"
            | "continue" | "pub" | "crate" | "super" | "self" | "Self" | "where" | "as"
            | "async" | "await" | "move" | "unsafe" | "const" | "static" | "mod" | "extern" => {
                Some(TokenType::Keyword)
            }
            "primitive_type" | "type_identifier" => Some(TokenType::Type),
            "identifier" => {
                if let Some(parent) = node.parent() {
                    match parent.kind() {
                        "function_item" | "call_expression" => Some(TokenType::Function),
                        "attribute_item" => Some(TokenType::Attribute),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            "string_literal" | "char_literal" | "raw_string_literal" => Some(TokenType::String),
            "line_comment" | "block_comment" => Some(TokenType::Comment),
            "integer_literal" | "float_literal" => Some(TokenType::Number),
            "macro_invocation" => Some(TokenType::Function),
            "attribute_item" | "inner_attribute_item" => Some(TokenType::Attribute),
            _ => None,
        };

        if let Some(tt) = token_type {
            let start_col = if start_pos.row == target_line { start_pos.column } else { 0 };
            let end_col = if end_pos.row == target_line { end_pos.column } else { usize::MAX };
            spans.push(HighlightSpan {
                start_col,
                end_col,
                token_type: tt,
            });
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.collect_ts_spans(child, target_line, spans);
        }
    }

    fn tokenize_line_regex(&self, line: &str) -> Vec<HighlightSpan> {
        let mut spans = Vec::new();
        let chars: Vec<char> = line.chars().collect();
        let len = chars.len();
        let mut i = 0;

        let comment_prefix = match self.language {
            Language::Python | Language::Bash => Some("#"),
            Language::Sql => Some("--"),
            Language::Html => Some("<!--"),
            Language::PlainText => None,
            _ => Some("//"),
        };

        while i < len {
            // Check for single line comments
            if let Some(cp) = comment_prefix {
                if line[i..].starts_with(cp) {
                    spans.push(HighlightSpan {
                        start_col: i,
                        end_col: len,
                        token_type: TokenType::Comment,
                    });
                    break;
                }
            }

            // Check for strings
            if chars[i] == '"' || chars[i] == '\'' || (chars[i] == '`' && self.language == Language::JavaScript) {
                let quote = chars[i];
                let start = i;
                i += 1;
                let mut escaped = false;
                while i < len {
                    if escaped {
                        escaped = false;
                    } else if chars[i] == '\\' {
                        escaped = true;
                    } else if chars[i] == quote {
                        i += 1;
                        break;
                    }
                    i += 1;
                }
                spans.push(HighlightSpan {
                    start_col: start,
                    end_col: i,
                    token_type: TokenType::String,
                });
                continue;
            }

            // Check for numbers
            if chars[i].is_ascii_digit() || (chars[i] == '.' && i + 1 < len && chars[i + 1].is_ascii_digit()) {
                let start = i;
                if chars[i] == '0' && i + 1 < len && (chars[i + 1] == 'x' || chars[i + 1] == 'b' || chars[i + 1] == 'o') {
                    i += 2;
                }
                while i < len && (chars[i].is_ascii_hexdigit() || chars[i] == '.' || chars[i] == '_' || chars[i] == 'e' || chars[i] == 'E' || chars[i] == 'f' || chars[i] == 'u' || chars[i] == 'i') {
                    i += 1;
                }
                spans.push(HighlightSpan {
                    start_col: start,
                    end_col: i,
                    token_type: TokenType::Number,
                });
                continue;
            }

            // Check for identifiers / keywords
            if chars[i].is_alphabetic() || chars[i] == '_' || chars[i] == '$' || chars[i] == '@' {
                let start = i;
                while i < len && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == '!') {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();

                let token_type = if self.is_keyword(&word) {
                    TokenType::Keyword
                } else if self.is_type(&word) {
                    TokenType::Type
                } else if word.ends_with('!') || (i < len && chars[i] == '(') {
                    TokenType::Function
                } else if word.starts_with('@') || (start > 0 && chars[start - 1] == '#') {
                    TokenType::Attribute
                } else if word.chars().all(|c| c.is_uppercase() || c == '_') && word.len() > 1 {
                    TokenType::Constant
                } else {
                    TokenType::Variable
                };

                spans.push(HighlightSpan {
                    start_col: start,
                    end_col: i,
                    token_type,
                });
                continue;
            }

            // Operators & punctuation
            if "=+-*/%&|^!<>~?:;.,()[]{}".contains(chars[i]) {
                let start = i;
                let is_punct = ".,;()[]{}".contains(chars[i]);
                i += 1;
                spans.push(HighlightSpan {
                    start_col: start,
                    end_col: i,
                    token_type: if is_punct { TokenType::Punctuation } else { TokenType::Operator },
                });
                continue;
            }

            i += 1;
        }

        spans
    }

    fn is_keyword(&self, word: &str) -> bool {
        let kw = match self.language {
            Language::Rust => &[
                "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum",
                "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod",
                "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super",
                "trait", "true", "type", "unsafe", "use", "where", "while",
            ][..],
            Language::Python => &[
                "and", "as", "assert", "async", "await", "break", "class", "continue", "def", "del",
                "elif", "else", "except", "False", "finally", "for", "from", "global", "if", "import",
                "in", "is", "lambda", "None", "nonlocal", "not", "or", "pass", "raise", "return",
                "True", "try", "while", "with", "yield",
            ][..],
            Language::JavaScript | Language::TypeScript => &[
                "async", "await", "break", "case", "catch", "class", "const", "continue", "debugger",
                "default", "delete", "do", "else", "export", "extends", "false", "finally", "for",
                "function", "if", "import", "in", "instanceof", "let", "new", "null", "return",
                "super", "switch", "this", "throw", "true", "try", "typeof", "var", "void", "while",
                "with", "yield", "interface", "type", "declare", "enum",
            ][..],
            Language::C | Language::Cpp => &[
                "auto", "break", "case", "char", "const", "continue", "default", "do", "double",
                "else", "enum", "extern", "float", "for", "goto", "if", "int", "long", "register",
                "return", "short", "signed", "sizeof", "static", "struct", "switch", "typedef",
                "union", "unsigned", "void", "volatile", "while", "class", "namespace", "public",
                "private", "protected", "template", "typename", "new", "delete", "true", "false",
            ][..],
            Language::Toml | Language::Json => &["true", "false", "null"][..],
            Language::Bash => &[
                "if", "then", "else", "elif", "fi", "case", "esac", "for", "select", "while",
                "until", "do", "done", "in", "function", "time", "return", "exit", "local",
            ][..],
            Language::Go => &[
                "break", "default", "func", "interface", "select", "case", "defer", "go", "map",
                "struct", "chan", "else", "goto", "package", "switch", "const", "fallthrough",
                "if", "range", "type", "continue", "for", "import", "return", "var",
            ][..],
            Language::Sql => &[
                "SELECT", "FROM", "WHERE", "INSERT", "INTO", "UPDATE", "DELETE", "JOIN", "LEFT",
                "RIGHT", "INNER", "OUTER", "ON", "GROUP", "BY", "ORDER", "HAVING", "LIMIT", "OFFSET",
                "CREATE", "TABLE", "DROP", "ALTER", "INDEX", "VIEW", "AS", "AND", "OR", "NOT", "IN",
                "select", "from", "where", "insert", "into", "update", "delete", "join", "left",
                "right", "inner", "outer", "on", "group", "by", "order", "having", "limit",
            ][..],
            _ => &[][..],
        };
        kw.contains(&word)
    }

    fn is_type(&self, word: &str) -> bool {
        let types = match self.language {
            Language::Rust => &[
                "bool", "char", "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32",
                "u64", "u128", "usize", "f32", "f64", "str", "String", "Vec", "Option", "Result",
                "Box", "Rc", "Arc", "Cell", "RefCell", "Mutex", "HashMap", "HashSet", "BTreeMap",
            ][..],
            Language::Python => &[
                "int", "float", "str", "bool", "list", "dict", "set", "tuple", "bytes", "object",
                "Any", "Union", "Optional", "List", "Dict", "Set", "Tuple", "Callable",
            ][..],
            Language::JavaScript | Language::TypeScript => &[
                "string", "number", "boolean", "symbol", "bigint", "undefined", "object", "any",
                "unknown", "never", "void", "Array", "Promise", "Record", "Map", "Set",
            ][..],
            Language::C | Language::Cpp => &[
                "int", "char", "float", "double", "void", "size_t", "ssize_t", "uint8_t", "uint16_t",
                "uint32_t", "uint64_t", "int8_t", "int16_t", "int32_t", "int64_t", "bool", "string",
                "vector", "map", "unique_ptr", "shared_ptr",
            ][..],
            Language::Go => &[
                "bool", "byte", "complex64", "complex128", "error", "float32", "float64",
                "int", "int8", "int16", "int32", "int64", "rune", "string", "uint", "uint8",
                "uint16", "uint32", "uint64", "uintptr",
            ][..],
            _ => &[][..],
        };
        types.contains(&word)
    }
}
