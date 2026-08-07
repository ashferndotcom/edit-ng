use std::path::PathBuf;

#[test]
fn test_theme_manager_loading() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../themes");
    assert!(root.exists(), "themes directory should exist");
    let files: Vec<_> = std::fs::read_dir(&root).unwrap().collect();
    assert!(files.len() >= 11, "Should have at least 11 themes, found {}", files.len());
}

#[test]
fn test_i18n_languages_loading() {
    let toml_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../i18n/edit.toml");
    assert!(toml_path.exists(), "i18n/edit.toml must exist");
    let content = std::fs::read_to_string(&toml_path).unwrap();
    match content.parse::<toml::Value>() {
        Ok(parsed) => {
            let table = parsed.as_table().unwrap();
            let default_langs = table.get("__default__").and_then(|v| v.as_array()).unwrap();
            assert!(default_langs.len() >= 34, "Should have at least 34 languages localized, found {}", default_langs.len());
        }
        Err(e) => {
            panic!("Failed to parse i18n/edit.toml: {}", e);
        }
    }
}
