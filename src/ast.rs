// SPDX-FileCopyrightText: 2025 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::HashSet;
use std::convert::Infallible;
use std::hash::Hash;
use std::rc::Rc;
use std::sync::LazyLock;

use oxrdf::vocab::rdf;
use oxrdf::BlankNodeRef;
use oxrdf::Graph;
use oxrdf::LiteralRef;
use oxrdf::NamedNodeRef;
use oxrdf::SubjectRef;
use oxrdf::TermRef;
use oxrdf::TripleRef;

use crate::compare;
use crate::input::Input;
use crate::options::FormatOptions;

static T_RDF_LIST: LazyLock<TermRef> = LazyLock::new(|| TermRef::NamedNode(rdf::LIST));

/// An AST node.
pub trait Part {
    /// Whether this part may have sub-parts.
    fn is_container(&self) -> bool;

    /// Whether this part has no sub-parts.
    fn is_empty(&self) -> bool;

    /// Whether this part "is" a single item.
    ///
    /// With "single leafed", we mean e.g. a subject with a single predicate with a single object,
    /// which is its self either a literal, a named node, an empty collection
    /// or a collection with a single entry,
    /// which is single leafed its self.
    /// In short, something that - depending on the formatting options -
    /// might be printed on a single line.
    fn is_single_leafed(&self) -> bool;
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum TSubject<'graph> {
    NamedNode(TNamedNode<'graph>),
    // PrefixedNamedNode(NamedNodeRef<'graph>, &'graph str, &'graph str),
    BlankNodeLabel(TBlankNodeRef<'graph>),
    // BlankNodeAnonymous(TBlankNode<'graph>),
    BlankNodeAnonymous(TBlankNode<'graph>),
    Collection(TCollection<'graph>),
    Triple(Box<TTriple<'graph>>),
}

impl<'us, 'graph> TSubject<'graph> {
    fn from(
        input: &'graph Input,
        g_main: &'graph Graph,
        non_empty_valid_cols: &'us HashMap<BlankNodeRef<'graph>, Vec<TermRef<'graph>>>,
        nestable_blank_nodes: &HashSet<BlankNodeRef<'graph>>,
        col_involved_triples: &Vec<TripleRef<'graph>>,
        other: SubjectRef<'graph>,
    ) -> Self {
        match other {
            SubjectRef::NamedNode(named_node_ref) => {
                Self::NamedNode(TNamedNode::from(input, named_node_ref))
            }
            SubjectRef::BlankNode(blank_node_ref) => {
                // Self::BlankNodeLabel(TBlankNodeRef(blank_node_ref))
                match blank_node_label_or_collection(
                    input,
                    g_main,
                    non_empty_valid_cols,
                    nestable_blank_nodes,
                    col_involved_triples,
                    blank_node_ref,
                )
                .expect("Infallible")
                {
                    Some(TBlankNodeOrCollection::BlankNode(_bn)) => {
                        // TObject::BlankNodeAnonymous(bn);
                        panic!("We should never be able to have an anonymous blank node as triple object");
                    }
                    Some(TBlankNodeOrCollection::Collection(col)) => Self::Collection(col),
                    None => Self::BlankNodeLabel(TBlankNodeRef(blank_node_ref)),
                }
            }
            SubjectRef::Triple(triple) => Self::Triple(Box::new(TTriple::from(
                input,
                g_main,
                non_empty_valid_cols,
                nestable_blank_nodes,
                col_involved_triples,
                &triple.as_ref(),
            ))),
        }
    }
}

impl Part for TSubject<'_> {
    fn is_container(&self) -> bool {
        match self {
            Self::BlankNodeAnonymous(_) | Self::Collection(_) | Self::Triple(_) => true,
            Self::BlankNodeLabel(_) | Self::NamedNode(_) => false,
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            Self::BlankNodeAnonymous(bn) => bn.is_empty(),
            Self::Collection(col) => col.is_empty(),
            Self::Triple(_triple) => false,
            Self::BlankNodeLabel(_) | Self::NamedNode(_) => true,
        }
    }

    fn is_single_leafed(&self) -> bool {
        match self {
            Self::BlankNodeAnonymous(bn) => bn.is_single_leafed(),
            Self::Collection(col) => col.is_single_leafed(),
            Self::Triple(triple) => triple.is_single_leafed(),
            Self::BlankNodeLabel(_) | Self::NamedNode(_) => true,
        }
    }
}

impl From<&TSubject<'_>> for u8 {
    fn from(value: &TSubject<'_>) -> Self {
        match value {
            TSubject::NamedNode(_) => 0,
            TSubject::BlankNodeLabel(_) => 4,
            TSubject::BlankNodeAnonymous(_) => 3,
            TSubject::Collection(_) => 2,
            TSubject::Triple(_) => 1,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TSubjectCont<'graph> {
    pub subject: TSubject<'graph>,
    pub predicates: Vec<TPredicateCont<'graph>>,
}

impl<'us, 'graph> TSubjectCont<'graph> {
    fn from(
        input: &'graph Input,
        g_main: &'graph Graph,
        non_empty_valid_cols: &'us HashMap<BlankNodeRef<'graph>, Vec<TermRef<'graph>>>,
        nestable_blank_nodes: &HashSet<BlankNodeRef<'graph>>,
        col_involved_triples: &Vec<TripleRef<'graph>>,
        other: SubjectRef<'graph>,
    ) -> Self {
        Self {
            subject: TSubject::from(
                input,
                g_main,
                non_empty_valid_cols,
                nestable_blank_nodes,
                col_involved_triples,
                other,
            ),
            predicates: Vec::new(),
        }
    }
}

impl Part for TSubjectCont<'_> {
    fn is_container(&self) -> bool {
        true
    }

    fn is_empty(&self) -> bool {
        self.predicates.is_empty()
    }

    fn is_single_leafed(&self) -> bool {
        self.predicates.len() == 1 && self.predicates.first().unwrap().is_single_leafed()
    }
}

impl<'graph> PredicatesStore<'graph> for TSubjectCont<'graph> {
    fn get_predicates_mut<'us>(&'us mut self) -> &'us mut Vec<TPredicateCont<'graph>>
    where
        'graph: 'us,
    {
        &mut self.predicates
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum TNamedNode<'graph> {
    Plain(NamedNodeRef<'graph>),
    Prefixed(NamedNodeRef<'graph>, &'graph str, &'graph str),
    Based(NamedNodeRef<'graph>, &'graph str),
}

impl<'graph> TNamedNode<'graph> {
    fn from(input: &'graph Input, named_node: NamedNodeRef<'graph>) -> Self {
        if let Some((namespace, local_name)) = named_node
            .as_str()
            .rsplit_once('#')
            .or_else(|| named_node.as_str().rsplit_once('/'))
        {
            let namespace = &named_node.as_str()[0..=namespace.len()];
            if let Some(prefix) = input.prefixes_inverted.get(namespace) {
                return Self::Prefixed(named_node, prefix, local_name);
            }
        }
        if let Some(base) = input.base.as_deref() {
            if named_node.as_str().starts_with(base) {
                return Self::Based(named_node, &named_node.as_str()[base.len()..]);
            }
        }
        Self::Plain(named_node)
    }

    #[must_use]
    pub const fn as_named_node_ref(&'graph self) -> &'graph NamedNodeRef<'graph> {
        match self {
            Self::Plain(nn) | Self::Prefixed(nn, _, _) | Self::Based(nn, _) => nn,
        }
    }
}

impl Part for TNamedNode<'_> {
    fn is_container(&self) -> bool {
        false
    }

    fn is_empty(&self) -> bool {
        true
    }

    fn is_single_leafed(&self) -> bool {
        true
    }
}

impl Ord for TNamedNode<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (TNamedNode::Plain(_), TNamedNode::Prefixed(_, _, _) | TNamedNode::Based(_, _))
            | (TNamedNode::Based(_, _), TNamedNode::Prefixed(_, _, _)) => Ordering::Less,
            (TNamedNode::Prefixed(_, _, _), TNamedNode::Plain(_) | TNamedNode::Based(_, _))
            | (TNamedNode::Based(_, _), TNamedNode::Plain(_)) => Ordering::Greater,
            (TNamedNode::Plain(a_nn), TNamedNode::Plain(b_nn)) => a_nn.as_str().cmp(b_nn.as_str()),
            (
                TNamedNode::Prefixed(_a_nn, a_prefix, a_local_name),
                TNamedNode::Prefixed(_b_nn, b_prefix, b_local_name),
            ) => {
                let prefix_cmp = a_prefix.cmp(b_prefix);
                if prefix_cmp != Ordering::Equal {
                    return prefix_cmp;
                }
                a_local_name.cmp(b_local_name)
            }
            (
                TNamedNode::Based(_a_nn, a_additional_name),
                TNamedNode::Based(_b_nn, b_additional_name),
            ) => a_additional_name.cmp(b_additional_name),
        }
    }
}

impl PartialOrd for TNamedNode<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub type TPredicate<'graph> = TNamedNode<'graph>;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct TPredicateCont<'graph> {
    pub predicate: TPredicate<'graph>,
    pub objects: Vec<TObject<'graph>>,
}

impl<'graph> TPredicateCont<'graph> {
    fn from(input: &'graph Input, other: NamedNodeRef<'graph>) -> Self {
        Self {
            predicate: TPredicate::from(input, other),
            objects: Vec::new(),
        }
    }
}

impl Part for TPredicateCont<'_> {
    fn is_container(&self) -> bool {
        true
    }

    fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    fn is_single_leafed(&self) -> bool {
        self.objects.len() == 1 && self.objects.first().unwrap().is_single_leafed()
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct TBlankNodeRef<'graph>(pub BlankNodeRef<'graph>);

impl<'graph> From<BlankNodeRef<'graph>> for TBlankNodeRef<'graph> {
    fn from(other: BlankNodeRef<'graph>) -> Self {
        Self(other)
    }
}

impl Ord for TBlankNodeRef<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.as_str().cmp(other.0.as_str())
    }
}

impl PartialOrd for TBlankNodeRef<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Part for TBlankNodeRef<'_> {
    fn is_container(&self) -> bool {
        false
    }

    fn is_empty(&self) -> bool {
        true
    }

    fn is_single_leafed(&self) -> bool {
        true
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct TLiteralRef<'graph>(pub LiteralRef<'graph>, pub Option<TNamedNode<'graph>>);

impl Ord for TLiteralRef<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        let a = self.0;
        let b = other.0;
        let cmp_value = a.value().cmp(b.value());
        if cmp_value != Ordering::Equal {
            return cmp_value;
        }
        let cmp_datatype = a.datatype().cmp(&b.datatype());
        if cmp_datatype != Ordering::Equal {
            return cmp_datatype;
        }
        match (a.language(), b.language()) {
            (Some(a), Some(b)) => a.cmp(b),
            (Some(_a), None) => Ordering::Less,
            (None, Some(_b)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        }
    }
}

impl PartialOrd for TLiteralRef<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Part for TLiteralRef<'_> {
    fn is_container(&self) -> bool {
        false
    }

    fn is_empty(&self) -> bool {
        true
    }

    fn is_single_leafed(&self) -> bool {
        true
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum TObject<'graph> {
    NamedNode(TNamedNode<'graph>),
    BlankNodeLabel(TBlankNodeRef<'graph>),
    BlankNodeAnonymous(TBlankNode<'graph>),
    Collection(TCollection<'graph>),
    Literal(TLiteralRef<'graph>),
    Triple(Box<TTriple<'graph>>),
}

impl<'us, 'graph> TObject<'graph> {
    fn from(
        input: &'graph Input,
        g_main: &'graph Graph,
        non_empty_valid_cols: &'us HashMap<BlankNodeRef<'graph>, Vec<TermRef<'graph>>>,
        nestable_blank_nodes: &HashSet<BlankNodeRef<'graph>>,
        col_involved_triples: &Vec<TripleRef<'graph>>,
        other: TermRef<'graph>,
    ) -> Self {
        match other {
            TermRef::NamedNode(named_node_ref) => {
                if named_node_ref == rdf::NIL {
                    Self::Collection(TCollection::Empty)
                } else {
                    Self::NamedNode(TNamedNode::from(input, named_node_ref))
                }
            }
            TermRef::BlankNode(blank_node_ref) => {
                match blank_node_label_or_collection(
                    input,
                    g_main,
                    non_empty_valid_cols,
                    nestable_blank_nodes,
                    col_involved_triples,
                    blank_node_ref,
                )
                .expect("Infallible")
                {
                    Some(TBlankNodeOrCollection::BlankNode(bn)) => TObject::BlankNodeAnonymous(bn),
                    Some(TBlankNodeOrCollection::Collection(col)) => Self::Collection(col.clone()),
                    None => Self::BlankNodeLabel(TBlankNodeRef(blank_node_ref)),
                }
            }
            TermRef::Literal(literal_ref) => {
                let data_type_nn = if literal_ref.is_plain() {
                    None
                } else {
                    Some(TNamedNode::from(input, literal_ref.datatype()))
                };
                Self::Literal(TLiteralRef(literal_ref, data_type_nn))
            }
            TermRef::Triple(triple) => Self::Triple(Box::new(TTriple::from(
                input,
                g_main,
                non_empty_valid_cols,
                nestable_blank_nodes,
                col_involved_triples,
                &triple.as_ref(),
            ))),
        }
    }
}

impl Part for TObject<'_> {
    fn is_container(&self) -> bool {
        match self {
            Self::NamedNode(_nn) => false,
            Self::BlankNodeLabel(_bn) => false,
            Self::BlankNodeAnonymous(_bn) => true,
            Self::Collection(_col) => true,
            Self::Literal(_lit) => false,
            Self::Triple(_triple) => true,
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            Self::NamedNode(_nn) => true,
            Self::BlankNodeLabel(_bn) => true,
            Self::BlankNodeAnonymous(bn) => bn.is_empty(),
            Self::Collection(col) => col.is_empty(),
            Self::Literal(_lit) => true,
            Self::Triple(triple) => triple.is_empty(),
        }
    }

    fn is_single_leafed(&self) -> bool {
        match self {
            Self::NamedNode(_) | Self::BlankNodeLabel(_) | Self::Literal(_) => true,
            Self::BlankNodeAnonymous(bn) => bn.is_single_leafed(),
            Self::Collection(col) => col.is_single_leafed(),
            Self::Triple(_triple) => false, //triple.is_single_leafed(),
        }
    }
}

impl From<&TObject<'_>> for u8 {
    fn from(value: &TObject<'_>) -> Self {
        match value {
            TObject::NamedNode(_) => 0,
            TObject::BlankNodeLabel(_) => 5,
            TObject::BlankNodeAnonymous(_) => 4,
            TObject::Collection(_) => 3,
            TObject::Literal(_) => 2,
            TObject::Triple(_) => 1,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct TCollectionRef<'graph> {
    pub node: TBlankNode<'graph>,
    pub rest: Vec<TObject<'graph>>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum TCollection<'graph> {
    WithContent(TCollectionRef<'graph>),
    Empty,
}

impl Part for TCollection<'_> {
    fn is_container(&self) -> bool {
        true
    }

    fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    fn is_single_leafed(&self) -> bool {
        match self {
            Self::WithContent(col) => {
                col.rest.len() == 1 && col.rest.first().unwrap().is_single_leafed()
            }
            Self::Empty => true,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct TTriple<'graph>(
    pub TSubject<'graph>,
    pub TPredicate<'graph>,
    pub TObject<'graph>,
);

impl<'us, 'graph> TTriple<'graph> {
    fn from(
        input: &'graph Input,
        g_main: &'graph Graph,
        non_empty_valid_cols: &'us HashMap<BlankNodeRef<'graph>, Vec<TermRef<'graph>>>,
        nestable_blank_nodes: &HashSet<BlankNodeRef<'graph>>,
        col_involved_triples: &Vec<TripleRef<'graph>>,
        other: &TripleRef<'graph>,
    ) -> Self {
        Self(
            TSubject::from(
                input,
                g_main,
                non_empty_valid_cols,
                nestable_blank_nodes,
                col_involved_triples,
                other.subject,
            ),
            TPredicate::from(input, other.predicate),
            TObject::from(
                input,
                g_main,
                non_empty_valid_cols,
                nestable_blank_nodes,
                col_involved_triples,
                other.object,
            ),
        )
    }
}

impl Part for TTriple<'_> {
    fn is_container(&self) -> bool {
        true
    }

    fn is_empty(&self) -> bool {
        false
    }

    fn is_single_leafed(&self) -> bool {
        self.0.is_single_leafed() && self.1.is_single_leafed() && self.2.is_single_leafed()
    }
}

enum TBlankNodeOrCollection<'graph> {
    BlankNode(TBlankNode<'graph>),
    Collection(TCollection<'graph>),
}

fn blank_node_label_or_collection<'graph>(
    input: &'graph Input,
    g_main: &'graph Graph,
    non_empty_valid_cols: &HashMap<BlankNodeRef<'graph>, Vec<TermRef<'graph>>>,
    nestable_blank_nodes: &HashSet<BlankNodeRef<'graph>>,
    col_involved_triples: &Vec<TripleRef<'graph>>,
    bn: BlankNodeRef<'graph>,
) -> Result<Option<TBlankNodeOrCollection<'graph>>, Infallible> {
    Ok(if let Some(col) = non_empty_valid_cols.get(&bn) {
        let mut tbn = TBlankNode::from(bn);
        tbn.create_graph_entry(
            input,
            g_main,
            non_empty_valid_cols,
            nestable_blank_nodes,
            col_involved_triples,
            g_main.triples_for_subject(bn),
        )?;
        Some(TBlankNodeOrCollection::Collection(
            TCollection::WithContent(TCollectionRef {
                node: tbn,
                rest: col
                    .iter()
                    .map(|term| {
                        TObject::from(
                            input,
                            g_main,
                            non_empty_valid_cols,
                            nestable_blank_nodes,
                            col_involved_triples,
                            *term,
                        )
                    })
                    .collect(),
            }),
        ))
    } else if nestable_blank_nodes.contains(&bn) {
        let mut tbn = TBlankNode::from(bn);
        tbn.create_graph_entry(
            input,
            g_main,
            non_empty_valid_cols,
            nestable_blank_nodes,
            col_involved_triples,
            g_main.triples_for_subject(bn),
        )?;
        Some(TBlankNodeOrCollection::BlankNode(tbn))
    } else {
        None
    })
}

trait PredicatesStore<'graph> {
    fn get_predicates_mut<'us>(&'us mut self) -> &'us mut Vec<TPredicateCont<'graph>>
    where
        'graph: 'us;

    fn create_graph_entry<'us>(
        &'us mut self,
        input: &'graph Input,
        g_main: &'graph Graph,
        non_empty_valid_cols: &'us HashMap<BlankNodeRef<'graph>, Vec<TermRef<'graph>>>,
        nestable_blank_nodes: &HashSet<BlankNodeRef<'graph>>,
        col_involved_triples: &Vec<TripleRef<'graph>>,
        level_triples: impl Iterator<Item = TripleRef<'graph>>,
    ) -> Result<(), Infallible>
    where
        'graph: 'us,
    {
        let mut predicate_objects = HashMap::new();
        for triple in level_triples {
            if col_involved_triples.contains(&triple) {
                continue;
            }
            match triple.subject {
                SubjectRef::BlankNode(bn) => {
                    if non_empty_valid_cols.contains_key(&bn) || nestable_blank_nodes.contains(&bn)
                    {
                        // continue;
                    }
                }
                SubjectRef::NamedNode(_) | SubjectRef::Triple(_) => (),
            }
            predicate_objects
                .entry(triple.predicate)
                .or_insert_with(Vec::new)
                .push(triple.object);
        }
        for (predicate, objects) in predicate_objects {
            let mut predicate = TPredicateCont::from(input, predicate);
            for object in objects {
                let mut t_object = TObject::from(
                    input,
                    g_main,
                    non_empty_valid_cols,
                    nestable_blank_nodes,
                    col_involved_triples,
                    object,
                );
                if let TermRef::BlankNode(bn) = object {
                    match blank_node_label_or_collection(
                        input,
                        g_main,
                        non_empty_valid_cols,
                        nestable_blank_nodes,
                        col_involved_triples,
                        bn,
                    )? {
                        Some(TBlankNodeOrCollection::BlankNode(bn)) => {
                            t_object = TObject::BlankNodeAnonymous(bn);
                        }
                        Some(TBlankNodeOrCollection::Collection(col)) => {
                            t_object = TObject::Collection(col.clone());
                        }
                        None => (),
                    }
                }
                predicate.objects.push(t_object);
            }
            self.get_predicates_mut().push(predicate);
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TBlankNode<'graph> {
    pub node: TBlankNodeRef<'graph>,
    pub predicates: Vec<TPredicateCont<'graph>>,
}

impl<'graph> From<BlankNodeRef<'graph>> for TBlankNode<'graph> {
    fn from(other: BlankNodeRef<'graph>) -> Self {
        Self {
            node: other.into(),
            predicates: Vec::new(),
        }
    }
}

impl<'graph> PredicatesStore<'graph> for TBlankNode<'graph> {
    fn get_predicates_mut<'us>(&'us mut self) -> &'us mut Vec<TPredicateCont<'graph>>
    where
        'graph: 'us,
    {
        &mut self.predicates
    }
}

impl std::hash::Hash for TBlankNode<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.node.hash(state);
    }
}

impl Part for TBlankNode<'_> {
    fn is_container(&self) -> bool {
        true
    }

    fn is_empty(&self) -> bool {
        self.predicates.is_empty()
    }

    fn is_single_leafed(&self) -> bool {
        self.predicates.len() == 1 && self.predicates.first().unwrap().is_single_leafed()
    }
}

pub struct SortingContext<'sorting> {
    pub options: Rc<FormatOptions>,
    // pub prefixes: &'sorting Vec<(String, String)>,
    pub graph: &'sorting Graph,
    /// A cache for blank node sorting ids, for performance reasons.
    pub bn_sorting_ids: Rc<RefCell<HashMap<BlankNodeRef<'sorting>, Option<u32>>>>,
}

#[derive(Debug)]
pub struct TRoot<'graph> {
    pub subjects: Vec<TSubjectCont<'graph>>,
}

impl<'graph> TRoot<'graph> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            subjects: Vec::new(),
        }
    }

    fn sort_triple(triple: &mut TTriple<'graph>, context: &SortingContext<'graph>) {
        Self::sort_object(&mut triple.2, context);
    }

    fn sort_collection_ref(
        collection: &mut TCollectionRef<'graph>,
        context: &SortingContext<'graph>,
    ) {
        // NOTE NO! We can *not* sort collection items themselves, but we have to sort their terms internally (if multiple)!
        // Self::sort_objects(&mut collection.rest, context);
        for term in &mut collection.rest {
            Self::sort_object(term, context);
        }
    }

    fn sort_blank_node(blank_node: &mut TBlankNode<'graph>, context: &SortingContext<'graph>) {
        Self::sort_predicates(&mut blank_node.predicates, context);
    }

    fn sort_subject(subject: &mut TSubject<'graph>, context: &SortingContext<'graph>) {
        match subject {
            TSubject::Collection(TCollection::WithContent(ref mut collection)) => {
                Self::sort_collection_ref(collection, context);
            }
            TSubject::BlankNodeAnonymous(ref mut blank_node) => {
                Self::sort_blank_node(blank_node, context);
            }
            TSubject::Triple(ref mut triple_box) => {
                Self::sort_triple(triple_box, context);
            }
            // NOTE We need not sort BlankNodeLabel here,
            //      because it is already sorted by being a Subject within TRoot.
            TSubject::NamedNode(_)
            | TSubject::Collection(TCollection::Empty)
            | TSubject::BlankNodeLabel(_) => (),
        }
    }

    fn sort_subject_cont(
        subject_cont: &mut TSubjectCont<'graph>,
        context: &SortingContext<'graph>,
    ) {
        Self::sort_subject(&mut subject_cont.subject, context);
        Self::sort_predicates(&mut subject_cont.predicates, context);
    }

    fn sort_object(object: &mut TObject<'graph>, context: &SortingContext<'graph>) {
        match object {
            TObject::Collection(TCollection::WithContent(ref mut collection)) => {
                Self::sort_collection_ref(collection, context);
            }
            TObject::BlankNodeAnonymous(ref mut blank_node) => {
                Self::sort_blank_node(blank_node, context);
            }
            TObject::Triple(ref mut triple_box) => {
                Self::sort_triple(triple_box, context);
            }
            // NOTE We need not sort BlankNodeLabel here,
            //      because it is already sorted by being a Subject within TRoot.
            TObject::NamedNode(_)
            | TObject::Collection(TCollection::Empty)
            | TObject::BlankNodeLabel(_)
            | TObject::Literal(_) => (),
        }
    }

    fn sort_objects(objects: &mut Vec<TObject<'graph>>, context: &SortingContext<'graph>) {
        objects.sort_by(|a, b| compare::t_obj(context, a, b));
        for object in objects.iter_mut() {
            Self::sort_object(object, context);
        }
    }

    fn sort_predicates(
        predicates: &mut Vec<TPredicateCont<'graph>>,
        context: &SortingContext<'graph>,
    ) {
        predicates.sort_by(|a, b| compare::t_pred_cont(context, a, b));
        for predicate_cont in predicates {
            Self::sort_objects(&mut predicate_cont.objects, context);
        }
    }

    pub fn sort(&mut self, context: &SortingContext<'graph>) {
        self.subjects
            .sort_by(|a, b| compare::t_subj_cont(context, a, b));
        for subject_cont in &mut self.subjects {
            Self::sort_subject_cont(subject_cont, context);
        }
    }
}

impl Default for TRoot<'_> {
    fn default() -> Self {
        Self::new()
    }
}

fn extract_duplicates<'graph>(
    entries: &Vec<BlankNodeRef<'graph>>,
) -> HashSet<BlankNodeRef<'graph>> {
    let mut seen_at_least_once = HashSet::new();
    let mut seen_at_least_twice = HashSet::new();
    for entry in entries {
        if seen_at_least_once.contains(entry) {
            seen_at_least_twice.insert(*entry);
        } else {
            seen_at_least_once.insert(*entry);
        }
    }
    seen_at_least_twice.into_iter().collect()
}

fn extract_collection<'graph>(
    g_main: &'graph Graph,
    involved_triples: &Rc<RefCell<Vec<TripleRef<'graph>>>>,
    start: BlankNodeRef<'graph>,
) -> Option<Vec<TermRef<'graph>>> {
    let mut cur = start;
    let mut col = vec![];
    let mut involved_triples = involved_triples.borrow_mut();
    loop {
        let firsts = g_main
            .objects_for_subject_predicate(cur, rdf::FIRST)
            .collect::<Vec<_>>();
        if firsts.len() != 1 {
            return None;
        }
        let first = *firsts.first().unwrap();
        let cur_subj = SubjectRef::BlankNode(cur);
        involved_triples.push(TripleRef::new(cur_subj, rdf::FIRST, first));
        col.push(first);

        let rests = g_main
            .objects_for_subject_predicate(cur, rdf::REST)
            .collect::<Vec<_>>();
        if rests.len() != 1 {
            return None;
        }
        let rest = *rests.first().unwrap();
        involved_triples.push(TripleRef::new(cur_subj, rdf::REST, rest));

        let types = g_main
            .objects_for_subject_predicate(cur, rdf::TYPE)
            .collect::<Vec<_>>();
        if types.len() > 1 && cur != start {
            return None;
        }
        let mut list_native_triples = 2;
        if types.contains(&T_RDF_LIST) {
            involved_triples.push(TripleRef::new(cur_subj, rdf::TYPE, *T_RDF_LIST));
            list_native_triples += 1;
        }
        if cur != start {
            let subj_triples = g_main.triples_for_subject(cur).count();
            if subj_triples != list_native_triples {
                return None;
            }
        }

        match rest {
            TermRef::BlankNode(bn_rest) => {
                cur = bn_rest;
            }
            TermRef::NamedNode(nn_rest) => {
                if nn_rest == rdf::NIL {
                    break;
                }
                return None;
            }
            TermRef::Literal(lit_rest) => {
                tracing::error!("Literal as collection chain element is invalid: {lit_rest}");
                return None;
            }
            TermRef::Triple(triple) => {
                tracing::error!("Triple as collection chain element is invalid: {triple}");
                return None;
            }
        }
    }
    Some(col)
}

fn evaluate_nestable_and_unreferenced_blank_nodes<'graph, 'tree>(
    g_main: &'graph Graph,
    unreferenced_blank_nodes: &'tree mut HashSet<BlankNodeRef<'graph>>,
) -> HashSet<BlankNodeRef<'graph>>
where
    'graph: 'tree,
{
    let mut subject_bns = vec![];
    let mut object_bns = vec![];
    for triple in g_main {
        if let SubjectRef::BlankNode(bn_subj) = triple.subject {
            subject_bns.push(bn_subj);
        }
        if let TermRef::BlankNode(bn_obj) = triple.object {
            object_bns.push(bn_obj);
        }
    }
    for subj_bn in subject_bns.iter().filter(|bn| !object_bns.contains(bn)) {
        unreferenced_blank_nodes.insert(*subj_bn);
    }
    let duplicate_obj_bns = extract_duplicates(&object_bns);
    subject_bns.retain(|bn| object_bns.contains(bn) && !duplicate_obj_bns.contains(bn));
    subject_bns.into_iter().collect()
}

fn extract_non_empty_collections<'graph>(
    g_main: &'graph Graph,
    involved_triples: &Rc<RefCell<Vec<TripleRef<'graph>>>>,
) -> HashMap<BlankNodeRef<'graph>, Vec<TermRef<'graph>>> {
    let mut col_starts = vec![];
    {
        for triple in g_main {
            if let SubjectRef::BlankNode(bn_subj) = triple.subject {
                if triple.predicate == rdf::FIRST {
                    let rest_refs_to_subj = g_main
                        .subjects_for_predicate_object(rdf::REST, bn_subj)
                        .next()
                        .is_some();
                    if !rest_refs_to_subj {
                        // subj is a collection start
                        col_starts.push(bn_subj);
                    }
                }
            }
        }
    }

    let mut cols = HashMap::new();
    for col_start in col_starts {
        if let Some(col) = extract_collection(g_main, involved_triples, col_start) {
            cols.insert(col_start, col);
        }
    }

    cols
}

pub fn subjects(graph: &Graph) -> impl Iterator<Item = SubjectRef<'_>> + '_ {
    let mut seen = HashSet::new();
    graph.iter().filter_map(move |triple| {
        if seen.insert(triple.subject) {
            Some(triple.subject)
        } else {
            None
        }
    })
}

pub fn construct_tree<'tree, 'graph>(
    tree_root: &'tree mut TRoot<'graph>,
    unreferenced_blank_nodes: &'tree mut HashSet<BlankNodeRef<'graph>>,
    input: &'graph Input,
) -> Result<(), Infallible>
where
    'graph: 'tree,
{
    let col_involved_triples: Rc<RefCell<Vec<TripleRef<'_>>>> = Rc::new(RefCell::new(Vec::new()));
    let non_empty_valid_cols = extract_non_empty_collections(&input.graph, &col_involved_triples);
    tracing::debug!(
        "\ncol_involved_triples:\n{}",
        col_involved_triples
            .borrow()
            .iter()
            .map(|triple| format!(
                "{} {} {} .",
                triple.subject, triple.predicate, triple.object
            )
            .replace('\n', "\\n"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    tracing::debug!(
        "\nnon_empty_valid_cols:\n{}",
        non_empty_valid_cols
            .iter()
            .map(|(bn, terms)| format!(
                "{} ( {} )",
                bn,
                terms
                    .iter()
                    .map(|t| format!("{t}").replace('\n', "\\n"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
            .replace('\n', "\\n"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    let nestable_blank_nodes =
        evaluate_nestable_and_unreferenced_blank_nodes(&input.graph, unreferenced_blank_nodes);

    for subj in subjects(&input.graph) {
        if let SubjectRef::BlankNode(bn) = subj {
            if nestable_blank_nodes.contains(&bn) {
                continue;
            }
        }
        let level_triples = input.graph.triples_for_subject(subj);
        let mut parent = TSubjectCont::from(
            input,
            &input.graph,
            &non_empty_valid_cols,
            &nestable_blank_nodes,
            &col_involved_triples.borrow(),
            subj,
        );
        parent.create_graph_entry(
            input,
            &input.graph,
            &non_empty_valid_cols,
            &nestable_blank_nodes,
            &col_involved_triples.borrow(),
            level_triples,
        )?;
        tree_root.subjects.push(parent);
    }

    Ok(())
}
