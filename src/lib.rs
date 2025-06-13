// SPDX-FileCopyrightText: 2025 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

use crate::error::{Error, FilesListErrorType};
use crate::{formatter::format, options::FormatOptions};
use diffy::{create_patch, PatchFormatter};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use git_version::git_version;

pub mod ast;
pub mod compare;
pub mod context;
pub mod error;
pub mod formatter;
pub mod input;
pub mod options;
pub mod parser;
pub mod vocab;

// This tests rust code in the README with doc-tests.
// Though, It will not appear in the generated documentation.
#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;

pub const VERSION: &str = git_version!(cargo_prefix = "", fallback = "unknown");

/// Runs the formatter on the given files.
///
/// # Errors
///
/// Any error from [`Error`].
pub fn run(options: &Rc<FormatOptions>, input_files: &Vec<PathBuf>) -> Result<(), Error> {
    for file in input_files {
        let original =
            fs::read_to_string(file).map_err(|_err| Error::FailedToReadTargetFile(file.clone()))?;
        let input = parser::parse(original.as_bytes(), options)?;
        let formatted = format(&input, Rc::<_>::clone(options))?;
        if original == formatted {
            // Nothing to do
            continue;
        }
        if options.check {
            let patch = create_patch(&original, &formatted);
            let formatted_patch = PatchFormatter::new()
                .with_color()
                .fmt_patch(&patch)
                .to_string();
            return Err(Error::Check(formatted_patch));
        }
        fs::write(file, formatted)
            .map_err(|err| Error::FailedToWriteFormattedFile(err, file.clone()))?;
    }
    Ok(())
}

/// Recursively adds files from a directory,
/// which have the given suffix,
/// to a list of files given as parameter.
///
/// # Errors
///
/// - if the directory does not exist
/// - if the directory is not a directory
/// - if the directory is not readable (an issue with file-system permissions)
pub fn add_files_with_suffix(
    dir: &Path,
    extension: &OsStr,
    files: &mut Vec<PathBuf>,
) -> Result<(), Error> {
    for entry in fs::read_dir(dir).map_err(|err| {
        Error::FailedToListFilesInInputDir(err, dir.to_path_buf(), FilesListErrorType::ReadDir)
    })? {
        let entry = entry.map_err(|err| {
            Error::FailedToListFilesInInputDir(
                err,
                dir.to_path_buf(),
                FilesListErrorType::ExtractEntry,
            )
        })?;
        let entry_type = entry.file_type().map_err(|err| {
            Error::FailedToListFilesInInputDir(
                err,
                dir.to_path_buf(),
                FilesListErrorType::EvaluateFileType,
            )
        })?;
        if entry_type.is_file() {
            let file = entry.path();
            if file.extension() == Some(extension) {
                files.push(file);
            }
        } else if entry_type.is_dir() {
            add_files_with_suffix(&entry.path(), extension, files)?;
        }
    }
    Ok(())
}
