//! A package pinned by `url` and `sha256`: downloaded once, checked, unpacked
//! into the folder of the cache that its hash names, and read from there on
//! every later run.
//!
//! The cache keeps the package as its zip laid it out, so a Vale package keeps
//! its `.vale.ini`, its `meta.json` and its vocabularies where Vale puts them.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs::{self, File};
use std::io::{self, Cursor, Read};
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{env, process};

use sha2::{Digest, Sha256};
use zip::ZipArchive;

/// The most a download may hold, 32 MiB.
const MAX_DOWNLOAD: u64 = 32 * 1024 * 1024;

/// The most a package may hold once unpacked, 128 MiB.
const MAX_UNPACKED: u64 = 128 * 1024 * 1024;

/// How long one fetch may take, redirects included.
const TIMEOUT: Duration = Duration::from_secs(30);

/// A zip that has passed the checks that need no file written.
type Archive = ZipArchive<Cursor<Vec<u8>>>;

/// The package folder of the package `url` names, fetched into the cache folder
/// when it is not there yet.
///
/// `cache_dir` is the cache folder, or the per-user default when it is `None`.
/// The folder is the `styles` folder of the package's one top folder, or the
/// unpacked zip when that top folder is itself a style.
///
/// # Errors
///
/// Returns the fault, which names the `url`: the cache folder is unknown, the
/// request failed, the download is over 32 MiB or does not match `sha256`, or
/// the zip is not a zip, holds a file outside the package, holds a link, has no
/// single top folder, or unpacks to over 128 MiB.
pub(crate) fn fetch(cache_dir: Option<&Path>, url: &str, sha256: &str) -> Result<PathBuf, String> {
    let cache = cache_folder(cache_dir)?;
    let folder = cache.join(sha256);
    if !folder.is_dir() {
        let body = download(url)?;
        let actual = hex(&Sha256::digest(&body));
        if actual != sha256 {
            return Err(format!(
                "`{url}` does not match its `sha256`: expected {sha256}, got {actual}"
            ));
        }
        install(&cache, &folder, url, body)?;
    }
    package_folder(&folder).map_err(|error| format!("`{url}`: {error}"))
}

/// The cache folder: `option`, or the per-user folder `hakkiri`, which is
/// `~/Library/Caches/hakkiri` on macOS and `$XDG_CACHE_HOME/hakkiri` or
/// `~/.cache/hakkiri` elsewhere. An unset or relative base is ignored.
fn cache_folder(option: Option<&Path>) -> Result<PathBuf, String> {
    if let Some(folder) = option {
        return Ok(folder.to_path_buf());
    }
    let home = absolute_variable("HOME");
    let base = if cfg!(target_os = "macos") {
        home.map(|home| home.join("Library").join("Caches"))
    } else {
        absolute_variable("XDG_CACHE_HOME").or_else(|| home.map(|home| home.join(".cache")))
    };
    base.map(|base| base.join("hakkiri")).ok_or_else(|| {
        "no cache folder: set `--cache-dir` or `HAKKIRI_CACHE_DIR`, or HOME".to_owned()
    })
}

/// The value of the variable `name` when it is an absolute path.
fn absolute_variable(name: &str) -> Option<PathBuf> {
    env::var_os(name)
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
}

/// Downloads `url`, which fails when it answers an error status, takes over
/// [`TIMEOUT`], or holds over [`MAX_DOWNLOAD`] bytes.
fn download(url: &str) -> Result<Vec<u8>, String> {
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(TIMEOUT))
        .build();
    let agent = ureq::Agent::new_with_config(config);
    let mut response = agent
        .get(url)
        .call()
        .map_err(|error| fetch_fault(url, &error))?;
    response
        .body_mut()
        .with_config()
        .limit(MAX_DOWNLOAD)
        .read_to_vec()
        .map_err(|error| fetch_fault(url, &error))
}

/// The fault of a failed request.
fn fetch_fault(url: &str, error: &ureq::Error) -> String {
    if let ureq::Error::StatusCode(code) = error {
        format!("could not fetch `{url}`: the server answered {code}")
    } else if let ureq::Error::BodyExceedsLimit(_) = error {
        format!("`{url}` is over 32 MiB")
    } else {
        format!("could not fetch `{url}`: {error}")
    }
}

/// The lowercase hex of `bytes`.
fn hex(bytes: &[u8]) -> String {
    bytes.iter().fold(String::new(), |mut text, byte| {
        let _ = write!(text, "{byte:02x}");
        text
    })
}

/// Checks the zip in `body`, unpacks it into a folder beside `folder`, and
/// renames that folder to `folder`. A run that fetches at the same time may
/// win the rename, and then its folder is the one used.
fn install(cache: &Path, folder: &Path, url: &str, body: Vec<u8>) -> Result<(), String> {
    let mut archive = ZipArchive::new(Cursor::new(body))
        .map_err(|error| format!("`{url}` is not a zip: {error}"))?;
    check_names(&mut archive, url)?;
    fs::create_dir_all(cache).map_err(|error| {
        format!(
            "cannot create the cache folder `{}`: {error}",
            cache.display()
        )
    })?;
    let staging = cache.join(format!(".unpacking-{}", process::id()));
    let _ = fs::remove_dir_all(&staging);
    if let Err(fault) = unpack(&mut archive, &staging, url) {
        let _ = fs::remove_dir_all(&staging);
        return Err(fault);
    }
    match fs::rename(&staging, folder) {
        Ok(()) => Ok(()),
        Err(_) if folder.is_dir() => {
            let _ = fs::remove_dir_all(&staging);
            Ok(())
        }
        Err(error) => {
            let _ = fs::remove_dir_all(&staging);
            Err(format!(
                "cannot move the package into `{}`: {error}",
                folder.display()
            ))
        }
    }
}

/// Refuses a zip with a name outside the package, a link, a file at its root,
/// or other than one top folder.
fn check_names(archive: &mut Archive, url: &str) -> Result<(), String> {
    let mut tops = BTreeSet::new();
    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .map_err(|error| format!("`{url}` is not a zip: {error}"))?;
        let name = String::from_utf8_lossy(entry.name_raw()).into_owned();
        let Some(path) = entry.enclosed_name() else {
            return Err(format!(
                "`{url}` names a file outside the package: `{name}`"
            ));
        };
        if entry.is_symlink() {
            return Err(format!("`{url}` holds a link: `{name}`"));
        }
        let mut parts = path.components();
        let top = parts.next().map(|part| part.as_os_str().to_owned());
        if parts.next().is_none() && !entry.is_dir() {
            return Err(format!(
                "`{url}` holds a file at its root, `{name}`: a package holds one top folder"
            ));
        }
        tops.extend(top);
    }
    match tops.len() {
        1 => Ok(()),
        count => Err(format!(
            "`{url}` holds {count} top folders: a package holds one top folder"
        )),
    }
}

/// Writes the files of `archive` under `staging`, and refuses a zip that
/// unpacks to over [`MAX_UNPACKED`] bytes.
fn unpack(archive: &mut Archive, staging: &Path, url: &str) -> Result<(), String> {
    let mut total = 0_u64;
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| format!("`{url}` is not a zip: {error}"))?;
        let Some(relative) = entry.enclosed_name() else {
            continue;
        };
        let target = staging.join(relative);
        let written = if entry.is_dir() {
            fs::create_dir_all(&target)
        } else {
            let parent = target.parent().unwrap_or(staging);
            fs::create_dir_all(parent).and_then(|()| {
                let mut file = File::create(&target)?;
                let room = MAX_UNPACKED.saturating_sub(total).saturating_add(1);
                let count = io::copy(&mut (&mut entry).take(room), &mut file)?;
                total = total.saturating_add(count);
                Ok(())
            })
        };
        written.map_err(|error| format!("cannot unpack `{url}` into the cache: {error}"))?;
        if total > MAX_UNPACKED {
            return Err(format!("`{url}` unpacks to over 128 MiB"));
        }
    }
    Ok(())
}

/// The folder of `folder` that holds the styles: the `styles` folder of its one
/// top folder, or `folder` itself when that top folder is a style.
fn package_folder(folder: &Path) -> io::Result<PathBuf> {
    let top = fs::read_dir(folder)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| path.is_dir())
        .ok_or_else(|| io::Error::other("the package holds no folder"))?;
    let styles = top.join("styles");
    Ok(if styles.is_dir() {
        styles
    } else {
        folder.to_path_buf()
    })
}
