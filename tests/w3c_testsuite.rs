// SPDX-FileCopyrightText: 2023 Helsing GmbH
// SPDX-FileCopyrightText: 2025 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

use oxrdf::graph::CanonicalizationAlgorithm;
use oxrdf::vocab::rdf;
use oxrdf::{Graph, NamedNodeRef, NamedOrBlankNodeRef, TermRef};
use oxttl::TurtleParser;
use prttl::{
    error::Error as FmtError, formatter::format, options::FormatOptions, parser,
    parser::Error as ParsingError,
};
use std::rc::Rc;
use std::sync::LazyLock;
use std::{fs, str};
use thiserror::Error;

static NN_W3C_TESTS_ACTION: LazyLock<NamedNodeRef> = LazyLock::new(|| {
    NamedNodeRef::new("http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#action").unwrap()
});
static NN_W3_RDFTEST_APPROVAL: LazyLock<NamedNodeRef> =
    LazyLock::new(|| NamedNodeRef::new("http://www.w3.org/ns/rdftest#approval").unwrap());
static NN_W3_RDFTEST_APPROVED: LazyLock<NamedNodeRef> =
    LazyLock::new(|| NamedNodeRef::new("http://www.w3.org/ns/rdftest#Approved").unwrap());

const STR_W3_RDFTEST_TEST_TURTLE_EVAL: &str = "http://www.w3.org/ns/rdftest#TestTurtleEval";
const STR_W3_RDFTEST_TEST_TURTLE_POSITIVE_SYNTAX: &str =
    "http://www.w3.org/ns/rdftest#TestTurtlePositiveSyntax";
const STR_W3_RDFTEST_TEST_TURTLE_NEGATIVE_SYNTAX: &str =
    "http://www.w3.org/ns/rdftest#TestTurtleNegativeSyntax";
static NN_W3_RDFTEST_TEST_TURTLE_EVAL: LazyLock<NamedNodeRef> =
    LazyLock::new(|| NamedNodeRef::new(STR_W3_RDFTEST_TEST_TURTLE_EVAL).unwrap());
static NN_W3_RDFTEST_TEST_TURTLE_POSITIVE_SYNTAX: LazyLock<NamedNodeRef> =
    LazyLock::new(|| NamedNodeRef::new(STR_W3_RDFTEST_TEST_TURTLE_POSITIVE_SYNTAX).unwrap());
static NN_W3_RDFTEST_TEST_TURTLE_NEGATIVE_SYNTAX: LazyLock<NamedNodeRef> =
    LazyLock::new(|| NamedNodeRef::new(STR_W3_RDFTEST_TEST_TURTLE_NEGATIVE_SYNTAX).unwrap());

#[derive(Error, Debug)]
pub enum Error {
    #[error("No type for test subject {0}")]
    NoType(String),

    #[error("No action for test subject {0}")]
    NoAction(String),

    #[error(
        "The formatted graph for subject '{0}' is not the same as the expected graph.\nOriginal:\n'{1}'\n\nFormatted:\n'{2}'"
    )]
    DiffersFromExpected(String, Graph, Graph),

    #[error(
        "The formatting for subject '{0}' is not stable.\nFormatted:\n'{1}'\n\nReformatted:\n'{2}'"
    )]
    UnstableFormatting(String, String, String),

    #[error(
        "The content for subject '{0}' has been parsed without error, even though it should fail!\nContent:\n'{1}'"
    )]
    FalsePositive(String, String),

    #[error("Failed to parse content as turtle: {0}\ncontent:\n'{1}'")]
    Parsing(#[source] ParsingError, String),

    #[error("Failed to format content as turtle: {0}\ncontent:\n'{1}'")]
    Formatting(#[source] FmtError, String),

    #[error("Failed to read a file: {0}")]
    IO(#[from] std::io::Error),
}

fn format_turtle(original: &str, options: FormatOptions) -> Result<String, Error> {
    let options = Rc::new(options);
    let input = parser::parse(original.as_bytes(), &options)
        .map_err(|parser_err| Error::Parsing(parser_err, original.to_owned()))?;
    format(&input, options).map_err(|fmt_err| Error::Formatting(fmt_err, original.to_owned()))
}

// fn get_remote_file(url: &str) -> BoxResult<String> {
//     let mut hasher = DefaultHasher::new();
//     hasher.write(url.as_bytes());
//     let cache_path = Path::new(CACHE).join(hasher.finish().to_string());
//     if cache_path.exists() {
//         return Ok(fs::read_to_string(cache_path)?);
//     }

//     let content = reqwest::blocking::get(url)?.error_for_status()?.text()?;
//     fs::write(cache_path, &content)?;
//     Ok(content)
// }

fn parse_turtle(base: &str, data: &str) -> Result<Graph, ParsingError> {
    TurtleParser::new()
        // .with_quoted_triples()
        .with_base_iri(base)
        .map_err(ParsingError::BaseIri)?
        .for_slice(data.as_bytes())
        .collect::<core::result::Result<_, _>>()
        .map_err(std::convert::Into::into)
}

fn options() -> FormatOptions {
    FormatOptions {
        check: false,
        indentation: "  ".to_string(),
        single_leafed_new_lines: false,
        force: true,
        prtr_sorting: false,
        sparql_syntax: false,
        max_nesting: true,
        canonicalize: true,
        warn_unsupported_numbers: true,
        subject_type_order_preset: None,
        subject_type_order: None,
        predicate_order_preset: None,
        predicate_order: None,
    }
}

fn run_test(
    manifest_dir_url: &str,
    manifest_local_path_dir: &str,
    test: NamedOrBlankNodeRef<'_>,
    manifest: &Graph,
) -> Result<(), Error> {
    let Some(TermRef::NamedNode(test_type)) =
        manifest.object_for_subject_predicate(test, rdf::TYPE)
    else {
        return Err(Error::NoType(test.to_string()));
    };

    let Some(TermRef::NamedNode(input_url)) =
        manifest.object_for_subject_predicate(test, *NN_W3C_TESTS_ACTION)
    else {
        return Err(Error::NoAction(test.to_string()));
    };
    let input_local_path = input_url
        .as_str()
        .replace(manifest_dir_url, manifest_local_path_dir);
    let original = fs::read_to_string(&input_local_path)?.replace('\0', "");
    let formatted_result = format_turtle(&original, options());

    match test_type.as_str() {
        STR_W3_RDFTEST_TEST_TURTLE_EVAL | STR_W3_RDFTEST_TEST_TURTLE_POSITIVE_SYNTAX => {
            let formatted = formatted_result?;
            let input_url = "http://a.a".to_string(); // TODO HACK FIXME
            let mut original_graph = parse_turtle(input_url.as_str(), &original)
                .map_err(|parser_err| Error::Parsing(parser_err, original.clone()))?;
            original_graph.canonicalize(CanonicalizationAlgorithm::Unstable);
            let mut formatted_graph = parse_turtle(input_url.as_str(), &formatted)
                .map_err(|parser_err| Error::Parsing(parser_err, formatted.clone()))?;
            formatted_graph.canonicalize(CanonicalizationAlgorithm::Unstable);
            if original_graph != formatted_graph {
                return Err(Error::DiffersFromExpected(
                    test.to_string(),
                    original_graph,
                    formatted_graph,
                ));
            }
            let reformatted = format_turtle(&formatted, FormatOptions::default())?;
            if formatted != reformatted {
                return Err(Error::UnstableFormatting(
                    test.to_string(),
                    formatted,
                    reformatted,
                ));
            }
            Ok(())
        }
        STR_W3_RDFTEST_TEST_TURTLE_NEGATIVE_SYNTAX => {
            if formatted_result.is_ok() {
                return Err(Error::FalsePositive(test.to_string(), original));
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn test_w3c_test_suite(
    manifest_dir_url: &str,
    manifest_local_path_dir: &str,
    approved_only: bool,
) -> Result<(), Error> {
    let manifest_url = format!("{manifest_dir_url}/manifest.ttl");
    let manifest_local_path = format!("{manifest_local_path_dir}/manifest.ttl");
    let manifest_content = fs::read_to_string(manifest_local_path)?;
    let manifest_rdf = parse_turtle(&manifest_url, &manifest_content)
        .map_err(|parser_err| Error::Parsing(parser_err, manifest_content.clone()))?;
    let errors = if approved_only {
        manifest_rdf
            .subjects_for_predicate_object(*NN_W3_RDFTEST_APPROVAL, *NN_W3_RDFTEST_APPROVED)
            .chain(vec![])
            .collect::<Vec<_>>()
    } else {
        manifest_rdf
            .subjects_for_predicate_object(rdf::TYPE, *NN_W3_RDFTEST_TEST_TURTLE_EVAL)
            .chain(manifest_rdf.subjects_for_predicate_object(
                rdf::TYPE,
                *NN_W3_RDFTEST_TEST_TURTLE_POSITIVE_SYNTAX,
            ))
            .chain(manifest_rdf.subjects_for_predicate_object(
                rdf::TYPE,
                *NN_W3_RDFTEST_TEST_TURTLE_NEGATIVE_SYNTAX,
            ))
            .collect::<Vec<_>>()
    }
    .into_iter()
    .filter_map(|t| {
        run_test(manifest_dir_url, manifest_local_path_dir, t, &manifest_rdf)
            .err()
            .map(|err| err.to_string())
    })
    .collect::<Vec<_>>();
    // let errors = manifest_rdf
    //     .subjects_for_predicate_object(*NN_W3_RDFTEST_APPROVAL, *NN_W3_RDFTEST_APPROVED)
    //     .filter_map(|t| {
    //         run_test(manifest_dir_url, manifest_local_path_dir, t, &manifest_rdf)
    //             .err()
    //             .map(|err| err.to_string())
    //     })
    //     .collect::<Vec<_>>();
    assert!(errors.is_empty(), "{}", errors.join("\n"));
    Ok(())
}

// #[test]
fn test_w3c_modern_rdf11() -> Result<(), Error> {
    test_w3c_test_suite(
        "http://w3c.github.io/rdf-tests/rdf/rdf11/rdf-turtle",
        "tests/data/input/w3c/modern/rdf11",
        true,
    )
}

// #[test]
fn test_w3c_modern_rdf12_eval() -> Result<(), Error> {
    test_w3c_test_suite(
        "http://w3c.github.io/rdf-tests/rdf/rdf12/rdf-turtle/eval",
        "tests/data/input/w3c/modern/rdf12/eval",
        false,
    )
}

// #[test]
fn test_w3c_modern_rdf12_syntax() -> Result<(), Error> {
    test_w3c_test_suite(
        "http://w3c.github.io/rdf-tests/rdf/rdf12/rdf-turtle/syntax",
        "tests/data/input/w3c/modern/rdf12/syntax",
        false,
    )
}
