// SPDX-FileCopyrightText: 2025 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;
use std::rc::Rc;
use std::sync::LazyLock;

use oxrdf::BlankNodeRef;
use oxrdf::Graph;
use oxrdf::LiteralRef;
use oxrdf::NamedNodeRef;
use oxrdf::SubjectRef;
use oxrdf::TermRef;
use oxrdf::TripleRef;
use oxrdf::{NamedNode, Term};

use crate::input::Input;

const NS_RDF: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";

static NN_RDF_FIRST: LazyLock<NamedNode> =
    LazyLock::new(|| NamedNode::new(format!("{NS_RDF}first")).unwrap());
static NN_RDF_REST: LazyLock<NamedNode> =
    LazyLock::new(|| NamedNode::new(format!("{NS_RDF}rest")).unwrap());
static NN_RDF_NIL: LazyLock<NamedNode> =
    LazyLock::new(|| NamedNode::new(format!("{NS_RDF}nil")).unwrap());
pub static NN_RDF_TYPE: LazyLock<NamedNode> =
    LazyLock::new(|| NamedNode::new(format!("{NS_RDF}type")).unwrap());
static NN_RDF_LIST: LazyLock<NamedNode> =
    LazyLock::new(|| NamedNode::new(format!("{NS_RDF}List")).unwrap());

static T_RDF_FIRST: LazyLock<Term> = LazyLock::new(|| Term::NamedNode(NN_RDF_FIRST.clone()));
static T_RDF_REST: LazyLock<Term> = LazyLock::new(|| Term::NamedNode(NN_RDF_REST.clone()));
static T_RDF_NIL: LazyLock<Term> = LazyLock::new(|| Term::NamedNode(NN_RDF_NIL.clone()));
static T_RDF_TYPE: LazyLock<Term> = LazyLock::new(|| Term::NamedNode(NN_RDF_TYPE.clone()));
static T_RDF_LIST: LazyLock<Term> = LazyLock::new(|| Term::NamedNode(NN_RDF_LIST.clone()));

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum TSubject<'graph> {
    NamedNode(NamedNodeRef<'graph>),
    BlankNodeLabel(TBlankNodeRef<'graph>),
    // BlankNodeAnonymous(TBlankNode<'graph>),
    BlankNodeAnonymous(TBlankNode<'graph>),
    Collection(TCollection<'graph>),
    Triple((Box<TSubject<'graph>>, TPredicate<'graph>, TObject<'graph>)),
}

impl<'graph> From<SubjectRef<'graph>> for TSubject<'graph> {
    fn from(other: SubjectRef<'graph>) -> Self {
        match other {
            SubjectRef::NamedNode(named_node_ref) => Self::NamedNode(named_node_ref),
            SubjectRef::BlankNode(blank_node_ref) => {
                Self::BlankNodeLabel(TBlankNodeRef(blank_node_ref))
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TSubjectCont<'graph> {
    pub subject: TSubject<'graph>,
    pub predicates: Vec<TPredicateCont<'graph>>,
}

impl<'graph> From<SubjectRef<'graph>> for TSubjectCont<'graph> {
    fn from(other: SubjectRef<'graph>) -> Self {
        Self {
            subject: TSubject::from(other),
            predicates: Vec::new(),
        }
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

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TPredicate<'graph>(pub NamedNodeRef<'graph>);

impl<'graph> From<NamedNodeRef<'graph>> for TPredicate<'graph> {
    fn from(other: NamedNodeRef<'graph>) -> Self {
        Self(other)
    }
}

impl Ord for TPredicate<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.as_str().cmp(other.0.as_str())
    }
}

impl PartialOrd for TPredicate<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TPredicateCont<'graph> {
    pub predicate: TPredicate<'graph>,
    pub objects: Vec<TObject<'graph>>,
}

impl<'graph> From<NamedNodeRef<'graph>> for TPredicateCont<'graph> {
    fn from(other: NamedNodeRef<'graph>) -> Self {
        Self {
            predicate: TPredicate::from(other),
            objects: Vec::new(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
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

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TLiteralRef<'graph>(pub LiteralRef<'graph>);

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

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum TObject<'graph> {
    NamedNode(NamedNodeRef<'graph>),
    BlankNodeLabel(TBlankNodeRef<'graph>),
    BlankNodeAnonymous(TBlankNode<'graph>),
    Collection(TCollection<'graph>),
    Literal(TLiteralRef<'graph>),
    Triple(
        (
            Box<TSubject<'graph>>,
            TPredicate<'graph>,
            Box<TObject<'graph>>,
        ),
    ),
}

impl<'graph> From<TermRef<'graph>> for TObject<'graph> {
    fn from(other: TermRef<'graph>) -> Self {
        match other {
            TermRef::NamedNode(named_node_ref) => {
                if named_node_ref == NN_RDF_NIL.as_ref() {
                    Self::Collection(TCollection::Empty)
                } else {
                    Self::NamedNode(named_node_ref)
                }
            }
            TermRef::BlankNode(blank_node_ref) => {
                Self::BlankNodeLabel(TBlankNodeRef(blank_node_ref))
            }
            TermRef::Literal(literal_ref) => Self::Literal(TLiteralRef(literal_ref)),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TCollectionRef<'graph> {
    pub node: TBlankNode<'graph>,
    pub rest: Vec<TObject<'graph>>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum TCollection<'graph> {
    WithContent(TCollectionRef<'graph>),
    Empty,
}

trait PredicatesStore<'graph> {
    fn get_predicates_mut<'us>(&'us mut self) -> &'us mut Vec<TPredicateCont<'graph>>
    where
        'graph: 'us;

    fn create_graph_entry<'us>(
        &'us mut self,
        g_main: &'graph Graph,
        non_empty_valid_cols: &HashMap<BlankNodeRef<'graph>, Vec<TermRef<'graph>>>,
        nestable_blank_nodes: &HashSet<BlankNodeRef<'graph>>,
        col_involved_triples: &Vec<TripleRef<'graph>>,
        level_triples: impl Iterator<Item = TripleRef<'graph>>,
    ) -> BoxResult<()>
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
                SubjectRef::NamedNode(_) => (),
            }
            predicate_objects
                .entry(triple.predicate)
                .or_insert_with(Vec::new)
                .push(triple.object);
        }
        for (predicate, objects) in predicate_objects {
            let mut predicate = TPredicateCont::from(predicate);
            for object in objects {
                let mut t_object = TObject::from(object);
                if let TermRef::BlankNode(bn) = object {
                    if let Some(col) = non_empty_valid_cols.get(&bn) {
                        let mut tbn = TBlankNode::from(bn);
                        tbn.create_graph_entry(
                            g_main,
                            non_empty_valid_cols,
                            nestable_blank_nodes,
                            col_involved_triples,
                            g_main.triples_for_subject(bn),
                        )?;
                        t_object = TObject::Collection(TCollection::WithContent(TCollectionRef {
                            node: tbn,
                            rest: col.iter().map(|term| TObject::from(*term)).collect(),
                        }));
                    } else if nestable_blank_nodes.contains(&bn) {
                        let mut tbn = TBlankNode::from(bn);
                        tbn.create_graph_entry(
                            g_main,
                            non_empty_valid_cols,
                            nestable_blank_nodes,
                            col_involved_triples,
                            g_main.triples_for_subject(bn),
                        )?;
                        t_object = TObject::BlankNodeAnonymous(tbn);
                    }
                }
                predicate.objects.push(t_object);
            }
            self.get_predicates_mut().push(predicate);
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug)]
pub struct TRoot<'graph> {
    pub subjects: Vec<TSubjectCont<'graph>>,
}

impl TRoot<'_> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            subjects: Vec::new(),
        }
    }
}

impl Default for TRoot<'_> {
    fn default() -> Self {
        Self::new()
    }
}

// enum CollectionEntry<'graph> {
//     Between(TermRef<'graph>, BlankNodeRef<'graph>),
//     End(TermRef<'graph>),
//     Empty,
// }

fn remove_sorted_indices<T>(
    v: impl IntoIterator<Item = T>,
    indices: impl IntoIterator<Item = usize>,
) -> Vec<T> {
    let v = v.into_iter();
    let mut indices = indices.into_iter();
    let mut i = match indices.next() {
        None => return v.collect(),
        Some(i) => i,
    };
    let (min, max) = v.size_hint();
    let mut result = Vec::with_capacity(max.unwrap_or(min));

    for (j, x) in v.into_iter().enumerate() {
        if j == i {
            if let Some(idx) = indices.next() {
                i = idx;
            }
        } else {
            result.push(x);
        }
    }

    result
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

// fn extract_collection_entries<'graph>(
//     input: &'graph Input,
//     blank_node_subjects: &Vec<BlankNodeRef<'graph>>,
//     blank_node_objects: &Vec<BlankNodeRef<'graph>>,
// ) -> HashMap<BlankNodeRef<'graph>, CollectionEntry<'graph>> {
//     let referenced_multiple_times = extract_duplicates(blank_node_objects);

//     let mut collection_entries = HashMap::new();
//     let mut blank_node_list_subjects_indices = vec![];
//     for (sidx, subj) in blank_node_subjects.iter().enumerate() {
//         let first_opt = input
//             .graph
//             .object_for_subject_predicate(*subj, NN_RDF_FIRST.as_ref());
//         if let Some(first) = first_opt {
//             let rest_opt = input
//                 .graph
//                 .object_for_subject_predicate(*subj, NN_RDF_REST.as_ref());
//             if let Some(rest_val) = rest_opt {
//                 let triples = input.graph.triples_for_subject(*subj).count();
//                 let mut is_list_entry = false;
//                 if triples == 2 {
//                     is_list_entry = true;
//                 } else if triples == 3 {
//                     let type_opt = input
//                         .graph
//                         .object_for_subject_predicate(*subj, NN_RDF_TYPE.as_ref());
//                     if let Some(TermRef::NamedNode(type_val)) = type_opt {
//                         if type_val == NN_RDF_LIST.as_ref() {
//                             is_list_entry = true;
//                         }
//                     }
//                 }
//                 // ... Every other case means that there are extra triples in the collection/list entry,
//                 // besides the ones required for making it a collection/list.
//                 // If we would convert it into a Turtle syntax list,
//                 // these extra entries would be lost or have a (bank-node-)subject that would then be detached.

//                 if is_list_entry {
//                     if referenced_multiple_times.contains(subj) {
//                         // We can not convert to a Turtle-syntax collection
//                         // if a single (or more) element of the collection
//                         // is referenced more then once.
//                         // If one link in the collection is missing,
//                         // it will fail to assembled as a whole,
//                         // and thus will remain as a raw RDF list,
//                         // rather then a collection in Turtle syntax.
//                         continue;
//                     }
//                     let collection_entry = match rest_val {
//                         TermRef::BlankNode(rest) => {
//                             CollectionEntry::Between(first, rest)
//                         }
//                         TermRef::NamedNode(rest) => {
//                             if rest == NN_RDF_NIL.as_ref() {
//                                 CollectionEntry::End(first)
//                             } else {
//                                 // TODO FIXME Return an error
//                                 continue;
//                             }
//                         }
//                         TermRef::Literal(_rest) => {
//                             continue;
//                         }
//                     };
//                     blank_node_list_subjects_indices.push(sidx);
//                     collection_entries.insert(*subj, collection_entry);
//                 } else {
//                     // TODO;
//                 }
//             }
//         }
//     }
//     remove_sorted_indices(blank_node_subjects, blank_node_list_subjects_indices);
//     collection_entries
// }

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
            .objects_for_subject_predicate(cur, NN_RDF_FIRST.as_ref())
            .collect::<Vec<_>>();
        if firsts.len() != 1 {
            return None;
        }
        let first = *firsts.first().unwrap();
        let cur_subj = SubjectRef::BlankNode(cur);
        involved_triples.push(TripleRef::new(cur_subj, NN_RDF_FIRST.as_ref(), first));
        col.push(first);

        let rests = g_main
            .objects_for_subject_predicate(cur, NN_RDF_REST.as_ref())
            .collect::<Vec<_>>();
        if rests.len() != 1 {
            return None;
        }
        let rest = *rests.first().unwrap();
        involved_triples.push(TripleRef::new(cur_subj, NN_RDF_REST.as_ref(), rest));

        let types = g_main
            .objects_for_subject_predicate(cur, NN_RDF_TYPE.as_ref())
            .collect::<Vec<_>>();
        if types.len() > 1 && cur != start {
            return None;
        }
        let mut list_native_triples = 2;
        if types.contains(&T_RDF_LIST.as_ref()) {
            involved_triples.push(TripleRef::new(
                cur_subj,
                NN_RDF_TYPE.as_ref(),
                T_RDF_LIST.as_ref(),
            ));
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
                if nn_rest == NN_RDF_NIL.as_ref() {
                    break;
                }
                return None;
            }
            TermRef::Literal(lit_rest) => {
                eprintln!("Literal as collection chain element is invalid: {lit_rest}");
                return None;
            }
        }
    }
    Some(col)
}

fn evaluate_nestable_blank_nodes(g_main: &Graph) -> HashSet<BlankNodeRef<'_>> {
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
    let duplicate_obj_bns = extract_duplicates(&object_bns);
    subject_bns.retain(|bn| !duplicate_obj_bns.contains(bn));
    subject_bns.into_iter().collect()
}

// fn nest(
//     mut g_main: Rc<RefCell<Graph>>,
//     non_empty_valid_cols: &HashMap<BlankNode, Vec<Term>>,
//     nestable_blank_nodes: &HashSet<BlankNode>,
// ) {
//     TODO;
// }

fn extract_non_empty_collections<'graph>(
    g_main: &'graph Graph,
    involved_triples: &Rc<RefCell<Vec<TripleRef<'graph>>>>,
) -> HashMap<BlankNodeRef<'graph>, Vec<TermRef<'graph>>> {
    let mut col_starts = vec![];
    {
        for triple in g_main {
            if let SubjectRef::BlankNode(bn_subj) = triple.subject {
                if triple.predicate == NN_RDF_FIRST.as_ref() {
                    let rest_refs_to_subj = g_main
                        .subjects_for_predicate_object(NN_RDF_REST.as_ref(), bn_subj)
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

fn filter_blank_node_subjects<'a>(subjects: &Vec<SubjectRef<'a>>) -> Vec<BlankNodeRef<'a>> {
    subjects
        .iter()
        .filter_map(|subject| match subject {
            SubjectRef::BlankNode(blank_node) => Some(*blank_node),
            SubjectRef::NamedNode(_) => None,
        })
        .collect()
}

fn filter_blank_node_objects(input: &Input) -> Vec<BlankNodeRef<'_>> {
    input
        .graph
        .iter()
        .filter_map(|triple| {
            if let TermRef::BlankNode(blank_node) = triple.object {
                Some(blank_node)
            } else {
                None
            }
        })
        .collect()

    // triple.subject, triple.predicate, triple.object)

    // let mut blank_nodes = vec![];
    // for predicates in input.graph.iter().map(|triple| triple.object) {
    //     for objects in predicates.values() {
    //         for object in objects {
    //             if object.is_blank_node() {
    //                 blank_nodes.push(object.clone());
    //             }
    //         }
    //     }
    // }
    // blank_nodes
}

type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;
type BoxResult<T> = Result<T, BoxError>;

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
    input: &'graph Input,
) -> BoxResult<()>
where
    'graph: 'tree,
{
    // let g_main = Rc::new(RefCell::new(input.graph.clone()));

    let col_involved_triples = Rc::new(RefCell::new(Vec::new()));
    let non_empty_valid_cols = extract_non_empty_collections(&input.graph, &col_involved_triples);
    let nestable_blank_nodes = evaluate_nestable_blank_nodes(&input.graph);
    // nest(g_main.clone(), &non_empty_valid_cols, &nestable_blank_nodes);

    for subj in subjects(&input.graph) {
        if let SubjectRef::BlankNode(bn) = subj {
            if nestable_blank_nodes.contains(&bn) {
                continue;
            }
        }
        let level_triples = input.graph.triples_for_subject(subj);
        let mut parent = TSubjectCont::from(subj);
        parent.create_graph_entry(
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
