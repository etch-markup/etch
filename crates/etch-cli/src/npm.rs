use flate2::read::GzDecoder;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::{fs, io, path::Path};
use tar::Archive;
use tempfile::tempdir_in;

type DynError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Deserialize)]
struct RegistryResponse {
    name: String,
    version: String,
    dist: RegistryDist,
}

#[derive(Debug, Deserialize)]
struct RegistryDist {
    tarball: String,
}

#[derive(Debug, Deserialize)]
struct PluginPackageManifest {
    name: String,
    version: String,
    #[serde(rename = "etch-plugin")]
    etch_plugin: Option<serde_json::Value>,
    main: Option<String>,
    exports: Option<serde_json::Value>,
}

pub fn fetch_npm_package(name: &str, dest: &Path) -> Result<PackageInfo, DynError> {
    let package_name = normalized_package_name(name);
    let client = Client::builder().build()?;
    let response = client
        .get(format!("https://registry.npmjs.org/{package_name}/latest"))
        .send()?
        .error_for_status()?;
    let registry: RegistryResponse = response.json()?;

    let tempdir = tempdir_in(dest)?;
    let archive_path = tempdir.path().join("plugin.tgz");
    let bytes = client
        .get(&registry.dist.tarball)
        .send()?
        .error_for_status()?
        .bytes()?;
    fs::write(&archive_path, &bytes)?;

    let unpack_root = tempdir.path().join("unpacked");
    fs::create_dir_all(&unpack_root)?;
    let archive_file = fs::File::open(&archive_path)?;
    let decoder = GzDecoder::new(archive_file);
    let mut archive = Archive::new(decoder);
    archive.unpack(&unpack_root)?;

    let package_root = unpack_root.join("package");
    let manifest_path = package_root.join("package.json");
    let manifest_raw = fs::read_to_string(&manifest_path)?;
    let manifest: PluginPackageManifest = serde_json::from_str(&manifest_raw)?;
    validate_manifest(&package_name, &manifest, &package_root)?;

    let install_path = dest.join(&package_name);
    if install_path.exists() {
        fs::remove_dir_all(&install_path)?;
    }
    fs::rename(&package_root, &install_path)?;

    Ok(PackageInfo {
        name: registry.name,
        version: registry.version,
    })
}

pub fn normalized_package_name(name: &str) -> String {
    if name.starts_with("etch-plugin-") {
        name.to_string()
    } else {
        format!("etch-plugin-{name}")
    }
}

pub fn read_installed_package_info(path: &Path) -> Result<PackageInfo, DynError> {
    let manifest_raw = fs::read_to_string(path.join("package.json"))?;
    let manifest: PluginPackageManifest = serde_json::from_str(&manifest_raw)?;

    Ok(PackageInfo {
        name: manifest.name,
        version: manifest.version,
    })
}

fn validate_manifest(
    expected_package_name: &str,
    manifest: &PluginPackageManifest,
    package_root: &Path,
) -> Result<(), DynError> {
    if manifest.name != expected_package_name {
        return Err(io::Error::other(format!(
            "downloaded package name mismatch: expected {expected_package_name}, got {}",
            manifest.name
        ))
        .into());
    }

    if manifest.etch_plugin.is_none() {
        return Err(
            io::Error::other("package.json is missing the required \"etch-plugin\" field").into(),
        );
    }

    if manifest.main.is_none() && manifest.exports.is_none() {
        return Err(
            io::Error::other("package.json must declare either \"main\" or \"exports\"").into(),
        );
    }

    if let Some(main) = &manifest.main {
        let candidate = package_root.join(main);
        if !candidate.exists() {
            return Err(
                io::Error::other(format!("package entrypoint does not exist: {main}")).into(),
            );
        }
    }

    Ok(())
}
