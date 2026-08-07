use std::collections::HashMap;
use std::env;
use toml::Value;

pub struct I18n {
    current_lang: String,
    translations: HashMap<String, HashMap<String, String>>,
    aliases: HashMap<String, String>,
    pub available_languages: Vec<LanguageInfo>,
}

#[derive(Debug, Clone)]
pub struct LanguageInfo {
    pub code: String,
    pub name: String,
    pub native_name: String,
}

impl I18n {
    pub fn new() -> Self {
        let mut i18n = Self {
            current_lang: "en".into(),
            translations: HashMap::new(),
            aliases: HashMap::new(),
            available_languages: Self::init_language_list(),
        };

        i18n.load_embedded_toml();

        // Auto-detect system language
        if let Some(sys_lang) = Self::detect_system_language() {
            i18n.set_language(&sys_lang);
        }

        i18n
    }

    fn init_language_list() -> Vec<LanguageInfo> {
        vec![
            LanguageInfo { code: "en".into(), name: "English".into(), native_name: "English".into() },
            LanguageInfo { code: "de".into(), name: "German".into(), native_name: "Deutsch".into() },
            LanguageInfo { code: "es".into(), name: "Spanish".into(), native_name: "Español".into() },
            LanguageInfo { code: "fr".into(), name: "French".into(), native_name: "Français".into() },
            LanguageInfo { code: "it".into(), name: "Italian".into(), native_name: "Italiano".into() },
            LanguageInfo { code: "ja".into(), name: "Japanese".into(), native_name: "日本語".into() },
            LanguageInfo { code: "ko".into(), name: "Korean".into(), native_name: "한국어".into() },
            LanguageInfo { code: "zh_hans".into(), name: "Chinese (Simplified)".into(), native_name: "简体中文".into() },
            LanguageInfo { code: "zh_hant".into(), name: "Chinese (Traditional)".into(), native_name: "繁體中文".into() },
            LanguageInfo { code: "hi".into(), name: "Hindi".into(), native_name: "हिन्दी".into() },
            LanguageInfo { code: "pt_br".into(), name: "Portuguese (Brazil)".into(), native_name: "Português (Brasil)".into() },
            LanguageInfo { code: "pt_pt".into(), name: "Portuguese (Portugal)".into(), native_name: "Português (Portugal)".into() },
            LanguageInfo { code: "ru".into(), name: "Russian".into(), native_name: "Русский".into() },
            LanguageInfo { code: "ar".into(), name: "Arabic".into(), native_name: "العربية".into() },
            LanguageInfo { code: "bn".into(), name: "Bengali".into(), native_name: "বাংলা".into() },
            LanguageInfo { code: "cs".into(), name: "Czech".into(), native_name: "Čeština".into() },
            LanguageInfo { code: "da".into(), name: "Danish".into(), native_name: "Dansk".into() },
            LanguageInfo { code: "el".into(), name: "Greek".into(), native_name: "Ελληνικά".into() },
            LanguageInfo { code: "fi".into(), name: "Finnish".into(), native_name: "Suomi".into() },
            LanguageInfo { code: "he".into(), name: "Hebrew".into(), native_name: "עברית".into() },
            LanguageInfo { code: "hu".into(), name: "Hungarian".into(), native_name: "Magyar".into() },
            LanguageInfo { code: "id".into(), name: "Indonesian".into(), native_name: "Bahasa Indonesia".into() },
            LanguageInfo { code: "mr".into(), name: "Marathi".into(), native_name: "मराठी".into() },
            LanguageInfo { code: "nl".into(), name: "Dutch".into(), native_name: "Nederlands".into() },
            LanguageInfo { code: "no".into(), name: "Norwegian".into(), native_name: "Norsk".into() },
            LanguageInfo { code: "pl".into(), name: "Polish".into(), native_name: "Polski".into() },
            LanguageInfo { code: "ro".into(), name: "Romanian".into(), native_name: "Română".into() },
            LanguageInfo { code: "sr".into(), name: "Serbian".into(), native_name: "Српски".into() },
            LanguageInfo { code: "sv".into(), name: "Swedish".into(), native_name: "Svenska".into() },
            LanguageInfo { code: "ta".into(), name: "Tamil".into(), native_name: "தமிழ்".into() },
            LanguageInfo { code: "th".into(), name: "Thai".into(), native_name: "ไทย".into() },
            LanguageInfo { code: "tr".into(), name: "Turkish".into(), native_name: "Türkçe".into() },
            LanguageInfo { code: "uk".into(), name: "Ukrainian".into(), native_name: "Українська".into() },
            LanguageInfo { code: "vi".into(), name: "Vietnamese".into(), native_name: "Tiếng Việt".into() },
        ]
    }

    fn load_embedded_toml(&mut self) {
        let toml_str = include_str!("../../../i18n/edit.toml");
        if let Ok(Value::Table(root)) = toml_str.parse::<Value>() {
            for (key, val) in root {
                if key == "__default__" {
                    continue;
                }
                if key == "__alias__" {
                    if let Value::Table(aliases) = val {
                        for (alias_k, alias_v) in aliases {
                            if let Value::String(target) = alias_v {
                                self.aliases.insert(alias_k.to_lowercase(), target);
                            }
                        }
                    }
                    continue;
                }

                if let Value::Table(lang_map) = val {
                    let mut entry = HashMap::new();
                    for (l_code, str_val) in lang_map {
                        if let Value::String(s) = str_val {
                            entry.insert(l_code.to_lowercase(), s);
                        }
                    }
                    self.translations.insert(key, entry);
                }
            }
        }
    }

    pub fn set_language(&mut self, lang: &str) {
        let code = lang.to_lowercase().replace('-', "_");
        let resolved = if let Some(alias) = self.aliases.get(&code) {
            alias.clone()
        } else {
            code
        };
        self.current_lang = resolved;
    }

    pub fn current_language(&self) -> &str {
        &self.current_lang
    }

    pub fn current_language_name(&self) -> &str {
        for lang in &self.available_languages {
            if lang.code.eq_ignore_ascii_case(&self.current_lang) {
                return &lang.native_name;
            }
        }
        &self.current_lang
    }

    pub fn t<'a>(&'a self, key: &'a str) -> &'a str {
        if let Some(entry) = self.translations.get(key) {
            // 1. Exact match
            if let Some(text) = entry.get(&self.current_lang) {
                return text.as_str();
            }

            // 2. Base code (e.g. "pt_br" -> "pt")
            if let Some((base, _)) = self.current_lang.split_once('_') {
                if let Some(text) = entry.get(base) {
                    return text.as_str();
                }
            }

            // 3. Fallback to English
            if let Some(text) = entry.get("en") {
                return text.as_str();
            }
        }
        key
    }

    fn detect_system_language() -> Option<String> {
        for var in &["LC_ALL", "LC_MESSAGES", "LANG"] {
            if let Ok(val) = env::var(var) {
                if !val.is_empty() {
                    // Extract language code e.g. "en_US.UTF-8" -> "en_us"
                    let cleaned = val.split('.').next().unwrap_or(&val);
                    return Some(cleaned.to_lowercase().replace('-', "_"));
                }
            }
        }
        None
    }
}
