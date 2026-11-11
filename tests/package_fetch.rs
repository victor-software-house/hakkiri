//! A package pinned by `url` and `sha256`: the scenarios of the package-fetch
//! change.
//!
//! Each test starts a small HTTP server on a local port, writes a project in a
//! temporary directory, and runs the binary there with its cache in another
//! temporary folder.

#![expect(
    clippy::panic_in_result_fn,
    reason = "A test returns Result for `?` and fails through an assertion."
)]

mod common;

use std::collections::HashMap;
use std::error::Error;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread::JoinHandle;

use common::run;
use ctl_core::fixture::{Root, Suite};

type TestResult = Result<(), Box<dyn Error>>;

/// The fixture packages and the SHA-256 of each file.
const TEAM: &[u8] = include_bytes!("fixtures/packages/Team.zip");
const TEAM_SHA256: &str = "69c6ce94db62052c9b383536874bed0f580bd03ab680876bdc34f56d4604c028";
const ESCAPE: &[u8] = include_bytes!("fixtures/packages/Escape.zip");
const ESCAPE_SHA256: &str = "612cd8e55ade07150e50de33f1a654a0183c24c2614438c4d8c67f715ae6db52";
const BARE: &[u8] = include_bytes!("fixtures/packages/Bare.zip");
const BARE_SHA256: &str = "b558c3748c056d21d89795c2ae42b36b714a9df0c6b964ef2c946eb6201219a0";
const ONLY: &[u8] = include_bytes!("fixtures/packages/Only.zip");
const ONLY_SHA256: &str = "b21447815726d5e21fe2fb72aca781c09c914b1d0e819a53a566f545790f3e7b";

/// A hash no fixture has.
const WRONG_SHA256: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// What the server answers for one path.
#[derive(Clone)]
enum Route {
    /// 200 with this body.
    Body(Vec<u8>),
    /// This status, with no body.
    Status(u16),
    /// 302 to this path.
    Redirect(&'static str),
}

/// A server on a local port that counts the requests it answers.
struct Server {
    address: SocketAddr,
    hits: Arc<AtomicUsize>,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl Server {
    /// Starts a server for `routes`, which answers a path no route names with
    /// 404.
    fn start(routes: Vec<(&'static str, Route)>) -> Result<Self, Box<dyn Error>> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let address = listener.local_addr()?;
        let routes: HashMap<&'static str, Route> = routes.into_iter().collect();
        let hits = Arc::new(AtomicUsize::new(0));
        let stop = Arc::new(AtomicBool::new(false));
        let seen = Arc::clone(&hits);
        let done = Arc::clone(&stop);
        let thread = std::thread::spawn(move || {
            for stream in listener.incoming() {
                if done.load(Ordering::SeqCst) {
                    break;
                }
                if let Ok(stream) = stream {
                    let _ = answer(stream, &routes, &seen);
                }
            }
        });
        Ok(Self {
            address,
            hits,
            stop,
            thread: Some(thread),
        })
    }

    /// The URL of `path` on this server.
    fn url(&self, path: &str) -> String {
        format!("http://{}{path}", self.address)
    }

    /// The requests answered so far.
    fn hits(&self) -> usize {
        self.hits.load(Ordering::SeqCst)
    }

    /// Stops the server, so a later request is refused.
    fn shut_down(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        let _ = TcpStream::connect(self.address);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.shut_down();
    }
}

/// Reads one request from `stream` and writes the answer of its route.
fn answer(
    mut stream: TcpStream,
    routes: &HashMap<&'static str, Route>,
    hits: &AtomicUsize,
) -> std::io::Result<()> {
    let mut request = Vec::new();
    let mut buffer = [0_u8; 1024];
    while !request.windows(4).any(|window| window == b"\r\n\r\n") {
        let read = stream.read(&mut buffer)?;
        if read == 0 {
            return Ok(());
        }
        request.extend_from_slice(&buffer[..read]);
    }
    hits.fetch_add(1, Ordering::SeqCst);
    let text = String::from_utf8_lossy(&request);
    let path = text.split_whitespace().nth(1).unwrap_or("/");
    match routes.get(path) {
        Some(Route::Body(body)) => {
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            )?;
            stream.write_all(body)?;
        }
        Some(Route::Status(code)) => {
            write!(
                stream,
                "HTTP/1.1 {code} Status\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            )?;
        }
        Some(Route::Redirect(to)) => {
            write!(
                stream,
                "HTTP/1.1 302 Found\r\nLocation: {to}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            )?;
        }
        None => {
            write!(
                stream,
                "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            )?;
        }
    }
    stream.flush()
}

/// A project that holds `a.md`, the config with `packages`, and a cache folder.
fn project<'a>(suite: &'a Suite, name: &str, packages: &str) -> Root<'a> {
    let root = suite.scenario(name);
    root.file(
        ".ctl/hakkiri.yaml",
        format!("packages:\n{packages}files:\n  '*.md': {{styles: [Team]}}\n"),
    );
    root.file("a.md", "This is very good.\n");
    root
}

/// The package entry for `url` and `sha256`.
fn entry(url: &str, sha256: &str) -> String {
    format!("  - url: {url}\n    sha256: {sha256}\n")
}

/// Runs `hakkiri` in `root` with its cache in `cache`.
fn hakkiri(root: &Root<'_>, cache: &Path, args: &[&str]) -> Result<common::Run, Box<dyn Error>> {
    let cache = cache.to_str().ok_or("the cache path is not UTF-8")?;
    run(root.path(), args, &[("HAKKIRI_CACHE_DIR", cache)])
}

/// A cache folder inside the suite's temporary tree, apart from the project.
fn cache_folder<'a>(suite: &'a Suite, name: &str) -> Root<'a> {
    suite.scenario(name)
}

#[test]
fn the_first_run_fetches_the_package_and_lints_with_it() -> TestResult {
    let suite = Suite::new("package-first");
    let server = Server::start(vec![("/Team.zip", Route::Body(TEAM.to_vec()))])?;
    let root = project(
        &suite,
        "first",
        &entry(&server.url("/Team.zip"), TEAM_SHA256),
    );
    let cache = cache_folder(&suite, "first-cache");
    let result = hakkiri(&root, cache.path(), &["check"])?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(
        result.lines(),
        [
            "a.md:1:9: warning Team.Filler: Remove 'very'.",
            "1 alert in 1 file"
        ]
    );
    assert_eq!(server.hits(), 1);
    assert!(
        cache
            .path()
            .join(TEAM_SHA256)
            .join("Team/styles/Team/Filler.yml")
            .is_file(),
        "the cache holds the package as the zip laid it out, under the hash"
    );
    Ok(())
}

#[test]
fn a_second_run_reads_the_cache_and_asks_for_nothing() -> TestResult {
    let suite = Suite::new("package-hit");
    let server = Server::start(vec![("/Team.zip", Route::Body(TEAM.to_vec()))])?;
    let root = project(&suite, "hit", &entry(&server.url("/Team.zip"), TEAM_SHA256));
    let cache = cache_folder(&suite, "hit-cache");
    let first = hakkiri(&root, cache.path(), &["check"])?;
    let second = hakkiri(&root, cache.path(), &["check"])?;
    assert_eq!(first.out, second.out);
    assert_eq!(server.hits(), 1, "the second run made no request");
    Ok(())
}

#[test]
fn a_cached_package_lints_with_the_server_gone() -> TestResult {
    let suite = Suite::new("package-offline-hit");
    let mut server = Server::start(vec![("/Team.zip", Route::Body(TEAM.to_vec()))])?;
    let root = project(
        &suite,
        "offline-hit",
        &entry(&server.url("/Team.zip"), TEAM_SHA256),
    );
    let cache = cache_folder(&suite, "offline-hit-cache");
    let first = hakkiri(&root, cache.path(), &["check"])?;
    assert_eq!(first.code, 0, "{}{}", first.out, first.err);
    server.shut_down();
    let second = hakkiri(&root, cache.path(), &["check"])?;
    assert_eq!(second.code, 0, "{}{}", second.out, second.err);
    assert_eq!(first.out, second.out);
    Ok(())
}

#[test]
fn an_empty_cache_and_no_server_is_an_error_that_names_the_url() -> TestResult {
    let suite = Suite::new("package-offline-miss");
    let mut server = Server::start(vec![("/Team.zip", Route::Body(TEAM.to_vec()))])?;
    let url = server.url("/Team.zip");
    server.shut_down();
    let root = project(&suite, "offline-miss", &entry(&url, TEAM_SHA256));
    let cache = cache_folder(&suite, "offline-miss-cache");
    let result = hakkiri(&root, cache.path(), &["check"])?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    assert!(result.err.contains(&url), "{}", result.err);
    assert!(result.out.is_empty(), "{}", result.out);
    Ok(())
}

#[test]
fn a_download_that_does_not_match_its_hash_is_refused_and_not_stored() -> TestResult {
    let suite = Suite::new("package-mismatch");
    let server = Server::start(vec![("/Team.zip", Route::Body(TEAM.to_vec()))])?;
    let url = server.url("/Team.zip");
    let root = project(&suite, "mismatch", &entry(&url, WRONG_SHA256));
    let cache = cache_folder(&suite, "mismatch-cache");
    let result = hakkiri(&root, cache.path(), &["check"])?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    assert!(result.err.contains(&url), "{}", result.err);
    assert!(result.err.contains(WRONG_SHA256), "{}", result.err);
    assert!(result.err.contains(TEAM_SHA256), "{}", result.err);
    assert!(result.out.is_empty(), "{}", result.out);
    assert!(!cache.path().join(WRONG_SHA256).exists());
    assert!(!cache.path().join(TEAM_SHA256).exists());
    Ok(())
}

#[test]
fn a_missing_package_names_the_url_and_the_status() -> TestResult {
    let suite = Suite::new("package-404");
    let server = Server::start(vec![])?;
    let url = server.url("/Nope.zip");
    let root = project(&suite, "missing", &entry(&url, TEAM_SHA256));
    let cache = cache_folder(&suite, "missing-cache");
    let result = hakkiri(&root, cache.path(), &["check"])?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    assert!(result.err.contains(&url), "{}", result.err);
    assert!(result.err.contains("404"), "{}", result.err);
    Ok(())
}

#[test]
fn a_redirect_is_followed() -> TestResult {
    let suite = Suite::new("package-redirect");
    let server = Server::start(vec![
        ("/latest", Route::Redirect("/Team.zip")),
        ("/Team.zip", Route::Body(TEAM.to_vec())),
    ])?;
    let root = project(
        &suite,
        "redirect",
        &entry(&server.url("/latest"), TEAM_SHA256),
    );
    let cache = cache_folder(&suite, "redirect-cache");
    let result = hakkiri(&root, cache.path(), &["check"])?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(server.hits(), 2);
    Ok(())
}

#[test]
fn a_download_that_is_not_a_zip_is_an_error_that_names_the_url() -> TestResult {
    let suite = Suite::new("package-not-zip");
    let body = b"hello\n".to_vec();
    let hash = "5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03";
    let server = Server::start(vec![("/Bad.zip", Route::Body(body))])?;
    let url = server.url("/Bad.zip");
    let root = project(&suite, "not-zip", &entry(&url, hash));
    let cache = cache_folder(&suite, "not-zip-cache");
    let result = hakkiri(&root, cache.path(), &["check"])?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    assert!(result.err.contains(&url), "{}", result.err);
    assert!(result.err.contains("not a zip"), "{}", result.err);
    Ok(())
}

#[test]
fn a_name_that_leaves_the_package_is_refused_and_writes_nothing_outside() -> TestResult {
    let suite = Suite::new("package-escape");
    let server = Server::start(vec![("/Escape.zip", Route::Body(ESCAPE.to_vec()))])?;
    let url = server.url("/Escape.zip");
    let root = project(&suite, "escape", &entry(&url, ESCAPE_SHA256));
    let cache = cache_folder(&suite, "escape-cache");
    let result = hakkiri(&root, cache.path(), &["check"])?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    assert!(result.err.contains(&url), "{}", result.err);
    assert!(result.err.contains("../escape.txt"), "{}", result.err);
    assert!(!cache.path().join("escape.txt").exists());
    assert!(
        !cache
            .path()
            .parent()
            .ok_or("no parent")?
            .join("escape.txt")
            .exists()
    );
    assert!(!cache.path().join(ESCAPE_SHA256).exists());
    Ok(())
}

#[test]
fn a_package_with_no_top_folder_is_an_error_that_names_the_url() -> TestResult {
    let suite = Suite::new("package-bare");
    let server = Server::start(vec![("/Bare.zip", Route::Body(BARE.to_vec()))])?;
    let url = server.url("/Bare.zip");
    let root = project(&suite, "bare", &entry(&url, BARE_SHA256));
    let cache = cache_folder(&suite, "bare-cache");
    let result = hakkiri(&root, cache.path(), &["check"])?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    assert!(result.err.contains(&url), "{}", result.err);
    assert!(result.err.contains("one top folder"), "{}", result.err);
    assert!(!cache.path().join(BARE_SHA256).exists());
    Ok(())
}

#[test]
fn a_style_only_package_loads_its_top_folder_as_a_style() -> TestResult {
    let suite = Suite::new("package-style-only");
    let server = Server::start(vec![("/Only.zip", Route::Body(ONLY.to_vec()))])?;
    let root = project(
        &suite,
        "style-only",
        &entry(&server.url("/Only.zip"), ONLY_SHA256),
    );
    let cache = cache_folder(&suite, "style-only-cache");
    let result = hakkiri(&root, cache.path(), &["check"])?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(
        result.lines(),
        [
            "a.md:1:9: warning Team.Filler: Remove 'very'.",
            "1 alert in 1 file"
        ]
    );
    Ok(())
}

#[test]
fn a_hash_that_is_not_64_hex_digits_is_refused_with_its_line() -> TestResult {
    let suite = Suite::new("package-short-hash");
    let root = project(
        &suite,
        "short-hash",
        &entry("https://example.test/Team.zip", "abc123"),
    );
    let cache = cache_folder(&suite, "short-hash-cache");
    let result = hakkiri(&root, cache.path(), &["check"])?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    assert!(
        result
            .err
            .contains("line 2: packages[0]: `sha256` is 64 lowercase hex digits"),
        "{}",
        result.err
    );
    Ok(())
}

#[test]
fn a_url_that_is_not_http_is_refused_with_its_line() -> TestResult {
    let suite = Suite::new("package-scheme");
    let root = project(
        &suite,
        "scheme",
        &entry("file:///tmp/Team.zip", TEAM_SHA256),
    );
    let cache = cache_folder(&suite, "scheme-cache");
    let result = hakkiri(&root, cache.path(), &["check"])?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    assert!(
        result
            .err
            .contains("line 2: packages[0]: a package `url` starts with `https://` or `http://`"),
        "{}",
        result.err
    );
    Ok(())
}

#[test]
fn hakkiri_test_loads_the_fetched_package_for_a_case_through_the_config() -> TestResult {
    let suite = Suite::new("package-test-command");
    let server = Server::start(vec![("/Team.zip", Route::Body(TEAM.to_vec()))])?;
    let root = project(
        &suite,
        "test-command",
        &entry(&server.url("/Team.zip"), TEAM_SHA256),
    );
    root.file(
        "cases.test.yml",
        "- name: the package flags very\n  input: This is very good.\n  want: \"1:9:Team.Filler:Remove 'very'.\"\n",
    );
    let cache = cache_folder(&suite, "test-command-cache");
    let result = hakkiri(&root, cache.path(), &["test", "cases.test.yml"])?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(server.hits(), 1);
    Ok(())
}

#[test]
fn a_download_over_the_size_limit_is_refused_and_not_stored() -> TestResult {
    let suite = Suite::new("package-too-big");
    let body = vec![0_u8; 33 * 1024 * 1024];
    let server = Server::start(vec![("/Big.zip", Route::Body(body))])?;
    let url = server.url("/Big.zip");
    let root = project(&suite, "too-big", &entry(&url, WRONG_SHA256));
    let cache = cache_folder(&suite, "too-big-cache");
    let result = hakkiri(&root, cache.path(), &["check"])?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    assert!(result.err.contains(&url), "{}", result.err);
    assert!(result.err.contains("32 MiB"), "{}", result.err);
    assert!(!cache.path().join(WRONG_SHA256).exists());
    Ok(())
}

#[test]
fn the_cache_folder_defaults_to_the_per_user_folder() -> TestResult {
    let suite = Suite::new("package-default-cache");
    let server = Server::start(vec![("/Team.zip", Route::Body(TEAM.to_vec()))])?;
    let root = project(
        &suite,
        "default-cache",
        &entry(&server.url("/Team.zip"), TEAM_SHA256),
    );
    let home = suite.scenario("default-cache-home");
    let home_path = home.path().to_str().ok_or("the home path is not UTF-8")?;
    let result = run(
        root.path(),
        &["check"],
        &[("HOME", home_path), ("XDG_CACHE_HOME", "")],
    )?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    let base = if cfg!(target_os = "macos") {
        home.path().join("Library/Caches")
    } else {
        home.path().join(".cache")
    };
    assert!(
        base.join("hakkiri")
            .join(TEAM_SHA256)
            .join("Team/styles")
            .is_dir(),
        "the package is under the per-user cache folder"
    );
    Ok(())
}

#[test]
fn the_cache_dir_option_moves_the_cache() -> TestResult {
    let suite = Suite::new("package-cache-option");
    let server = Server::start(vec![("/Team.zip", Route::Body(TEAM.to_vec()))])?;
    let root = project(
        &suite,
        "cache-option",
        &entry(&server.url("/Team.zip"), TEAM_SHA256),
    );
    let cache = cache_folder(&suite, "cache-option-cache");
    let folder = cache.path().to_str().ok_or("the cache path is not UTF-8")?;
    let result = run(root.path(), &["check", "--cache-dir", folder], &[])?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert!(cache.path().join(TEAM_SHA256).join("Team/styles").is_dir());
    Ok(())
}
