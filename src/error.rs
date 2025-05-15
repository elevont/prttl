// SPDX-FileCopyrightText: 2022 Helsing GmbH
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
    // #[error("Failed to initialize the CLI tool: {0}")]
    // Init(#[from] InitError),
    #[error("Input is not equivalent to the (re-)formatted version of its self: {0}")]
    Check(String),

    #[error("We do not support redefinition of prefixes, which is the case with {0}")]
    PrefixRedefinition(String),

    #[error("We do not support more then one base IRI defined per file")]
    MultipleBases,

    #[error(transparent)]
    TurtleSyntaxError(#[from] oxttl::TurtleSyntaxError),

    /// Represents all cases of `std::io::Error`.
    #[error(transparent)]
    Format(#[from] std::fmt::Error),

    #[error("The target to format {0} does not seem to exist")]
    TargetFileDoesNotExist(PathBuf),

    #[error("Error while reading {0}")]
    FailedToReadTargetFile(PathBuf),

    #[error("Failed to parse input as turtle: {0}")]
    ParseError(#[from] parser::Error),

    #[error("Error while writing {0}")]
    FailedToWriteFormattedFile(PathBuf),

    #[error("Failed to list files in input directory {0}: {1:?}")]
    FailedToListFilesInInputDir(PathBuf, FilesListErrorType),

    #[error("Failed to create Turtle file tree structure: {0}")]
    FailedToCreateTurtleStructure(String),
}

pub type FmtResult<T> = std::result::Result<T, Error>;
