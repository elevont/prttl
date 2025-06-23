# `prttl` - Prettier Turtle

<!--
SPDX-FileCopyrightText: 2022 Helsing GmbH
SPDX-FileCopyrightText: 2025 Robin Vobruba <hoijui.quaero@gmail.com>

SPDX-License-Identifier: Apache-2.0
-->

[![License: Apache-2.0](
    https://img.shields.io/badge/License-Apache--2.0-blue.svg)](
    LICENSE)
[![REUSE status](
    https://api.reuse.software/badge/codeberg.org/elevont/prttl)](
    https://api.reuse.software/info/codeberg.org/elevont/prttl)
[![Repo](
    https://img.shields.io/badge/Repo-CodeBerg-555555&logo=github.svg)](
    https://codeberg.org/elevont/prttl)
[![Package Releases](
    https://img.shields.io/crates/v/prttl.svg)](
    https://crates.io/crates/prttl)
[![Documentation Releases](
    https://docs.rs/prttl/badge.svg)](
    https://docs.rs/prttl)
[![Dependency Status](
    https://deps.rs/repo/codeberg.org/elevont/prttl/status.svg)](
    https://deps.rs/repo/codeberg.org/elevont/prttl)
[![Build Status](
    https://codeberg.org/elevont/prttl/workflows/build/badge.svg)](
    https://codeberg.org/elevont/prttl/actions)

`prttl` is an auto formatter (aka pretty printer)
for [RDF Turtle](https://www.w3.org/TR/turtle/).
It is optimized for diff minimization,
which is of value when developing Turtle (e.g. an Ontology) in a git repo.

## Installation

It is distributed on [Crates.io](https://crates.io/crates/prttl),
and can be installed with `cargo install prttl`.
Make sure you have [cargo installed](
https://doc.rust-lang.org/cargo/getting-started/installation.html)
before doing that.

## Usage

To use it:

```sh
prttl MY_TURTLE_FILE.ttl
```

It is also possible to check if formatting of a given file is valid
according to the formatter using:

```sh
prttl --check MY_TURTLE_FILE.ttl
```

If the formatting is not valid,
a patch to properly format the file is written to the standard output.

It is also possible to check a complete directory (and its subdirectories):

```sh
prttl MY_DIR
```

---

All the options:

```bash
$ prttl
Takes RDF data as input (commonly a Turtle file), and generates diff optimized RDF/Turtle, using a lot of new-lines. One peculiarity of this tool is, that it removes (Turtle-syntax) comments. We do this, because we believe that all comments should rather be encoded into triples, and we celebrate this in our own data, specifically our ontologies. More about this: <https://codeberg.org/elevont/cmt-ont>

Usage: prttl [OPTIONS] <FILE_OR_DIR>...

Arguments:
  <FILE_OR_DIR>...
          Source RDF file(s) or director(y|ies) containing Turtle files to format

Options:
      --canonicalize
          Whether to canonicalize the input before formatting. This refers to <https://www.w3.org/TR/rdf-canon/>, and effectively just label the blank nodes in a uniform way.

  -c, --check
          Do not edit the file but only check if it already applies this tools format

  -f, --force
          Forces overwriting of the output file, if it already exists, which includes the case of the input and output file being equal

  -l, --label-all-blank-nodes
          Whether to use labels for all blank nodes, or rather maximize nesting of them. NOTE That blank nodes referenced in more then one place can never be nested.

  -i, --indentation <NUM>
          Number of spaces per level of indentation

          [default: 2]

      --no-prtr-sorting
          Whether to disable sorting of blank nodes using their `prtr:sortingId` value, if any. [`prtr`](https://codeberg.org/elevont/prtr) is an ontology concerned with [RDF Pretty Printing](https://www.w3.org/DesignIssues/Pretty.html).

      --no-sparql-syntax
          Whether to use SPARQL-ish syntax for base and prefix, or the traditional Turtle syntax. - SPARQL-ish: ```turtle BASE <http://example.com/> PREFIX foaf: <http://xmlns.com/foaf/0.1/> ``` - Traditional Turtle: ```turtle @base <http://example.com/> . @prefix foaf: <http://xmlns.com/foaf/0.1/> . ```

      --pred-order [<PREDICATE>...]
          Sets a custom order of predicates to be used for sorting.
          Predicates that match come first; in the provided order.
          Predicates that do not match come afterwards; in alphabetical order.

          You may specify predicate names as absolute IRIs or as prefixed names.
          Only direct matches are considered; meaning: No type inference is conducted.

      --pred-order-preset <PREDICATE_ORDER_PRESET>
          Sets a predefined order of predicates to be used for sorting.
          Predicates that match come first; in the provided order.
          Predicates that do not match come afterwards; in alphabetical order.

          You may specify predicate names as absolute IRIs or as prefixed names.
          Only direct matches are considered; meaning: No type inference is conducted.

          [possible values: owl, skos, shacl, shex, rdf]

  -n, --single-leafed-new-lines
          Whether to move a single/lone predicate-object pair or object alone onto a new line

      --subj-type-order [<SUBJECT_TYPE>...]
          Sets a custom order of subject types to be used for sorting.
          Subjects with a matching type come first; in the provided order.
          Subjects without any matching type come afterwards; in alphabetical order.

          You may specify subject type names as absolute IRIs or as prefixed names.
          Only direct matches are considered; meaning: No type inference is conducted.

      --subj-type-order-preset <SUBJECT_TYPE_ORDER_PRESET>
          Sets a predefined order of subject types to be used for sorting.
          Subjects with a matching type come first; in the provided order.
          Subjects without any matching type come afterwards; in alphabetical order.

          Only direct matches are considered; meaning: No type inference is conducted.

          [possible values: owl, skos, shacl, shex, rdf]

  -q, --quiet
          Minimize or suppress output to stdout, and only shows log output on stderr.

  -v, --verbose
          more verbose output (useful for debugging)

  -V, --version
          Print version information and exit. May be combined with -q,--quiet, to really only output the version string.

  -h, --help
          Print help (see a summary with '-h')
```

## Format

`prttl` is in development and its output format is not stable yet.

## Sample Output

```turtle
@prefix ex: <http://example.com/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

<s>
  a ex:Foo ;
  <p>
    "foo"@en ,
    (
      +01
      +1.0
      1.0e0
    ) ;
  .

[
  ex:p
    ex:o ,
    ex:o2 ;
  ex:p2 ex:o3 ;
  ex:p3 true ;
] .
```

## Features

- **Removes all Turtle syntax comments.**
- Checks that the file is valid.
- Maintains consistent indentation and new-lines.
- Normalizes string and IRI escapes as much as possible.
- Enforces the use of `"` instead of `'` in literals.
- Uses literals short notation for booleans, integers, decimals and doubles
  when it keeps the lexical representation unchanged.
- Uses `a` for `rdf:type` where possible.

A much more detailed account and reasoning behind what this tool does
can be found in [DesignDecisions.md](DesignDecisions.md).
