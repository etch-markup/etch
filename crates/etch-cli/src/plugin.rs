use crate::{config, npm};
use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

type DynError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, Clone, PartialEq, Eq)]
struct ListedPlugin {
    display_name: String,
    version: String,
    source: &'static str,
}

pub fn add_plugin(name: &str, global: bool) -> Result<(), DynError> {
    let cwd = env::current_dir()?;
    let project_root = config::find_project_root(&cwd);
    let plugin_root = if global {
        global_plugin_root()?
    } else {
        project_root.join(".etch/plugins")
    };

    fs::create_dir_all(&plugin_root)?;
    let info = npm::fetch_npm_package(name, &plugin_root)?;

    if !global {
        let mut current = config::load_config(&project_root)?;
        config::add_plugin_to_config(&mut current, name);
        config::save_config(&project_root, &current)?;
    }

    println!("Installed {}@{}", info.name, info.version);
    Ok(())
}

pub fn remove_plugin(name: &str) -> Result<(), DynError> {
    let cwd = env::current_dir()?;
    let project_root = config::find_project_root(&cwd);
    let package_name = npm::normalized_package_name(name);
    let install_path = project_root.join(".etch/plugins").join(&package_name);

    if install_path.exists() {
        fs::remove_dir_all(&install_path)?;
    } else {
        eprintln!(
            "warning: plugin directory not found: {}",
            install_path.display()
        );
    }

    let mut current = config::load_config(&project_root)?;
    config::remove_plugin_from_config(&mut current, name);
    config::save_config(&project_root, &current)?;

    println!("Removed {name}");
    Ok(())
}

pub fn list_plugins() -> Result<(), DynError> {
    let cwd = env::current_dir()?;
    let project_root = config::find_project_root(&cwd);
    let mut plugins = Vec::new();

    collect_installed_plugins(&project_root.join(".etch/plugins"), "project", &mut plugins)?;
    collect_installed_plugins(&global_plugin_root()?, "global", &mut plugins)?;

    if plugins.is_empty() {
        println!("No plugins installed");
        return Ok(());
    }

    println!("Plugins:");
    for plugin in plugins {
        println!(
            "  ● {:<16} {:<8} ({})",
            plugin.display_name, plugin.version, plugin.source
        );
    }

    Ok(())
}

fn global_plugin_root() -> io::Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "failed to determine home directory",
        )
    })?;
    Ok(home.join(".etch/plugins"))
}

fn collect_installed_plugins(
    root: &Path,
    source: &'static str,
    out: &mut Vec<ListedPlugin>,
) -> Result<(), DynError> {
    if !root.exists() {
        return Ok(());
    }

    let mut entries = fs::read_dir(root)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let file_name = entry.file_name();
        let package_name = file_name.to_string_lossy();
        if !package_name.starts_with("etch-plugin-") {
            continue;
        }

        let info = npm::read_installed_package_info(&path)?;
        out.push(ListedPlugin {
            display_name: package_name.trim_start_matches("etch-plugin-").to_string(),
            version: info.version,
            source,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::collect_installed_plugins;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn collects_installed_plugins_from_directory() {
        let tempdir = tempdir().expect("tempdir");
        let plugin_dir = tempdir.path().join("etch-plugin-math-extended");
        fs::create_dir_all(&plugin_dir).expect("create plugin dir");
        fs::write(
            plugin_dir.join("package.json"),
            r#"{
  "name": "etch-plugin-math-extended",
  "version": "1.2.3",
  "etch-plugin": {},
  "main": "index.js"
}"#,
        )
        .expect("write package");
        fs::write(plugin_dir.join("index.js"), "export default {};").expect("write entrypoint");

        let mut plugins = Vec::new();
        collect_installed_plugins(tempdir.path(), "project", &mut plugins)
            .expect("collect plugins");

        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].display_name, "math-extended");
        assert_eq!(plugins[0].version, "1.2.3");
        assert_eq!(plugins[0].source, "project");
    }
}
