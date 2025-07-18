// SPDX-FileCopyrightText: 2021-2025 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

use std::{path::PathBuf, sync::LazyLock};

use clap::{Arg, ArgAction, Command, ValueHint, command, crate_name, value_parser};
use cli_utils::logging;
use const_format::formatcp;
use prttl::options::{FormatOptions, SpecialPredicateOrder, SpecialSubjectTypeOrder};
use thiserror::Error;
use tracing_subscriber::filter::LevelFilter;

pub const A_L_CANONICALIZE: &str = "canonicalize";
// pub const A_S_CANONICALIZE: char = 'C';
pub const A_L_CHECK: &str = "check";
pub const A_S_CHECK: char = 'c';
pub const A_L_FORCE: &str = "force";
pub const A_S_FORCE: char = 'f';
pub const A_L_INDENTATION: &str = "indentation";
pub const A_S_INDENTATION: char = 'i';
// pub const A_L_INPUT: &str = "input";
// pub const A_S_INPUT: char = 'I';
pub const A_L_LABEL_ALL_BLANK_NODES: &str = "label-all-blank-nodes";
pub const A_S_LABEL_ALL_BLANK_NODES: char = 'l';
pub const A_L_NO_PRTR_SORTING: &str = "no-prtr-sorting";
// pub const A_S_NO_PRTR_SORTING: char = 'p';
pub const A_L_NO_SPARQL_SYNTAX: &str = "no-sparql-syntax";
// pub const A_S_NO_SPARQL_SYNTAX: char = 's';
pub const A_L_PREDICATE_ORDER: &str = "pred-order";
pub const A_L_PREDICATE_ORDER_PRESET: &str = "pred-order-preset";
pub const A_L_SINGLE_LEAFED_NEW_LINES: &str = "single-leafed-new-lines";
pub const A_S_SINGLE_LEAFED_NEW_LINES: char = 'n';
pub const A_L_SUBJECT_TYPE_ORDER: &str = "subj-type-order";
pub const A_L_SUBJECT_TYPE_ORDER_PRESET: &str = "subj-type-order-preset";
pub const A_L_QUIET: &str = "quiet";
pub const A_S_QUIET: char = 'q';
pub const A_L_VERBOSE: &str = "verbose";
pub const A_S_VERBOSE: char = 'v';
pub const A_L_VERSION: &str = "version";
pub const A_S_VERSION: char = 'V';
pub const A_L_SRC: &str = "src";

pub const DEFAULT_INDENTATION: u8 = 2;
static DEFAULT_INDENTATION_STR: LazyLock<String> =
    LazyLock::new(|| DEFAULT_INDENTATION.to_string());

// /// File(s) or directory to format.
// #[arg()]
// src: Vec<PathBuf>,

fn arg_canonicalize() -> Arg {
    Arg::new(A_L_CANONICALIZE)
        .help("Whether to canonicalize the input before formatting")
        .long_help(
            "Whether to canonicalize the input before formatting. \
This refers to <https://www.w3.org/TR/rdf-canon/>, \
and effectively just label the blank nodes in a uniform way.",
        )
        .action(ArgAction::SetTrue)
        // .short(A_S_CANONICALIZE)
        .long(A_L_CANONICALIZE)
}

fn arg_check() -> Arg {
    Arg::new(A_L_CHECK)
        .help(
            "Do not edit the file but only check \
if it already applies this tools format",
        )
        .action(ArgAction::SetTrue)
        .short(A_S_CHECK)
        .long(A_L_CHECK)
}

fn arg_force() -> Arg {
    Arg::new(A_L_FORCE)
        .help(
            "Forces overwriting of the output file, \
if it already exists, which includes the case of the input and output file \
being equal",
        )
        // Whether to force-write the output,
        // even if potential issues with the formatting have been detected.
        .action(ArgAction::SetTrue)
        .short(A_S_FORCE)
        .long(A_L_FORCE)
}

fn arg_label_all_blank_nodes() -> Arg {
    Arg::new(A_L_LABEL_ALL_BLANK_NODES)
        .help(
            "Whether to disable sorting of blank nodes \
using their `prtr:sortingId` value, if any",
        )
        .long_help(
            "Whether to use labels for all blank nodes, \
or rather maximize nesting of them. \
 \
NOTE That blank nodes referenced in more then one place can never be nested.",
        )
        .action(ArgAction::SetTrue)
        .short(A_S_LABEL_ALL_BLANK_NODES)
        .long(A_L_LABEL_ALL_BLANK_NODES)
}

fn arg_indentation() -> Arg {
    Arg::new(A_L_INDENTATION)
        .help("Number of spaces per level of indentation")
        .num_args(1)
        .short(A_S_INDENTATION)
        .long(A_L_INDENTATION)
        .action(ArgAction::Set)
        // .value_hint(ValueHint::Other)
        .value_name("NUM")
        .value_parser(value_parser!(u8).range(1..))
        .default_value(DEFAULT_INDENTATION_STR.as_str())
}

// fn arg_input() -> Arg {
//     Arg::new(A_L_INPUT)
//         .help("an input RDF file to pretty print to Turtle; '-' for stdin")
//         .num_args(1)
//         .short(A_S_INPUT)
//         .long(A_L_INPUT)
//         .action(ArgAction::Set)
//         .value_hint(ValueHint::FilePath)
//         .value_name("FILE")
//         .default_value("-")
// }

fn arg_no_prtr_sorting() -> Arg {
    Arg::new(A_L_NO_PRTR_SORTING)
        .help(
            "Whether to disable sorting of blank nodes \
using their `prtr:sortingId` value, if any",
        )
        .long_help(
            "Whether to disable sorting of blank nodes \
using their `prtr:sortingId` value, if any. \
\
[`prtr`](https://codeberg.org/elevont/prtr) \
is an ontology concerned with \
[RDF Pretty Printing](https://www.w3.org/DesignIssues/Pretty.html).",
        )
        .action(ArgAction::SetTrue)
        // .short(A_S_NO_PRTR_SORTING)
        .long(A_L_NO_PRTR_SORTING)
}

fn arg_no_sparql_syntax() -> Arg {
    Arg::new(A_L_NO_SPARQL_SYNTAX)
        .help(
            "Whether to use SPARQL-ish syntax for base and prefix, \
or the traditional Turtle syntax",
        )
        .long_help(
            "Whether to use SPARQL-ish syntax for base and prefix, \
or the traditional Turtle syntax. \
 \
- SPARQL-ish: \
 \
```turtle \
BASE <http://example.com/> \
PREFIX foaf: <http://xmlns.com/foaf/0.1/> \
``` \
 \
- Traditional Turtle: \
 \
```turtle \
@base <http://example.com/> . \
@prefix foaf: <http://xmlns.com/foaf/0.1/> . \
``` \
",
        )
        .action(ArgAction::SetTrue)
        // .short(A_S_NO_SPARQL_SYNTAX)
        .long(A_L_NO_SPARQL_SYNTAX)
}

fn arg_predicate_order() -> Arg {
    Arg::new(A_L_PREDICATE_ORDER)
        .help(
            "Sets a custom order of predicates to be used for sorting.
Predicates that match come first; in the provided order.
Predicates that do not match come afterwards; in alphabetical order.

You may specify predicate names as absolute IRIs or as prefixed names.
Only direct matches are considered; meaning: No type inference is conducted.",
        )
        .long(A_L_PREDICATE_ORDER)
        .num_args(0..)
        .value_name("PREDICATE")
        .value_hint(ValueHint::Other)
        .action(ArgAction::Append)
}

fn arg_predicate_order_preset() -> Arg {
    Arg::new(A_L_PREDICATE_ORDER_PRESET)
        .help(
            "Sets a predefined order of predicates to be used for sorting.
Predicates that match come first; in the provided order.
Predicates that do not match come afterwards; in alphabetical order.

You may specify predicate names as absolute IRIs or as prefixed names.
Only direct matches are considered; meaning: No type inference is conducted.",
        )
        .long(A_L_PREDICATE_ORDER_PRESET)
        .value_name("PREDICATE_ORDER_PRESET")
        .value_parser(value_parser!(SpecialPredicateOrder))
        .action(ArgAction::Set)
}

fn arg_single_entry_on_new_line() -> Arg {
    Arg::new(A_L_SINGLE_LEAFED_NEW_LINES)
        .help("Whether to move a single/lone predicate-object pair or object alone onto a new line")
        .action(ArgAction::SetTrue)
        .short(A_S_SINGLE_LEAFED_NEW_LINES)
        .long(A_L_SINGLE_LEAFED_NEW_LINES)
}

fn arg_subject_type_order() -> Arg {
    Arg::new(A_L_SUBJECT_TYPE_ORDER)
        .help(
            "Sets a custom order of subject types to be used for sorting.
Subjects with a matching type come first; in the provided order.
Subjects without any matching type come afterwards; in alphabetical order.

You may specify subject type names as absolute IRIs or as prefixed names.
Only direct matches are considered; meaning: No type inference is conducted.",
        )
        .long(A_L_SUBJECT_TYPE_ORDER)
        .num_args(0..)
        .value_name("SUBJECT_TYPE")
        .value_hint(ValueHint::Other)
        .action(ArgAction::Append)
}

fn arg_subject_type_order_preset() -> Arg {
    Arg::new(A_L_SUBJECT_TYPE_ORDER_PRESET)
        .help(
            "Sets a predefined order of subject types to be used for sorting.
Subjects with a matching type come first; in the provided order.
Subjects without any matching type come afterwards; in alphabetical order.

Only direct matches are considered; meaning: No type inference is conducted.",
        )
        .long(A_L_SUBJECT_TYPE_ORDER_PRESET)
        .value_name("SUBJECT_TYPE_ORDER_PRESET")
        .value_parser(value_parser!(SpecialSubjectTypeOrder))
        .action(ArgAction::Set)
}

fn arg_quiet() -> Arg {
    Arg::new(A_L_QUIET)
        .help("Minimize or suppress output to stdout")
        .long_help("Minimize or suppress output to stdout, and only shows log output on stderr.")
        .action(ArgAction::SetTrue)
        .short(A_S_QUIET)
        .long(A_L_QUIET)
        .conflicts_with(A_L_VERBOSE)
}

fn arg_verbose() -> Arg {
    Arg::new(A_L_VERBOSE)
        .help("more verbose output (useful for debugging)")
        .short(A_S_VERBOSE)
        .long(A_L_VERBOSE)
        .action(ArgAction::SetTrue)
}

fn arg_version() -> Arg {
    Arg::new(A_L_VERSION)
        .help(formatcp!(
            "Print version information and exit. \
May be combined with -{A_S_QUIET},--{A_L_QUIET}, \
to really only output the version string."
        ))
        .short(A_S_VERSION)
        .long(A_L_VERSION)
        .action(ArgAction::SetTrue)
}

fn arg_src() -> Arg {
    Arg::new(A_L_SRC)
        .help("Source RDF file(s) or director(y|ies) containing Turtle files to format")
        .num_args(1..)
        .value_name("FILE_OR_DIR")
        .value_hint(ValueHint::Other)
        .value_parser(value_parser!(PathBuf))
        .required(true)
        .action(ArgAction::Set)
}

fn args_matcher() -> Command {
    command!()
        .about("Pretty prints RDF/Turtle files")
        .long_about(
            "Takes RDF data as input (commonly a Turtle file), \
and generates diff optimized RDF/Turtle, \
using a lot of new-lines. \
 \
One peculiarity of this tool is, \
that it removes (Turtle-syntax) comments. \
We do this, because we believe that all comments \
should rather be encoded into triples, \
and we celebrate this in our own data, \
specifically our ontologies. \
More about this: \
<https://codeberg.org/elevont/cmt-ont>",
        )
        .bin_name(clap::crate_name!())
        .help_expected(true)
        .disable_version_flag(true)
        .arg(arg_canonicalize())
        .arg(arg_check())
        .arg(arg_force())
        .arg(arg_label_all_blank_nodes())
        .arg(arg_indentation())
        // .arg(arg_input())
        // .arg(arg_output())
        .arg(arg_no_prtr_sorting())
        .arg(arg_no_sparql_syntax())
        .arg(arg_predicate_order())
        .arg(arg_predicate_order_preset())
        .arg(arg_single_entry_on_new_line())
        .arg(arg_subject_type_order())
        .arg(arg_subject_type_order_preset())
        .arg(arg_quiet())
        .arg(arg_verbose())
        .arg(arg_version())
        .arg(arg_src())
}

#[allow(clippy::print_stdout)]
fn print_version_and_exit(quiet: bool) {
    if !quiet {
        print!("{} ", clap::crate_name!());
    }
    println!("{}", prttl::VERSION);
    std::process::exit(0);
}

#[derive(Error, Debug)]
pub enum InitError {
    #[error("Failed to init logging system: {0}")]
    LogInit(#[from] tracing_subscriber::util::TryInitError),

    #[error("Failed to change the logging level: {0}")]
    LogChangeLevel(#[from] tracing_subscriber::reload::Error),
}

pub fn init() -> Result<(FormatOptions, Vec<PathBuf>), InitError> {
    let log_reload_handle = logging::setup(crate_name!())?;
    let args = args_matcher().get_matches();

    let quiet = args.get_flag(A_L_QUIET);
    let version = args.get_flag(A_L_VERSION);
    if version {
        print_version_and_exit(quiet);
    }

    let verbose = args.get_flag(A_L_VERBOSE);
    let log_level = if verbose {
        LevelFilter::TRACE
    } else if quiet {
        LevelFilter::WARN
    } else {
        LevelFilter::INFO
    };
    logging::set_log_level_tracing(&log_reload_handle, log_level)?;

    let canonicalize = args.get_flag(A_L_CANONICALIZE);
    let check = args.get_flag(A_L_CHECK);
    let force = args.get_flag(A_L_FORCE);
    let indentation_spaces = args
        .get_one::<u8>(A_L_INDENTATION)
        .copied()
        .unwrap_or(DEFAULT_INDENTATION)
        .into();
    let max_nesting = !args.get_flag(A_L_LABEL_ALL_BLANK_NODES);
    let prtr_sorting = !args.get_flag(A_L_NO_PRTR_SORTING);
    let sparql_syntax = !args.get_flag(A_L_NO_SPARQL_SYNTAX);
    let single_leafed_new_lines = args.get_flag(A_L_SINGLE_LEAFED_NEW_LINES);
    let warn_unsupported_numbers = true;

    let predicate_order: Option<Vec<String>> = args
        .get_many::<String>(A_L_PREDICATE_ORDER)
        .map(|vals| vals.cloned().collect());
    let predicate_order_preset: Option<SpecialPredicateOrder> = args
        .get_one::<SpecialPredicateOrder>(A_L_PREDICATE_ORDER_PRESET)
        .copied();
    let subject_type_order: Option<Vec<String>> = args
        .get_many::<String>(A_L_SUBJECT_TYPE_ORDER)
        .map(|vals| vals.cloned().collect());
    let subject_type_order_preset: Option<SpecialSubjectTypeOrder> = args
        .get_one::<SpecialSubjectTypeOrder>(A_L_SUBJECT_TYPE_ORDER_PRESET)
        .copied();

    let indentation = " ".repeat(indentation_spaces);
    let src: Vec<PathBuf> = args
        .get_many::<PathBuf>(A_L_SRC)
        .unwrap()
        .cloned()
        .collect();
    Ok((
        FormatOptions {
            check,
            indentation,
            single_leafed_new_lines,
            force,
            prtr_sorting,
            sparql_syntax,
            max_nesting,
            canonicalize,
            warn_unsupported_numbers,
            subject_type_order_preset,
            subject_type_order,
            predicate_order_preset,
            predicate_order,
        },
        src,
    ))
}
