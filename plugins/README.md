# edit-ng Plugins

The `plugins/` directory contains modular plugins that extend `edit-ng`'s functionality.

## Included Plugins

1. **`fuzzy_finder/`**: Fast fuzzy file finder and quick-open switcher (`Ctrl+P`).
2. **`git_status/`**: Git branch and file modification status indicator.
3. **`word_counter/`**: Document statistics (lines, words, characters, UTF-8 bytes).

## Plugin Format

Each plugin is defined by a folder containing a `plugin.toml` manifest:

```toml
[plugin]
name = "plugin_name"
display_name = "Human Readable Name"
version = "0.1.0"
author = "Author Name"
description = "Plugin description"
enabled = true

[keybindings]
trigger = "Ctrl+Shift+X"
```
