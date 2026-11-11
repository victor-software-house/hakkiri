//! Tests for the install of a fetched package into the cache.

#![expect(
    clippy::panic_in_result_fn,
    reason = "A test returns Result for `?` and fails through an assertion."
)]

use std::error::Error;
use std::io::{Cursor, Write};
use std::sync::{Arc, Barrier};
use std::{fs, thread};

use ctl_core::fixture::Suite;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use super::install;

type TestResult = Result<(), Box<dyn Error>>;

/// The rules in the package the test installs: enough files that a run is
/// still unpacking when another starts.
const RULES: usize = 400;

/// How many times two installs race, each in a cache of its own.
const ROUNDS: usize = 100;

/// The folder of the package in the cache.
const FOLDER: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

/// A zip of one top folder that holds [`RULES`] rule files.
fn package() -> Result<Vec<u8>, Box<dyn Error>> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    for index in 0..RULES {
        writer.start_file(format!("Team/styles/Team/Rule{index}.yml"), options)?;
        writer.write_all(b"extends: existence\nmessage: x\ntokens: [x]\n")?;
    }
    Ok(writer.finish()?.into_inner())
}

#[test]
fn two_installs_of_one_package_at_once_leave_one_complete_folder() -> TestResult {
    let body = package()?;
    let suite = Suite::new("package-install-race");
    for round in 0..ROUNDS {
        let cache = suite.scenario(&format!("round-{round}"));
        let cache = cache.path().to_path_buf();
        let folder = cache.join(FOLDER);
        let start = Arc::new(Barrier::new(2));
        let runs: Vec<_> = (0..2)
            .map(|_| {
                let (cache, folder, body) = (cache.clone(), folder.clone(), body.clone());
                let start = Arc::clone(&start);
                thread::spawn(move || {
                    start.wait();
                    install(&cache, &folder, "https://example.test/Team.zip", body)
                })
            })
            .collect();
        for run in runs {
            match run.join() {
                Ok(result) => result?,
                Err(_) => return Err("an install panicked".into()),
            }
        }
        let rules = fs::read_dir(folder.join("Team/styles/Team"))?.count();
        assert_eq!(rules, RULES, "round {round}: the folder holds the package");
        let names: Vec<_> = fs::read_dir(&cache)?
            .map(|entry| entry.map(|entry| entry.file_name()))
            .collect::<Result<_, _>>()?;
        assert_eq!(names, [FOLDER], "round {round}: the cache holds no staging");
    }
    Ok(())
}
