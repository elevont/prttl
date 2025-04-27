// SPDX-FileCopyrightText: 2022 Helsing GmbH
//
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, bail, Result};
use tree_sitter::Node;

/// The order of the variants in this enum
/// determines the sorting order.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum NodeKind {
    Base,
    Comment,
    A,
    PredicateObjects,
    Prefix,
    PrefixedName,
    PNPrefix,
    IriRef,
    Collection,
    BlankNodeAnonymous,
    BlankNodeLabel,
    BlankNodePropertyList,
    LangTag,
    Literal,
    BooleanLiteral,
    IntegerLiteral,
    DecimalLiteral,
    DoubleLiteral,
    StringLiteral,
    Triples,
    TurtleDoc,
    At,
    DataType,
    IriStart,
    IriEnd,
    None,
}

impl From<&str> for NodeKind {
    fn from(kind: &str) -> Self {
        match kind {
            "base" => Self::Base,
            "comment" => Self::Comment,
            "iriref" => Self::IriRef,
            "predicate_objects" => Self::PredicateObjects,
            "prefix" => Self::Prefix,
            "prefixed_name" => Self::PrefixedName,
            "pn_prefix" => Self::PNPrefix,
            "a" => Self::A,
            "@" => Self::At,
            "^^" => Self::DataType,
            "<" => Self::IriStart,
            ">" => Self::IriEnd,
            "anon" => Self::BlankNodeAnonymous,
            "blank_node_label" => Self::BlankNodeLabel,
            "blank_node_property_list" => Self::BlankNodePropertyList,
            "collection" => Self::Collection,
            "langtag" => Self::LangTag,
            "literal" => Self::Literal,
            "string" => Self::StringLiteral,
            "integer" => Self::IntegerLiteral,
            "boolean" => Self::BooleanLiteral,
            "decimal" => Self::DecimalLiteral,
            "double" => Self::DoubleLiteral,
            "triples" => Self::Triples,
            "turtle_doc" => Self::TurtleDoc,
            _ => Self::None,
        }
    }
}

impl<'tree> From<&Node<'tree>> for NodeKind {
    fn from(node: &Node<'tree>) -> Self {
        Self::from(node.kind())
    }
}

pub struct StringDecoder<'a> {
    input: &'a str,
    i: usize,
}

impl<'a> StringDecoder<'a> {
    pub const fn new(input: &'a str) -> Self {
        Self { input, i: 0 }
    }
}

impl Iterator for StringDecoder<'_> {
    type Item = Result<char>;

    fn next(&mut self) -> Option<Result<char>> {
        let c = self.input[self.i..].chars().next()?;
        Some(if c == '\\' {
            match self.input[self.i + 1..].chars().next().unwrap() {
                'u' => {
                    self.i += 6;
                    decode_uchar(&self.input[self.i - 6..self.i])
                }
                'U' => {
                    self.i += 10;
                    decode_uchar(&self.input[self.i - 10..self.i])
                }
                c => {
                    self.i += c.len_utf8() + 1;
                    decode_echar(c)
                }
            }
        } else {
            self.i += c.len_utf8();
            Ok(c)
        })
    }
}

pub fn decode_echar(c: char) -> Result<char> {
    match c {
        't' => Ok('\t'),
        'b' => Ok('\x08'),
        'n' => Ok('\n'),
        'r' => Ok('\r'),
        'f' => Ok('\x0C'),
        '"' => Ok('"'),
        '\'' => Ok('\''),
        '\\' => Ok('\\'),
        _ => bail!("The escaped character '\\{c}' is not valid"),
    }
}

pub fn decode_uchar(input: &str) -> Result<char> {
    char::from_u32(u32::from_str_radix(&input[2..], 16).unwrap()).ok_or_else(|| {
        anyhow!("The escaped unicode character '{input}' is not encoding a valid unicode character")
    })
}

pub fn is_turtle_integer(value: &str) -> bool {
    // [19] 	INTEGER 	::= 	[+-]? [0-9]+
    let mut value = value.as_bytes();
    if value.starts_with(b"+") || value.starts_with(b"-") {
        value = &value[1..];
    }
    !value.is_empty() && value.iter().all(u8::is_ascii_digit)
}

pub fn is_turtle_decimal(value: &str) -> bool {
    // [20] 	DECIMAL 	::= 	[+-]? [0-9]* '.' [0-9]+
    let mut value = value.as_bytes();
    if value.starts_with(b"+") || value.starts_with(b"-") {
        value = &value[1..];
    }
    while value.first().is_some_and(u8::is_ascii_digit) {
        value = &value[1..];
    }
    if !value.starts_with(b".") {
        return false;
    }
    value = &value[1..];
    !value.is_empty() && value.iter().all(u8::is_ascii_digit)
}

pub fn is_turtle_double(value: &str) -> bool {
    // [21] 	DOUBLE 	::= 	[+-]? ([0-9]+ '.' [0-9]* EXPONENT | '.' [0-9]+ EXPONENT | [0-9]+ EXPONENT)
    // [154s] 	EXPONENT 	::= 	[eE] [+-]? [0-9]+
    let mut value = value.as_bytes();
    if value.starts_with(b"+") || value.starts_with(b"-") {
        value = &value[1..];
    }
    let mut with_before = false;
    while value.first().is_some_and(u8::is_ascii_digit) {
        value = &value[1..];
        with_before = true;
    }
    let mut with_after = false;
    if value.starts_with(b".") {
        value = &value[1..];
        while value.first().is_some_and(u8::is_ascii_digit) {
            value = &value[1..];
            with_after = true;
        }
    }
    if !(value.starts_with(b"e") || value.starts_with(b"E")) {
        return false;
    }
    value = &value[1..];
    if value.starts_with(b"+") || value.starts_with(b"-") {
        value = &value[1..];
    }
    (with_before || with_after) && !value.is_empty() && value.iter().all(u8::is_ascii_digit)
}

#[derive(Eq, PartialEq)]
pub enum RootContext {
    Start,
    Prefixes,
    Triples,
    Comment,
}
