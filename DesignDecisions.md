---
title: RDF/Turtle Pretty-Printer
breaks: false
---

## Output Structure Decisions

<!--
SPDX-FileCopyrightText: 2025 Robin Vobruba <hoijui.quaero@gmail.com>

SPDX-License-Identifier: Apache-2.0
-->

In this document, we go through a few decisions
that need to be taken by a Turtle pretty-printer,
either for the fixed format they provide,
of for the default format they prescribe.

We took a rather opinionated and strict stance,
as our main goal is to provide a more diff optimized option
to the Turtle pretty printers that are already available.

Many decisions are between two options,
most of which fall into two opposing optimization targets:

1. what looks pleasing to human eyes/is easy to read
2. diff minimization

If - in the case of a specific issue -
the decision is not abundantly clear in our eyes,
we go the route of making it configurable,
and usually choose the more human-readable option as the default.
We choose the default like that,
because even though we want to optimize for diffs,
in the end, Turtle is specifically made for humans,
and thus optimizing it primarily for machine consumption
would probably not be what most people want,
having chosen this format in the first place.

### Major Decisions

### New-Lines for Single-Leafed Nodes

tags: new-lines

Should visually single-leafed nodes in the Turtle syntax tree
also be separated by new-lines?

1. no new-lines:

    ```turtle
    <s> <p> <o> .

    <s2>
      <p2> <o2> ;
      <p3> <o3> ;
      .

    <s3> <p4> [ <p5> <o4> ] .
    ```

    sample diff:

    ```diff
    - <s> <p> <o> .
    + <s> <p> <o_x> .

    <s2>
    -   <p2> <o2> ;
    +   <p2>
    +     <o2> ,
    +     <o2_2> ;
      <p3> <o3> ;
      .

    - <s3> <p4> [ <p5> <o4> ] .
    + <s3>
    +   <p4> [
    +     <p5> <o4> ;
    +     <p6> <o5> ;
    +   ]
    +   .
    ```

2. max new-lines:

    ```turtle
    <s>
      <p>
        <o> ;
      .

    <s2>
      <p2>
        <o2> ;
      <p3>
        <o3> ;
      .

    <s3>
      <p4>
        [
          <p5>
            <o4> ;
        ] ;
      .
    ```

    sample diff:

    ```diff
    <s>
      <p>
    -     <o> ;
    +     <o_x> ;
      .

    <s2>
      <p2>
        <o2> ;
    +     <o2_2> ;
      <p3>
        <o3> ;
      .

    <s3>
      <p4>
        [
          <p5>
            <o4> ;
    +       <p6>
    +         <o7> ;
        ] ;
      .
    ```

This decision is important.
It is probably the single-biggest difference we would bring to the field,
in terms of practical difference in lines of (Turtle) code formatted,
compared to other pretty printers;
in fact to all of them, as far as we know.

The no-new-lines solution is standard,
and it is what people are used to see in Turtle.
Therefore we decided to use it as the default,
but make the new-lines approach available
(under the CLI flag `-n`, `--single-leafed-new-lines`).
We recommend using the new-lines option if you care more about a clean,
consistent set of rules and diff minimization,
and/or if the Turtle is primarily edited and viewed in a graphical way,
rather then as text.

#### Sorting Blank Nodes

tags: blank-nodes, sorting

When thinking about pretty printing Turtle ...

> **the hard problem** is:
_**How to sort blank nodes**_

Blank nodes usually have a different random ID
each time they get serialized or deserialized.
That means, that sorting them by ID
would make them jump around each time the ID changes.

1. Sorting by ID

    An example, showing one set of data in two serializations,
    and then the diff.
    We show the actual blank node label in a comment,
    because it would otherwise not be visible for an anonymized Turtle blank node.

    first serialization:

    ```turtle
    [ # _:198
      a ex:Person ;
      ex:name "Lynn"@en ;
    ]
      .
    
    [ # _:361
      a ex:Person ;
      ex:name "Bob"@en ;
    ]
      .
    
    [ # _:427
      a ex:Organization ;
      ex:name "Company X"@en ;
    ]
      .
    
    [ # _:754
      a ex:Thing ;
      ex:name "Hammer"@en ;
    ]
      .
    ```

    second serialization:

    ```turtle
    [ # _:276
      a ex:Organization ;
      ex:name "Company X"@en ;
    ]
      .
    
    [ # _:396
      a ex:Thing ;
      ex:name "Hammer"@en ;
    ]
      .
    
    [ # _:676
      a ex:Person ;
      ex:name "Bob"@en ;
    ]
      .
    
    [ # _:873
      a ex:Person ;
      ex:name "Lynn"@en ;
    ]
      .
    ```

    diff:

    ```diff
    [ # _:198 -> _:276
    -   a ex:Person ;
    -   ex:name "Lynn"@en ;
    +   a ex:Organization ;
    +   ex:name "Company X"@en ;
    ]
      .
    
    [ # _:361 -> _:396
    -   a ex:Person ;
    -   ex:name "Bob"@en ;
    +   a ex:Thing ;
    +   ex:name "Hammer"@en ;
    ]
      .
    
    [ # _:427 -> _:676
    -   a ex:Organization ;
    -   ex:name "Company X"@en ;
    +   a ex:Person ;
    +   ex:name "Bob"@en ;
    ]
      .
    
    [ # _:754 -> _:873
    -   a ex:Thing ;
    -   ex:name "Hammer"@en ;
    +   a ex:Person ;
    +   ex:name "Lynn"@en ;
    ]
      .
    ```

    So if this is not a solution, what could be?

2. Hashing

    The first thing that comes to the mind of an IT person would be:
    to hash _the content_ of the blank node.
    The content here would mean:
    What is visible within the blank-node.
    That in itself is very hard to do,
    because there can be cycles,
    and other blank nodes can appear with their labels, and so on.
    We believe, it would not be realistically doable in the general case.
    Yet, even if it would be doable, it would not be practical,
    because as soon as any small thing changes within a blank node,
    so does its hash, and it would jump up or down again.

    So if this is _also_ not a solution, is there anything left?

3. Assigning IDs

    The next obvious solution in line, would be to introduce IDs.
    Now, this definitely solves the issue,
    but it is a very drastic thing to do,
    as it creates new potential issues:

    1. introducing additional RDF data
    2. creates additional visual noise in the (Turtle) serialization
    3. if one does not know what this ID means/is good for,
        it will likely look confusing
    4. When c&p a blank node, one has to know and remember
        to edit this ID for the new node
    5. When introducing a new blank node in-between existing ones,
        and the previous and next blank nodes sorting ID are just one apart,
        one would have to change the ID of one till many other nodes,
        creating more diff and visual noise,
        and likely merge-conflicts between different (git) branches

    an example, already sorted; note the `prtr:sortingId`:

    ```turtle
    @prefix prtr: <http://w3id.org/oseg/ont/prtr#> .

    ex:anonymous
      a schema:Text ;
      schema:author
        [
          prtr:sortingId 2 ;
          schema:name "Robert Polson" ;
        ] ;
      .

    [
      a schema:Person ;
      prtr:sortingId 1 ;
      schema:name "Micha Maloun" ;
    ] .

    [ prtr:sortingId 100 ] .

    [
      a schema:Person ;
      prtr:sortingId 1800 ;
      schema:name "Jane Doe" ;
    ] .
    ```

4. Collection (global)

    This is an other approach that introduces new RDF data,
    but in a single, central location only, as a [Collection].
    It defines a fixed order of terms,
    as does a _list_ or _array_ in most programming languages.
    We would reference all blank nodes in a single such collection,
    in the same order in which they should appear in the file
    (though it only matters within one level of hierarchy).

    sample data:

    ```turtle
    <data-set>
      prtr:order (
        _:576
        _:796
        _:176
        _:473
        ) ;
      .

    _:576
      a ex:Organization ;
      ex:name "Company X"@en ;
      .
    
    _:796
      a ex:Thing ;
      ex:name "Hammer"@en ;
      .
    
    _:176
      a ex:Person ;
      ex:name "Bob"@en ;
      .
    
    _:473
      a ex:Person ;
      ex:name "Lynn"@en ;
      .
    ```

    diff (no data change):

    ```diff
    <data-set>
      prtr:order (
    -     _:576
    -     _:796
    -     _:176
    -     _:473
    +     _:631
    +     _:275
    +     _:728
    +     _:892
        ) ;
      .

    - _:576
    + _:631
      a ex:Organization ;
      ex:name "Company X"@en ;
      .
    
    - _:796
    + _:275
      a ex:Thing ;
      ex:name "Hammer"@en ;
      .
    
    - _:176
    + _:728
      a ex:Person ;
      ex:name "Bob"@en ;
      .
    
    - _:473
    + _:892
      a ex:Person ;
      ex:name "Lynn"@en ;
      .
    ```

    The issues with this approach:

    1. introducing a small bit of additional RDF data
    2. creates a little additional visual noise in the (Turtle) serialization
    3. if one does not know what this collection means/is good for,
        it will likely look confusing
    4. When c&p a blank node, one has to know and remember to add its ID
        to the collection in the right place
    5. While it happens in a single place in the file,
        all the otherwise unreferenced blank nodes will be nested in the collection,
        and all the nested ones will then be labelled,
        and these labels will change (and thus be part of the diff)
        on each re-serialization.

       Because the most common way to use blank nodes is to nest them,
       most will turn into labelled ones, and thus their labels/IDs
       may change on each re-serialization.

    Mainly because of point 5.,
    we do not regard this as a viable option.

5. [Collection] (local)

    Very similar to the last option,
    instead of a single (file-global) collection,
    this introduces a local collection everywhere where there are multiple blank-nodes.

    The drawbacks of this solution is similar to the ones of the global Collection.
    We neither consider this a viable option.

As this is probably the second most important part of a Turtle pretty printer -
even though we do not have a fully satisfying solution -
we want to take a stance:
We chose to use number 3 (Assigning IDs) as our go-to solution.
We do not, however,
automatically introduce such IDs by default.
To introduce them -
which will fix the sorting of IDs
to be in the order in which they appear in the input -
one needs to specifically request that.
We do it this way,
as we deem it unfit for a pretty printer to introduce new data by default.

#### Comments

tags: comments, sorting

Comments!
With which we do not refer to RDF comments (like `rdfs:comment`),
but Turtle Syntax comments,
which are very similar to comments in Python.

Comments in Turtle are started with a `#`,
and continue to the end of the line.

samples:

```turtle
# SPDX-FileCopyrightText: Organization-X
#
# SPDX-License-Identifier: CC-BY-SA-4.0 AND Apache-2.0

# This file contains the X Ontology.

# Base comment
BASE # Base comment 2
    <http://example.org/>
# Prefix comment
@prefix ex: </new/> . # Prefix comment 2

# Classes

#<commented> <out> <code> .

#  Primary Classes

# Subject comment
<s> # Subject comment 2
  # Predicate comment
  a # Predicate comment 2
    # Object comment
    owl:Class , # Object comment 2
  # Subject comment 3
    # Object2 comment
    skos:Concept ; # Object2 comment 2
  # Subject comment 4
  # Commented out predicate and object:
  #<p> # Commented out predicate comment
  #  <o> ;
  .

# Class 2 comment
<s2> <p2> <o2> .
# Commented out predicate and object - 2:
#  <p3> # Commented out predicate comment
#    <o3> ;

# Predicates

# ...
```

As this example tries to show (quite over-excessively so, of course),
is that comments usually have a scope,
or say, they are associated with a part of the code
(in this case: the actual RDF data).
These comments are almost exclusively targeted at humans,
and they are written by humans too.
Most human readers can make out quite quickly the scope/target of a comment in the code,
for machines though, that is a hard task,
and is not possible without heuristics and a lot of guessing.
For all practical purposes, we can think of it as impossible (for machines).

Without a mapping of comments to parts of the code,
sorting of the code parts while retaining the comments
**in a location where they make sense to a human**
is not possible.

Because we want to clearly lean towards diff minimization,
we definitely want to have sorting,
and thus the only way to deal with comments,
is to remove support for them entirely.

Given this,
we still have to decide how exactly to go about this,
as we could:

1. Silently drop/ignore all comments (by default)
2. Fail if any comments are detected in the input,
    and suggest to convert them to RDF comments
3. ... with an option to ignore them forcefully,
    which has the same effect as 1.
4. Have an automated way to convert them to RDF comments
5. Fail if comments are detected,
    but suggest to use the optional process from 4.

The process of auto-converting to RDF comments
used in 4. and 5. would be nice to have,
but is tedious to write, test and get right
in a way that feels right for most humans.
It goes beyond what we could do in this project,
but is an interesting [feature for the future](TODO link to issue on the new repo for this software) of this software.

TODO Document how to convert/refactor Turtle comments into RDF ones.

#### Nested vs Labelled Blank Nodes

tags: blank-nodes

Max Nested vs All Labelled Blank Nodes

1. Max nested

    ```turtle
    <dorm1>
      rdfs:label "Dormitory 1"@en ;
      ex:spokesPerson _:lynn ;
      ex:students [
        a rdf:Bag ;
        rdfs:label "Students A"@en ;
        ex:student
          [ ex:name "Caroline C."@en ] ,
          _:bob ,
          _:lynn ;
      ] ;
      .
    
    <dorm2>
      rdfs:label "Dormitory 2"@en ;
      ex:spokesPerson _:bob ;
      ex:students [
        a rdf:Bag ;
        rdfs:label "Students B"@en ;
        ex:student
          [ ex:name "Chubi D."@en ] ,
          [ ex:name "Hamster R."@en ] ,
          [ ex:name "Lolo L."@en ] ;
      ] ;
      .
    
    _:bob ex:name "Bob Haugen"@en .
    
    _:lynn ex:name "Lynn Foster"@en .
    
    _:waldi ex:name "Waldi W."@en .
    ```

2. All Labelled

    ```turtle
    <dorm1>
      rdfs:label "Dormitory 1"@en ;
      ex:spokesPerson _:lynn ;
      ex:students _:studentsA ;
      .

    <dorm2>
      rdfs:label "Dormitory 2"@en ;
      ex:spokesPerson _:bob ;
      ex:students _:studentsB ;
      .

    _:studentsA
      a rdf:Bag ;
      rdfs:label "Students A"@en ;
      ex:student
        _:caroline ,
        _:bob ,
        _:lynn ;
      .

    _:studentsB
      a rdf:Bag ;
      rdfs:label "Students B"@en ;
      ex:student
        _:chubi ,
        _:hamster ,
        _:lolo ;
      .

    _:caroline ex:name "Caroline C."@en .

    _:chubi ex:name "Chubi D."@en .

    _:hamster ex:name "Hamster R."@en .

    _:lolo ex:name "Lolo L."@en .
    
    _:bob ex:name "Bob Haugen"@en .
    
    _:lynn ex:name "Lynn Foster"@en .
    
    _:waldi ex:name "Waldi W."@en .
    ```

There is no mentionable difference between these
regarding diff optimization per se,
though the max nested approach removes a big chunk
of the issue of sorting blank-nodes,
simply because there are not many on the same level
(e.g. blank-node subjects in the root,
or as objects of the same subject-predicate pair).
That already makes the situation much less messy.
That is only relevant if `prtr:sortingId` is not used.

For human readability,
we deem it a clear case of the _max nested_ approach being far superior.

Thus, _max nested_ wins in all regards,
and is therefore what we use,
without even the option to choose _max labelled_.

### Intermediate Decisions

#### Sorting - Special Predicates

tags: sorting

Currently, we sort `a` (aka `rdf:type`) at the top,
then prefixed named nodes in alphabetic order,
then non-prefixed(/by IRI) named nodes.

In short: It is just sorted by type and then alphabetically within each type.

We could however introduce sorting for some other special named nodes,
similar as we already do for `a`, if they appear.

Possible ways of handling this are:

1. hard-code those predicates (and their order),
2. give a few predefined sets (like RDF, OWL, SKOS, SHACL and ShEx), or
3. allow the user to define such a set themselves.

We decided to go with both 2. and 3. in this matter.

#### Auto-Insert Sorting IDs

tags: blank-nodes, sorting, prtr

TODO tag: _should-have_

Whether to automatically insert `prtr:sortingId`
for blank nodes that do not yet have such an ID.

Because this means, changing the actual RDF data -
which is a clear no-go for a pretty-printer -
we don't do this by default,
but we allow to enable it optionally.

#### Term Types

tags: sorting

This refers to:
Which types of terms should be sorted before which other.
For example:

1. Should prefixed named nodes come before those given with a full IRI?
2. Should non-prefixed named nodes always be alphabetically sorted,
    or should relative ones be a different type then absolute ones
    (and therefore always be before/after the other type)?
3. Should blank nodes come before or after non-blank ones?
4. Should anonymous and labelled blank nodes be different categories?
    (I think here the answer can only be:
    yes (if prtr is disabled),
    because otherwise there would be no way to compare them in a meaningful way,
    as anonymous ones have no label (in Turtle syntax))
5. Should Collections come before or after blank nodes, named nodes, ...?
6. Same question for Literals
7. how to compare Turtle Syntax literas (BOOL, INTEGER, DECIMAL, DOUBLE)
    with the "stringified" ones?
    should they all be compared by value first (and then language/datatype),
    or should the Turtle Syntax ones be a separate type from the others,
    and therefore be grouped (before/after)?

Examples:

```turtle
@base <http://example.net/> .
@prefix ex: <http://example.net/> .
@prefix : <http://example.net/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

# ## Subjects sorting order

# 1. empty prefixed
:s a <to> .

# 2. prefixed (non-empty)
ex:s a <to> .

# 3. relative IRI
<s> a <to> .

# 4. absolute IRI
<http://example.net/s> a <to> .

# 5. Collection
( 1 2 3 ) a <to> .

# 6. Anonymous, empty blank-node
#    (though this is theoretical, as we don't generate these at all) 
[] a <to> .

# 7. Anonymous, non-empty blank-node
[]
  a ex:Bag ;
  rdfs:label ex:Bag ;
  .
[ a ex:Bag ]
  rdfs:label ex:Bag ;
  .
[
  a ex:Bag ;
  rdfs:label ex:Bag ;
]
  .

# 8. Labelled blank-node
_:123abc a <to> .


# ## Predicates sorting order

# Same as Subjects, though only points 1 to 4,
# because predicates can't be blank-nodes.


# ## Objects sorting order

# Same as Subjects, but additionally sort literals
# in-between points 4 (absolute IRI) and 5 (Collection),
# in the following order:

<s>
  <p>

# 1. simple string (no language, no datatype)
    "some string" ,

# 2. language tagged string
    "some string"@en ,
    "some string"@en-US ,

# 3. datatype annotated string
    "\_/'_'\_/"^^ex:myDatatype ,
    "\_/'_'\_/"^^<http://example.org#myDatatype> ,

# 4. Turtle native literals
#
#     1. boolean
    false ,
    true ,
#   == "true"^^xsd:boolean
#   == "123"^^<http://www.w3.org/2001/XMLSchema#boolean>
#
#     2. integer
    123 ,
#   == "123"^^xsd:integer
#
#     3. decimal
    123.345 ,
#   == "123.345"^^xsd:decimal
#
#     4. double
    123.7e3 ;
#   == "123.7e3"^^xsd:double

  .
```

### Minor Decisions

#### Nested Bracket Location

tags: blank-nodes

If a blank node is nested/inlined as an object,
should it's opening bracket be placed ...

On the same line as the predicate:

```turtle
<s>
  ex:students [
    a rdf:Bag ;
    rdfs:label "Students"@en ;
  ] ;
```

sample diff:

```diff
<s>
-  ex:students [
-    a rdf:Bag ;
-    rdfs:label "Students"@en ;
-  ] ;
+  ex:students
+    ex:class5B
```

or on a new line:

```turtle
<s>
  ex:students
    [
      a rdf:Bag ;
      rdfs:label "Students"@en ;
    ] ;
```

sample diff:

```diff
<s>
  ex:students
-    [
-      a rdf:Bag ;
-      rdfs:label "Students"@en ;
-    ] ;
+    ex:class5B
```

For human readability, we think that the _same line_ way is clearly the winner.
For diff optimization, the _new-line_ way is clearly the winner.

We chose to make this consistent with the way a named node object is placed:
If the predicate has a single object,
and the setting for putting a single object on a new line is `false`,
we put the opening bracket on she same line,
else on a new one.

#### Empty Anonymous Blank Nodes

tags: blank-nodes

Whether to use empty anonymous blank nodes ('[]')
at the Turtle syntax tree root?

1. Filled

    ```turtle
    [
      a rdf:Bag ;
      rdfs:label "H Students"@en ;
      rdfs:comment "These are allowed to enter the chemistry lab H"@en ;
    ]
      .
    ```

2. Partly filled

    ```turtle
    [ a rdf:Bag ]
      rdfs:label "F Students"@en ;
      rdfs:comment "These are allowed to enter the chemistry lab F"@en ;
      .
    
    [
      a rdf:Bag ;
      rdfs:label "G Students"@en ;
    ]
      rdfs:comment "These are allowed to enter the chemistry lab G"@en ;
      .
    ```

3. Empty

    ```turtle
    []
      a rdf:Bag ;
      rdfs:label "A Students"@en ;
      rdfs:comment "These are allowed to enter the chemistry lab A"@en ;
      .
    ```

The second style (partly filled) is not an option,
because the pretty printer would not know what to put inside and what outside.

Regrading diff optimization, _full_ and _empty_ are equal.
One clear argument pro _full_,
is that it works both for root-level anonymous (== unreferenced) blank nodes,
as well as for nested/inlined ones (referenced exactly once).
_Empty_ only works on the root level.
An other point pro _full_,
is that it more naturally conveys to the human eye what is part of the blank node.
We therefore settle for _full_,
without even the option to choose empty.

#### Traditional vs SPARQL Syntax

tags: prefix, base

Whether to use traditional or SPARQL syntax for `@prefix` and `@base`.

traditional Turtle style:

```turtle
@base <http://example.net/> .
@prefix ex: <http://example.org/> .
```

vs new SPARQL style:

```turtle
BASE <http://example.net/>
PREFIX ex: <http://example.org/>
```

The traditional syntax is much more prevalent out there,
and visually somewhat more in-line with most of the prefixes and local names
being made up of predominantly minor-case letters.
The SPARQL syntax allows for direct copying and pasting
from a Turtle file to a SPARQL query.

We do not feel strongly about this,
but default on the traditional syntax,
with an option to use the SPARQL one.

#### Multi-Line Quoting

tags: strings

Whether/when to use triple-quotes for multi-line strings?

triple-quotes with actual new-lines:

```turtle
 <s>
   <p>
     """Lorem ipsum dolor sit amet, consectetur adipiscing elit.
Nullam turpis leo, convallis in aliquam at, dictum ut nisi.
Donec vulputate ornare bibendum.
Nulla viverra viverra sapien sagittis pretium.
Interdum et malesuada fames ac ante ipsum primis in faucibus.
Aliquam id leo euismod purus eleifend cursus.
Fusce nibh felis, tincidunt vel justo a, iaculis sagittis justo.
Aenean feugiat non diam ut pretium.
Integer nec ullamcorper ligula.
Aliquam pharetra tellus vitae laoreet pellentesque.
Sed sed massa ut lacus congue convallis eu vel nisi.
Aenean sit amet felis tellus.
Nam euismod fermentum est ut eleifend.
Nullam ligula arcu, porta eget cursus ac, vehicula eget sapien.
Aliquam convallis odio at arcu vestibulum, ac commodo ex pulvinar.
Vestibulum varius ullamcorper lorem, at pulvinar sapien tincidunt nec.
Proin sit amet erat sodales, mollis leo a, posuere quam.""\"
\""""
    ;
  .
```

single quotes, containing quoted new-lines (`\n`):

```turtle
 <s>
   <p>
     "Lorem ipsum dolor sit amet, consectetur adipiscing elit.\nNullam turpis leo, convallis in aliquam at, dictum ut nisi.\nDonec vulputate ornare bibendum.\nNulla viverra viverra sapien sagittis pretium.\nInterdum et malesuada fames ac ante ipsum primis in faucibus.\nAliquam id leo euismod purus eleifend cursus.\nFusce nibh felis, tincidunt vel justo a, iaculis sagittis justo.\nAenean feugiat non diam ut pretium.\nInteger nec ullamcorper ligula.\nAliquam pharetra tellus vitae laoreet pellentesque.\nSed sed massa ut lacus congue convallis eu vel nisi.\nAenean sit amet felis tellus.\nNam euismod fermentum est ut eleifend.\nNullam ligula arcu, porta eget cursus ac, vehicula eget sapien.\nAliquam convallis odio at arcu vestibulum, ac commodo ex pulvinar.\nVestibulum varius ullamcorper lorem, at pulvinar sapien tincidunt nec.\nProin sit amet erat sodales, mollis leo a, posuere quam.\"\"\"\n\""
    ;
  .
```

Triple-quotes win here, clearly,
both in human readability as well as in diff minimization.

#### Triple-Quoted Strings (no new lines)

tags: strings

Whether, when and how to use triple quoted strings for quoting
if there are no new lines.

```turtle
 <s>
   <p>
     """foo'"bar"""
    ;
```

vs

```turtle
 <s>
   <p>
     "foo'\"bar"
    ;
```

This is a tricky, but not very important decision to take.
In order to prevent changes back and forth from single to triple quoting
when adding or removing parts that requrie quoting,
we will always use single quoting here.
Feel free to bring up good arguments agasint it in an issue, please.

#### Single-Quoted vs Double-Quoted

tags: strings

The question here is,
whether to use `'` vs `"` quoted string literals.

```turtle
 <s>
   <p>
     "We talk about 'foo' and \"bar\"." ,
     'We talk about \'foo\' and "bar".' ,
     """We talk about 'foo' and "bar".""" ,
     '''We talk about 'foo' and "bar".'''
    ;
```

vs

```turtle
 <s>
   <p>
     "foo\"bar"
     "foo\"bar"
    ;
```

Turtle (supposedly) allows to use either,
because using one measn, the other can be used within,
without requiring quoting.

To us, this seems like a small advantage to gain,
payd for with the additional complexity and ambiguity,
and thus we decide to go for always using one,
without the option to choose the other.
We choose `"`, because it is kind of the default,
and much more widely used in the data that is out there,
by other pretty-printers, and even in coding in general.

#### Prefix vs Base

tags: prefix, base

If a prefix and the base cover the same namespace,
which one to prefer when formatting?

```turtle
@base       <http://example.org/> .
@prefix ex: <http://example.org/> .

<s> <p> <o> .
```

vs

```turtle
@base       <http://example.org/> .
@prefix ex: <http://example.org/> .

ex:s ex:p ex:o .
```

We chose to fail-fast already at the parsing stage,
if this is the case.

#### Prefixes with Equal Namespace

tags: prefix

If multiple prefixes cover the same namespace prefix
which one to prefer when formatting?

```turtle
@prefix a: <http://example.org/> .
@prefix b: <http://example.org/> .

a:s a:p a:o .
```

vs

```turtle
@prefix a: <http://example.org/> .
@prefix b: <http://example.org/> .

b:s b:p b:o .
```

We chose to fail-fast already at the parsing stage,
if this is the case.

#### Prefix Redefinition

tags: prefix

What to do on re-definition of a prefix?

```turtle
@prefix a: <http://example.org/> .

a:s66 a:p66 a:o66 .
a:s2 a:p2 a:o2 .
a:s7 a:p7 a:o7 .

@prefix a: <http://google.com/> .

a:s55 a:p55 a:o55 .
a:s4 a:p4 a:o4 .
a:s1 a:p1 a:o1 .
```

This means, that the prefix stands for different namespaces
in the upper and the lower part of the data.
The issue then is,
that if we want to keep the prefix definitions,
we can only sort triples within the sections divided by them:

```turtle
@prefix a: <http://example.org/> .

a:s2 a:p2 a:o2 .
a:s7 a:p7 a:o7 .
a:s66 a:p66 a:o66 .

@prefix a: <http://google.com/> .

a:s1 a:p1 a:o1 .
a:s4 a:p4 a:o4 .
a:s55 a:p55 a:o55 .
```

While prefix redefinition is valid Turtle,
we consider it:

- bad practise
- of no practical benefit
- confusing to human eyes, and therefore error phrone
- limits the overall potential impact of the pretty-printing on diff minimization

Thus, we do not accept such input,
and fail-fast already at the parsing stage.
If this happens,
we also issue a warning message,
suggesting to manually refactor the input accordingly
before running the pretty printer on it.
This refactoring would usually be one of:

- splitting up the content, so each section becomes a separate file
- replace each prefix redefinition with a separate (custom/local) prefix

---

The following code snippet shows a very borderline,
but somewhat valid use-case.
We still consider it clearly bad practise to do this,
and recommend to splt this file into multiple ones,
one for each base/prefix redefinition.

```turtle
@prefix mything: <http://aaa.org/> .
@base <http://github.com/user_x/aaa/> .

mything:okhProject okh:bom <bom.csv> .
mything:okhProject okh:readme <README.md> .

@prefix mything: <http://bbb.org/> .
@base <http://github.com/user_x/bbb/> .

mything:okhProject okh:bom <bom.csv> .
mything:okhProject okh:readme <README.md> .
```

### Base Redefinition

tags: base

What to do on re-definition of base?

```turtle
@base <http://github.com/user_x/aaa/> .

<s66> <p66> <o66> .
<s2> <p2> <o2> .
<s7> <p7> <o7> .

@base <http://github.com/user_x/bbb/> .

<s55> <p55> <o55> .
<s4> <p4> <o4> .
<s1> <p1> <o1> .
```

Same as with prefix redefinition,
the issue here is,
that if we want to keep the base definitions,
we can only sort triples within the sections divided by them:

```turtle
@base <http://github.com/user_x/aaa/> .

<s2> <p2> <o2> .
<s7> <p7> <o7> .
<s66> <p66> <o66> .

@base <http://github.com/user_x/bbb/> .

<s1> <p1> <o1> .
<s4> <p4> <o4> .
<s55> <p55> <o55> .
```

While base redefinition is valid Turtle,
we consider it:

- bad practise
- of almost no practical benefit
- confusing to human eyes, and therefore error phrone
- limits the overall potential impact of the pretty-printing on diff minimization

Thus, we do not accept such input,
and fail-fast already at the parsing stage.
If this happens,
we also issue a warning message,
suggesting to manually refactor the input accordingly
before running the pretty printer on it.
This refactoring would usually be one of:

- splitting up the content, so each section becomes a separate file
- convert relative IRIs into absolute ones
- replace each base definition with a separate (custom/local) prefix

[Collection]: https://ontola.io/blog/ordered-data-in-rdf
