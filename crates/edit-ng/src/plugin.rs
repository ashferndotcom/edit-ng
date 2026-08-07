use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginMetadata {
    pub name: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginManifest {
    pub plugin: PluginMetadata,
    #[serde(default)]
    pub keybindings: HashMap<String, String>,
    #[serde(default)]
    pub settings: HashMap<String, toml::Value>,
}

pub struct PluginManager {
    pub plugins: Vec<PluginManifest>,
}

impl PluginManager {
    pub fn new() -> Self {
        let mut manager = Self {
            plugins: Vec::new(),
        };
        manager.discover_plugins();
        manager
    }

    pub fn discover_plugins(&mut self) {
        let mut search_dirs = vec![
            PathBuf::from("plugins"),
            PathBuf::from("../plugins"),
            PathBuf::from("../../plugins"),
        ];

        if let Some(config_dir) = dirs::config_dir() {
            search_dirs.push(config_dir.join("edit-ng").join("plugins"));
        }

        for base_dir in search_dirs {
            if base_dir.is_dir() {
                if let Ok(entries) = fs::read_dir(&base_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            let manifest_file = path.join("plugin.toml");
                            if manifest_file.is_file() {
                                if let Ok(content) = fs::read_to_string(&manifest_file) {
                                    if let Ok(manifest) = toml::from_str::<PluginManifest>(&content) {
                                        if !self.plugins.iter().any(|p| p.plugin.name == manifest.plugin.name) {
                                            self.plugins.push(manifest);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn get_plugin(&self, name: &str) -> Option<&PluginManifest> {
        self.plugins.iter().find(|p| p.plugin.name == name)
    }

    pub fn is_plugin_enabled(&self, name: &str) -> bool {
        self.plugins
            .iter()
            .find(|p| p.plugin.name == name)
            .map_or(false, |p| p.plugin.enabled)
    }
}
