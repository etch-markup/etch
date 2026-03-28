use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct EtchConfig {
    #[serde(default)]
    pub plugins: Vec<PluginDeclaration>,
    #[serde(default)]
    pub theme: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PluginDeclaration {
    Name(String),
    Detailed { name: String, config: Value },
}

impl PluginDeclaration {
    pub fn name(&self) -> &str {
        match self {
            Self::Name(name) => name,
            Self::Detailed { name, .. } => name,
        }
    }
}

pub fn find_project_root(start: &Path) -> PathBuf {
    let anchor = if start.is_dir() {
        start.to_path_buf()
    } else {
        start.parent().unwrap_or(start).to_path_buf()
    };

    for ancestor in anchor.ancestors() {
        if config_path(ancestor).exists() {
            return ancestor.to_path_buf();
        }
    }

    anchor
}

pub fn config_path(project_root: &Path) -> PathBuf {
    project_root.join("etch.config.json")
}

pub fn load_config(project_root: &Path) -> io::Result<EtchConfig> {
    let path = config_path(project_root);
    if !path.exists() {
        return Ok(EtchConfig::default());
    }

    let raw = fs::read_to_string(&path)?;
    serde_json::from_str(&raw).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

pub fn save_config(project_root: &Path, config: &EtchConfig) -> io::Result<()> {
    let path = config_path(project_root);
    let json = serde_json::to_string_pretty(config).map_err(io::Error::other)?;
    fs::write(path, format!("{json}\n"))
}

pub fn add_plugin_to_config(config: &mut EtchConfig, name: &str) {
    if config.plugins.iter().any(|plugin| plugin.name() == name) {
        return;
    }

    config
        .plugins
        .push(PluginDeclaration::Name(name.to_string()));
}

pub fn remove_plugin_from_config(config: &mut EtchConfig, name: &str) {
    config.plugins.retain(|plugin| plugin.name() != name);
}

#[cfg(test)]
mod tests {
    use super::{
        EtchConfig, PluginDeclaration, add_plugin_to_config, find_project_root, load_config,
        remove_plugin_from_config, save_config,
    };
    use std::{fs, path::Path};
    use tempfile::tempdir;

    #[test]
    fn reads_and_writes_config() {
        let tempdir = tempdir().expect("tempdir");
        let root = tempdir.path();

        let config = EtchConfig {
            plugins: vec![
                PluginDeclaration::Name("math-extended".into()),
                PluginDeclaration::Detailed {
                    name: "anthroverse".into(),
                    config: serde_json::json!({ "apiBase": "https://example.com" }),
                },
            ],
            theme: Some("academic".into()),
        };

        save_config(root, &config).expect("save config");
        let loaded = load_config(root).expect("load config");

        assert_eq!(loaded, config);
    }

    #[test]
    fn adds_and_removes_plugins_without_touching_detailed_entries() {
        let mut config = EtchConfig {
            plugins: vec![PluginDeclaration::Detailed {
                name: "anthroverse".into(),
                config: serde_json::json!({ "enabled": true }),
            }],
            theme: None,
        };

        add_plugin_to_config(&mut config, "math-extended");
        add_plugin_to_config(&mut config, "math-extended");
        remove_plugin_from_config(&mut config, "anthroverse");

        assert_eq!(
            config.plugins,
            vec![PluginDeclaration::Name("math-extended".into())]
        );
    }

    #[test]
    fn finds_nearest_project_root_with_config() {
        let tempdir = tempdir().expect("tempdir");
        let root = tempdir.path();
        let nested = root.join("a/b/c");
        fs::create_dir_all(&nested).expect("create nested");
        fs::write(root.join("etch.config.json"), "{\n  \"plugins\": []\n}\n")
            .expect("write config");

        assert_eq!(find_project_root(&nested), Path::new(root));
    }
}
