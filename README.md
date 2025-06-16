# `prttl` - Prettier Turtle

<!--
SPDX-FileCopyrightText: 2022 Helsing GmbH

SPDX-License-Identifier: Apache-2.0
-->

[![License: Apache-2.0](
    https://img.shields.io/badge/License-Apache--2.0-blue.svg)](
    LICENSE.txt)
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
for [RDF Turtle](https://www.w3.org/TR/turtle/),
Optimized for diff minimization,
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

## Format

`prttl` is in development and its output format is not stable yet.

## Sample Output

```turtle
@prefix ex: <http://example.com/> . # Prefix
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

# Some facts

<s> a ex:Foo ;
    <p> "foo"@en , ( +01 +1.0 1.0e0 ) . # Foo

# An anonymous blank node
[ ex:p ex:o , ex:o2 ; ex:p2 ex:o3 ] ex:p3 true . # Bar
```

## Features

- Validates that the file is valid.
- Maintains consistent indentation and line jumps.
- Normalizes string and IRI escapes
  to reduce their number as much as possible.
- Enforces the use of `"` instead of `'` in literals.
- Uses literals short notation for booleans, integers, decimals and doubles
  when it keeps the lexical representation unchanged.
- Uses `a` for `rdf:type` where possible.
