// SPDX-FileCopyrightText: 2022 Helsing GmbH
//
// SPDX-License-Identifier: Apache-2.0

// use anyhow::{bail, Context, Result};
use clap::Parser;
use diffy::{create_patch, PatchFormatter};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::rc::Rc;
use turtlefmt::formatter::{Error, FilesListErrorType};
use turtlefmt::parser;
use turtlefmt::{formatter::format, options::FormatOptions};

/// Apply a consistent formatting to a Turtle file
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// File(s) or directory to format.
    #[arg()]
    src: Vec<PathBuf>,
    /// Do not edit the file but only check if it already applies this tools format.
    #[arg(long)]
    check: bool,
    /// Number of spaces per level of indentation
    #[arg(long, default_value = "2")]
    indentation: usize,
    // /// Whether to apply formatting options that try to minimize diffs
    // /// between different versions of the same file.
    // /// This additionally sorts subjects, predicates and objects,
    // /// and it puts each of those onto a new line.
    // ///
    // /// This might be useful if the file is stored on an SCM like git,
    // /// and you can ensure that this tool is applied before each commit.
    // ///
    // /// NOTE: This (because of how the sorting works)
    // ///       does not play well with comments;
    // ///       We thus recommend to only use this
    // ///       if you are not using comments,
    // ///       or if you convert the comments into RDF triples.
    // #[arg(long)]
    // diff_optimized: bool,
    // /// Whether to cleanup/unify empty lines used as dividers.
    // /// This ensures that there is exactly one empty line
    // /// before and after each subject,
    // /// and that there is none anywhere else.
    // #[arg(long)]
    // cleanup_dividing_empty_lines: bool,
    /// Whether to force-write the output,
    /// even if potential issues with the formatting have been detected.
    #[arg(long)]
    force: bool,
    /// Whether to disable sorting of blank nodes
    /// using their `prtyr:sortingId` value, if any.
    ///
    /// [`prtyr`](https://codeberg.org/elevont/prtyr)
    /// is an ontology concerned with
    /// [RDF Pretty Printing](https://www.w3.org/DesignIssues/Pretty.html).
    #[arg(long)]
    no_prtyr_sorting: bool,
    /// Whether to use SPARQL-ish syntax for base and prefix,
    /// or the traditional Turtle syntax.
    ///
    /// - SPARQL-ish:
    ///
    ///   ```turtle
    ///   BASE <http://example.com/>
    ///   PREFIX foaf: <http://xmlns.com/foaf/0.1/>
    ///   ```
    ///
    /// - Traditional Turtle:
    ///
    ///   ```turtle
    ///   @base <http://example.com/> .
    ///   @prefix foaf: <http://xmlns.com/foaf/0.1/> .
    ///   ```
    #[arg(long)]
    pub sparql_syntax: bool,
    /// Whether to use labels for all blank nodes,
    /// or rather maximize nesting of them.
    ///
    /// NOTE That blank nodes referenced in more then one place can never be nested.
    #[arg(long)]
    pub label_all_blank_nodes: bool,
    /// Whether to canonicalize the input before formatting.
    /// This refers to <https://www.w3.org/TR/rdf-canon/>,
    /// and effectively just label the blank nodes in a uniform way.
    #[arg(long)]
    pub canonicalize: bool,
}

impl From<&Args> for FormatOptions {
    fn from(args: &Args) -> Self {
        let indentation = " ".repeat(args.indentation);
        let force = args.force;
        let prtyr_sorting = !args.no_prtyr_sorting;
        let sparql_syntax = args.sparql_syntax;
        let max_nesting = !args.label_all_blank_nodes;
        let canonicalize = args.canonicalize;
        Self {
            indentation,
            single_object_on_new_line: false,
            force,
            prtyr_sorting,
            sparql_syntax,
            max_nesting,
            canonicalize,
        }
    }
}

fn main() -> Result<ExitCode, Error> {
    let args = Args::parse();
    let options: Rc<FormatOptions> = Rc::new((&args).into());
    let mut exit_code = ExitCode::SUCCESS;

    let mut files = Vec::new();
    for source in args.src {
        if source.is_file() {
            files.push(source);
        } else if source.is_dir() {
            add_files_with_suffix(&source, OsStr::new("ttl"), &mut files)?;
        } else {
            return Err(Error::TargetFileDoesNotExist(source));
        }
    }

    for file in files {
        let original = fs::read_to_string(&file)
            .map_err(|_err| Error::FailedToReadTargetFile(file.clone()))?;
        let input = parser::parse(original.as_bytes(), &options)?;
        let formatted = format(&input, Rc::<_>::clone(&options))?;
        if original == formatted {
            // Nothing to do
            continue;
        }
        if args.check {
            let patch = create_patch(&original, &formatted);
            eprintln!("The format of {} is not correct", file.display());
            println!("{}", PatchFormatter::new().with_color().fmt_patch(&patch));
            exit_code = ExitCode::from(65);
        } else {
            fs::write(&file, formatted).map_err(|err| Error::FailedToWriteFormattedFile(file))?;
        }
    }
    Ok(exit_code)
}

fn add_files_with_suffix(
    dir: &Path,
    extension: &OsStr,
    files: &mut Vec<PathBuf>,
) -> Result<(), Error> {
    for entry in fs::read_dir(dir).map_err(|err| {
        Error::FailedToListFilesInInputDir(dir.to_path_buf(), FilesListErrorType::ReadDir)
    })? {
        let entry = entry.map_err(|err| {
            Error::FailedToListFilesInInputDir(dir.to_path_buf(), FilesListErrorType::ExtractEntry)
        })?;
        let entry_type = entry.file_type().map_err(|err| {
            Error::FailedToListFilesInInputDir(
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
