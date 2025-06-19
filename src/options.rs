// SPDX-FileCopyrightText: 2022 Helsing GmbH
//
// SPDX-License-Identifier: Apache-2.0

use std::sync::LazyLock;

use clap::ValueEnum;

static CLS_ORDER_OWL: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    vec![
        "owl:Ontology",
        "owl:Class",
        "owl:ObjectProperties",
        "owl:DatatypeProperties",
        "owl:NamedIndividual",
        // "owl:Thing",
        // "owl:AnnotationProperty",
        // "owl:AsymmetricProperty",
        // "owl:FunctionalProperty",
        // "owl:InverseFunctionalProperty",
        // "owl:IrreflexiveProperty",
        // "owl:OntologyProperty",
        // "owl:ReflexiveProperty",
        // "owl:SymmetricProperty",
        // "owl:TransitiveProperty",
    ]
});
static CLS_ORDER_SKOS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    vec![
        "skos:ConceptScheme",
        "skos:Concept",
        "skos:OrderedCollection",
        "skos:Collection",
    ]
});
static CLS_ORDER_SHACL: LazyLock<Vec<&'static str>> =
    LazyLock::new(|| vec!["sh:NodeShape", "sh:PropertyShape", "sh:Shape"]);
static CLS_ORDER_SHEX: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    vec![
        "shex:Schema",
        "shex:ShapeExternal",
        "shex:ShapeAnd",
        "shex:ShapeOr",
        "shex:ShapeNot",
        "shex:Shape",
        "shex:TripleConstraint",
        "shex:Wildcard",
    ]
});
static CLS_ORDER_RDF: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    vec![
        "rdfs:Datatype",
        "rdfs:Class",
        "rdfs:Resource",
        "rdf:Property",
        // "rdfs:Container",
        // "rdfs:Literal",
        // "rdf:Bag",
        // "rdf:Seq",
        // "rdf:Alt",
        // "rdf:List",
        // "rdf:Statement",
    ]
});

static PRED_ORDER_OWL: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    vec![
        "owl:imports",
        "owl:deprecated",
        "owl:versionInfo",
        "owl:versionIRI",
        "owl:sameAs",
        "owl:cardinality",
        "owl:maxCardinality",
        "owl:minCardinality",
        // "owl:allValuesFrom",
        // "owl:annotatedProperty",
        // "owl:annotatedSource",
        // "owl:annotatedTarget",
        // "owl:assertionProperty",
        // "owl:backwardCompatibleWith",
        // "owl:bottomDataProperty",
        // "owl:bottomObjectProperty",
        // "owl:complementOf",
        // "owl:datatypeComplementOf",
        // "owl:differentFrom",
        // "owl:disjointUnionOf",
        // "owl:disjointWith",
        // "owl:distinctMembers",
        // "owl:equivalentClass",
        // "owl:equivalentProperty",
        // "owl:hasKey",
        // "owl:hasSelf",
        // "owl:hasValue",
        // "owl:incompatibleWith",
        // "owl:intersectionOf",
        // "owl:inverseOf",
        // "owl:maxQualifiedCardinality",
        // "owl:members",
        // "owl:minQualifiedCardinality",
        // "owl:onClass",
        // "owl:onDataRange",
        // "owl:onDatatype",
        // "owl:oneOf",
        // "owl:onProperties",
        // "owl:onProperty",
        // "owl:priorVersion",
        // "owl:propertyChainAxiom",
        // "owl:propertyDisjointWith",
        // "owl:qualifiedCardinality",
        // "owl:someValuesFrom",
        // "owl:sourceIndividual",
        // "owl:targetIndividual",
        // "owl:targetValue",
        // "owl:topDataProperty",
        // "owl:topObjectProperty",
        // "owl:unionOf",
        // "owl:withRestrictions",
    ]
});
static PRED_ORDER_SKOS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    vec![
        "skos:prefLabel",
        "skos:altLabel",
        "skos:hiddenLabel",
        "skos:changeNote",
        "skos:editorialNote",
        "skos:historyNote",
        "skos:scopeNote",
        "skos:note",
        "skos:example",
        "skos:broader",
        "skos:narrower",
        "skos:related",
        "skos:member",
        "skos:memberList",
        "skos:broadMatch",
        "skos:narrowMatch",
        "skos:relatedMatch",
        "skos:exactMatch",
        "skos:closeMatch",
        // "skos:mappingRelation",
        // "skos:inScheme",
        // "skos:hasTopConcept",
        // "skos:topConceptOf",
        // "skos:notation",
        // "skos:definition",
        // "skos:semanticRelation",
        // "skos:broaderTransitive",
        // "skos:narrowerTransitive",
    ]
});
static PRED_ORDER_SHACL: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    vec![
        "sh:deactivated",
        "sh:name",
        "sh:description",
        "sh:namespace",
        "sh:targetClass",
        "sh:targetNode",
        "sh:targetObjectsOf",
        "sh:targetSubjectsOf",
        "sh:target",
        "sh:node",
        "sh:nodeKind",
        "sh:property",
        "sh:value",
        "sh:shapesGraph",
        "sh:suggestedShapesGraph",
        "sh:optional",
        "sh:class",
        "sh:datatype",
        "sh:equals",
        "sh:disjoint",
        "sh:lessThan",
        "sh:lessThanOrEquals",
        "sh:maxCount",
        "sh:maxExclusive",
        "sh:maxInclusive",
        "sh:maxLength",
        "sh:minCount",
        "sh:minExclusive",
        "sh:minInclusive",
        "sh:minLength",
        "sh:pattern",
        // "sh:message",
        // "sh:severity",
        // "sh:conforms",
        // "sh:result",
        // "sh:detail",
        // "sh:focusNode",
        // "sh:sourceConstraint",
        // "sh:sourceShape",
        // "sh:sourceConstraintComponent",
        // "sh:entailment",
        // "sh:path",
        // "sh:inversePath",
        // "sh:alternativePath",
        // "sh:zeroOrMorePath",
        // "sh:oneOrMorePath",
        // "sh:zeroOrOnePath",
        // "sh:parameter",
        // "sh:labelTemplate",
        // "sh:validator",
        // "sh:nodeValidator",
        // "sh:propertyValidator",
        // "sh:and",
        // "sh:closed",
        // "sh:ignoredProperties",
        // "sh:hasValue",
        // "sh:in",
        // "sh:languageIn",
        // "sh:not",
        // "sh:or",
        // "sh:flags",
        // "sh:qualifiedMaxCount",
        // "sh:qualifiedMinCount",
        // "sh:qualifiedValueShape",
        // "sh:qualifiedValueShapesDisjoint",
        // "sh:uniqueLang",
        // "sh:xone",
        // "sh:ask",
        // "sh:construct",
        // "sh:select",
        // "sh:update",
        // "sh:prefixes",
        // "sh:declare",
        // "sh:prefix",
        // "sh:sparql",
        // "sh:defaultValue",
        // "sh:group",
        // "sh:order",
        // "sh:returnType",
        // "sh:resultAnnotation",
        // "sh:annotationProperty",
        // "sh:annotationValue",
        // "sh:annotationVarName",
        // "sh:this",
        // "sh:filterShape",
        // "sh:nodes",
        // "sh:intersection",
        // "sh:union",
        // "sh:expression",
        // "sh:rule",
        // "sh:condition",
        // "sh:subject",
        // "sh:predicate",
        // "sh:object",
    ]
});
static PRED_ORDER_SHEX: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    // TODO Select and order some of the below
    vec![
        // "shex:annotation",
        // "shex:closed",
        // "shex:code",
        // "shex:datatype",
        // "shex:exclusion",
        // "shex:expression",
        // "shex:expressions",
        // "shex:extra",
        // "shex:extends",
        // "shex:flags",
        // "shex:fractiondigits",
        // "shex:inverse",
        // "shex:languageTag",
        // "shex:length",
        // "shex:max",
        // "shex:maxexclusive",
        // "shex:maxinclusive",
        // "shex:maxlength",
        // "shex:min",
        // "shex:minexclusive",
        // "shex:mininclusive",
        // "shex:minlength",
        // "shex:name",
        // "shex:nodeKind",
        // "shex:numericFacet",
        // "shex:object",
        // "shex:pattern",
        // "shex:predicate",
        // "shex:semActs",
        // "shex:shapeExpr",
        // "shex:shapeExprs",
        // "shex:shapes",
        // "shex:start",
        // "shex:startActs",
        // "shex:stem",
        // "shex:stringFacet",
        // "shex:totaldigits",
        // "shex:valueExpr",
        // "shex:values",
        // "shex:xsFacet",
        // "shex:bnode",
        // "shex:iri",
        // "shex:literal",
        // "shex:nonliteral",
    ]
});
static PRED_ORDER_RDF: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    vec![
        "rdf:type",
        "rdf:first",
        "rdf:rest",
        "rdfs:label",
        "rdfs:comment",
        "rdf:language",
        "rdfs:subClassOf",
        "rdfs:subPropertyOf",
        "rdfs:domain",
        "rdfs:range",
        "rdfs:seeAlso",
        // "rdf:value",
    ]
});

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum SpecialSubjectTypeOrder {
    Owl,
    Skos,
    Shacl,
    Shex,
    Rdf,
}

impl SpecialSubjectTypeOrder {
    #[must_use]
    pub fn as_list(&self) -> &'static Vec<&'static str> {
        match self {
            Self::Owl => &CLS_ORDER_OWL,
            Self::Skos => &CLS_ORDER_SKOS,
            Self::Shacl => &CLS_ORDER_SHACL,
            Self::Shex => &CLS_ORDER_SHEX,
            Self::Rdf => &CLS_ORDER_RDF,
        }
    }
}

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum SpecialPredicateOrder {
    Owl,
    Skos,
    Shacl,
    Shex,
    Rdf,
}

impl SpecialPredicateOrder {
    #[must_use]
    pub fn as_list(&self) -> &'static Vec<&'static str> {
        match self {
            Self::Owl => &PRED_ORDER_OWL,
            Self::Skos => &PRED_ORDER_SKOS,
            Self::Shacl => &PRED_ORDER_SHACL,
            Self::Shex => &PRED_ORDER_SHEX,
            Self::Rdf => &PRED_ORDER_RDF,
        }
    }
}

pub struct FormatOptions {
    /// Do not edit the file but only check if it already applies this tools format.
    pub check: bool,
    /// Space(s) or tab(s) representing one level of indentation.
    pub indentation: String,
    /// Whether to move a single/lone object
    /// (within one subject-predicate pair) onto a new line,
    /// or to keep it on the same line as the predicate.
    pub single_leafed_new_lines: bool,
    /// Whether to force-write the output,
    /// even if potential issues with the formatting have been detected.
    ///
    /// One such issue would be,
    /// if comments have been found in the input.
    /// Because they will be completely removed in the output,
    /// we require `force = true` to try to avoid unintentional loss of information.
    pub force: bool,
    /// Sort blank nodes according to their `prtr:sortingId` value.
    ///
    /// [`prtr`](https://codeberg.org/elevont/prtr)
    /// is an ontology concerned with
    /// [RDF Pretty Printing](https://www.w3.org/DesignIssues/Pretty.html).
    pub prtr_sorting: bool,
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
    pub sparql_syntax: bool,
    /// Whether maximize nesting of blank nodes,
    /// or rather use labels for all of them.
    ///
    /// NOTE That blank nodes referenced in more then one place can never be nested.
    pub max_nesting: bool,
    /// Whether to canonicalize the input before formatting.
    /// This refers to <https://www.w3.org/TR/rdf-canon/>,
    /// and effectively just label the blank nodes in a uniform way.
    pub canonicalize: bool,
    /// Warn if a double or decimal literal can not be formatted as native Turtle literal.
    ///
    /// Turtles DOUBLE supports less formats then `xsd:double`,
    /// and DECIMAL supports less formats then `xsd:decimal`.
    /// See <https://github.com/w3c/rdf-turtle/issues/98> for more details.
    pub warn_unsupported_numbers: bool,
    /// A special subject type sorting order.
    ///
    /// This allows to choose _one_ predefined order of subject types,
    /// which prescribes the way subjects will be sorted
    /// according to their `a` (aka `rdf:type`) predicate value.
    /// The available options are somewhat common ways
    /// for sorting subjects according to their type,
    /// used in a certain context,
    /// e.g. OWL Ontologies, SKOS Vocabularies, SHACL Rules, etc.
    ///
    /// It uses the top most ordered type found,
    /// so tries to place a subject with types `list_idx_1` and `list_idx_3`
    /// before an other subject with type `list_idx_2`.
    ///
    /// NOTE: This does not use RDF inference, only 1-to-1 type matching!
    pub subject_type_order_preset: Option<SpecialSubjectTypeOrder>,
    /// A custom subject type sorting order.
    ///
    /// This allows you to define your own set of subject types to be sorted on top,
    /// which prescribes the way subjects will be sorted
    /// according to their `a` (aka `rdf:type`) predicate value.
    /// Each type can be defined as wither an absolute IRI,
    /// or as a prefixed name.
    ///
    /// It uses the top most ordered type found,
    /// so tries to place a subject with types `list_idx_1` and `list_idx_3`
    /// before an other subject with type `list_idx_2`.
    ///
    /// NOTE: This does not use RDF inference, only 1-to-1 type matching!
    pub subject_type_order: Option<Vec<String>>,
    /// A special predicate sorting order.
    ///
    /// This allows to choose _one_ predefined order of predicates.
    /// The available options are somewhat common ways
    /// for sorting the primary few predicates used in a certain context,
    /// e.g. OWL Ontologies, SKOS Vocabularies, SHACL Rules, etc.
    ///
    /// If this is used, it overrides the special case
    /// of sorting `a` (aka `rdf:type`) at the top.
    pub predicate_order_preset: Option<SpecialPredicateOrder>,
    /// A custom predicates sorting order.
    ///
    /// This allows you to define your own set of predicates to be sorted on top,
    /// and the order they should be sorted in.
    /// Each predicate can be defined as wither an absolute IRI,
    /// or as a prefixed name.
    ///
    /// If this is used, it overrides the special case
    /// of sorting `a` (aka `rdf:type`) at the top.
    /// If you still want that,
    /// you have to manually add include it in this list.
    pub predicate_order: Option<Vec<String>>,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            check: true,
            indentation: "  ".to_string(),
            single_leafed_new_lines: false,
            force: false,
            prtr_sorting: true,
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
}

impl FormatOptions {
    #[must_use]
    pub fn subject_type_order(&self) -> Option<Vec<String>> {
        self.subject_type_order.clone().or_else(|| {
            self.subject_type_order_preset
                .as_ref()
                .map(|variant| variant.as_list().iter().map(ToString::to_string).collect())
        })
    }

    #[must_use]
    pub fn predicate_order(&self) -> Option<Vec<String>> {
        self.predicate_order.clone().or_else(|| {
            self.predicate_order_preset
                .as_ref()
                .map(|variant| variant.as_list().iter().map(ToString::to_string).collect())
        })
    }
}
