# Fuzzy File Finder Plugin for edit-ng

The **Fuzzy File Finder** plugin provides an instantaneous, interactive quick-open file picker for `edit-ng`. Inspired by modern IDEs and editors like VS Code, Sublime Text, and Telescope/FZF, it enables developers to rapidly search, match, preview, and open files anywhere in their project directory tree.

## Features

- **Blazing Fast Matching**: Modified Sublime Text style scoring algorithm with bonuses for consecutive matches, path boundary / slash matches, word start boundaries (camelCase and snake_case), and filename matches.
- **Matched Character Highlights**: Highlights exactly which characters matched your query in the results list.
- **Interactive Live Preview**: Split preview window showing file contents, line numbers, and file size in real-time as you navigate search results.
- **Smart Ignore Filter**: Automatically respects `.gitignore` rules and ignores common noise directories (`target/`, `node_modules/`, `.git/`, `dist/`, etc.).
- **Keyboard Navigation**:
  - `Ctrl+P` / `Ctrl+O`: Open Fuzzy File Finder
  - `Up` / `Down` or `Ctrl+K` / `Ctrl+J`: Navigate search candidates
  - `Enter`: Open selected file in editor buffer
  - `Esc`: Close Finder and return to active document

## Configuration

Settings can be customized in `plugins/fuzzy_finder/plugin.toml`:

```toml
[settings]
max_results = 100
show_preview = true
preview_lines = 40
follow_symlinks = false
ignored_directories = [".git", "target", "node_modules", "dist", "build"]
```
