// SPDX-FileCopyrightText: 2025 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

use crate::parser;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug)]
pub enum FilesListErrorType {
    ReadDir,
    ExtractEntry,
    EvaluateFileType,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Input is not equivalent to the (re-)formatted version of its self: {0}")]
    Check(String),

    #[error(
        "We do not support more then one base IRI defined per file.
Consider refactoring the input first."
    )]
    MultipleBases,

    #[error(transparent)]
    TurtleSyntaxError(#[from] oxttl::TurtleSyntaxError),

    #[error(transparent)]
    Format(#[from] std::fmt::Error),

    #[error("The target file to format does not seem to exist: '{0}'")]
    TargetFileDoesNotExist(PathBuf),

    #[error("Error while reading file: '{0}'")]
    FailedToReadTargetFile(PathBuf),

    #[error("Failed to parse input as turtle: {0}")]
    ParseError(#[from] parser::Error),

    #[error("Error while writing to file: '{0}'")]
    FailedToWriteFormattedFile(#[source] std::io::Error, PathBuf),

    #[error("Failed to list files in input directory '{0}': {1:?}")]
    FailedToListFilesInInputDir(#[source] std::io::Error, PathBuf, FilesListErrorType),

    #[error("Failed to create Turtle file tree structure: {0}")]
    FailedToCreateTurtleStructure(String),
}

pub type FmtResult<T> = std::result::Result<T, Error>;
