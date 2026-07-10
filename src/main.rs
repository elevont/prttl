// SPDX-FileCopyrightText: 2025 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

use cli::InitError;
use prttl::error::Error;
use std::ffi::OsStr;
use std::rc::Rc;
use thiserror::Error as ThisError;

mod cli;

#[derive(ThisError)]
pub enum CliError {
    #[error("Failed to initialize the CLI tool: {0}")]
    Init(#[from] InitError),

    #[error("Failed to run the formatter: {0}")]
    Format(#[from] Error),
}

impl std::fmt::Debug for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

fn main() -> Result<(), CliError> {
    let (options, src) = cli::init()?;
    let options = Rc::new(options);

    let mut files = Vec::new();
    for source in src {
        if source.is_file() {
            files.push(source);
        } else if source.is_dir() {
            prttl::add_files_with_suffix(&source, OsStr::new("ttl"), &mut files)?;
        } else {
            return Err(Error::TargetFileDoesNotExist(source).into());
        }
    }

    prttl::run(&options, &files)?;
    Ok(())
}
