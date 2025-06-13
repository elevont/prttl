// SPDX-FileCopyrightText: 2025 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

//! Provides ready to use [`NamedNodeRef`]s
//! for the [prtyr](http://w3id.org/oseg/ont/prtyr) OWL/RDF ontology/vocabulary.

use const_format::formatcp;
use oxrdf::NamedNodeRef;
use std::sync::LazyLock;

pub const NS: &str = "http://w3id.org/oseg/ont/prtyr#";
pub const PREFIX: &str = "prtyr";

/// The datatype property to assign an integer to each blank node,
/// to be used for sorting them when pretty-printing.
pub static SORTING_ID: LazyLock<NamedNodeRef> =
    LazyLock::new(|| NamedNodeRef::new_unchecked(formatcp!("{NS}sortingId")));
