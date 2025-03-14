# Component Model Type System

This document explains the type system of the WebAssembly Component Model, including type definitions, value types, and type checking.

## Type Definitions

The syntax for defining core types extends the existing core type definition
syntax, adding a `module` type constructor:

```ebnf
core:rectype     ::= ... from the Core WebAssembly spec
core:typedef     ::= ... from the Core WebAssembly spec
core:subtype     ::= ... from the Core WebAssembly spec
core:comptype    ::= ... from the Core WebAssembly spec
                   | <core:moduletype>
core:moduletype  ::= (module <core:moduledecl>*)
core:moduledecl  ::= <core:importdecl>
                   | <core:type>
                   | <core:alias>
                   | <core:exportdecl>
core:alias       ::= (alias <core:aliastarget> (<core:sort> <id>?))
core:aliastarget ::= outer <u32> <u32>
core:importdecl  ::= (import <core:name> <core:name> <core:importdesc>)
core:exportdecl  ::= (export <core:name> <core:exportdesc>)
core:exportdesc  ::= strip-id(<core:importdesc>)

where strip-id(X) parses '(' sort Y ')' when X parses '(' sort <id>? Y ')'
```

Here, `core:comptype` (short for "composite type") as defined in the [GC]
proposal is extended with a `module` type constructor. The GC proposal also
adds recursion and explicit subtyping between core wasm types. Owing to
their different requirements and intended modes of usage, module types
support implicit subtyping and are not recursive. Thus, the existing core
validation rules would require the declared supertypes of module types to be
empty and disallow recursive use of module types.

In the MVP, validation will also reject `core:moduletype` defining or aliasing
other `core:moduletype`s, since, before module-linking, core modules cannot
themselves import or export other core modules.

The body of a module type contains an ordered list of "module declarators"
which describe, at a type level, the imports and exports of the module. In a
module-type context, import and export declarators can both reuse the existing
[`core:importdesc`] production defined in WebAssembly 1.0, with the only
difference being that, in the text format, `core:importdesc` can bind an
identifier for later reuse while `core:exportdesc` cannot.

With the Core WebAssembly [type-imports], module types will need the ability to
define the types of exports based on the types of imports. In preparation for
this, module types start with an empty type index space that is populated by
`type` declarators, so that, in the future, these `type` declarators can refer to
type imports local to the module type itself. For example, in the future, the
following module type would be expressible:

```wat
(component $C
  (core type $M (module
    (import "" "T" (type $T))
    (type $PairT (struct (field (ref $T)) (field (ref $T))))
    (export "make_pair" (func (param (ref $T)) (result (ref $PairT))))
  ))
)
```

In this example, `$M` has a distinct type index space from `$C`, where element
0 is the imported type, element 1 is the `struct` type, and element 2 is an
implicitly-created `func` type referring to both.

Lastly, the `core:alias` module declarator allows a module type definition to
reuse (rather than redefine) type definitions in the enclosing component's core
type index space via `outer` `type` alias. In the MVP, validation restricts
`core:alias` module declarators to _only_ allow `outer` `type` aliases (into an
enclosing component's or component-type's core type index space). In the
future, more kinds of aliases would be meaningful and allowed.

As an example, the following component defines two semantically-equivalent
module types, where the former defines the function type via `type` declarator
and the latter refers via `alias` declarator.

```wat
(component $C
  (core type $C1 (module
    (type (func (param i32) (result i32)))
    (import "a" "b" (func (type 0)))
    (export "c" (func (type 0)))
  ))
  (core type $F (func (param i32) (result i32)))
  (core type $C2 (module
    (alias outer $C $F (type))
    (import "a" "b" (func (type 0)))
    (export "c" (func (type 0)))
  ))
)
```

Component-level type definitions are symmetric to core-level type definitions,
but use a completely different set of value types. Unlike [`core:valtype`]
which is low-level and assumes a shared linear memory for communicating
compound values, component-level value types assume no shared memory and must
therefore be high-level, describing entire compound values.

```ebnf
type          ::= (type <id>? <deftype>)
deftype       ::= <defvaltype>
                | <resourcetype>
                | <functype>
                | <componenttype>
                | <instancetype>
defvaltype    ::= bool
                | s8 | u8 | s16 | u16 | s32 | u32 | s64 | u64
                | f32 | f64
                | char | string
                | error-context 🔀
                | (record (field "<label>" <valtype>)+)
                | (variant (case "<label>" <valtype>?)+)
                | (list <valtype>)
                | (list <valtype> <u32>) 🔧
                | (tuple <valtype>+)
                | (flags "<label>"+)
                | (enum "<label>"+)
                | (option <valtype>)
                | (result <valtype>? (error <valtype>)?)
                | (own <typeidx>)
                | (borrow <typeidx>)
                | (stream <typeidx>?) 🔀
                | (future <typeidx>?) 🔀
valtype       ::= <typeidx>
                | <defvaltype>
resourcetype  ::= (resource (rep i32) (dtor async? <funcidx> (callback <funcidx>)?)?)
functype      ::= (func (param "<label>" <valtype>)* (result <valtype>)?)
componenttype ::= (component <componentdecl>*)
instancetype  ::= (instance <instancedecl>*)
componentdecl ::= <importdecl>
                | <instancedecl>
instancedecl  ::= core-prefix(<core:type>)
                | <type>
                | <alias>
                | <exportdecl>
                | <value> 🪙
importdecl    ::= (import <importname> bind-id(<externdesc>))
exportdecl    ::= (export <exportname> bind-id(<externdesc>))
externdesc    ::= (<sort> (type <u32>) )
                | core-prefix(<core:moduletype>)
                | <functype>
                | <componenttype>
                | <instancetype>
                | (value <valuebound>) 🪙
                | (type <typebound>)
typebound     ::= (eq <typeidx>)
                | (sub resource)
valuebound    ::= (eq <valueidx>) 🪙
                | <valtype> 🪙

where bind-id(X) parses '(' sort <id>? Y ')' when X parses '(' sort Y ')'
```

Because there is nothing in this type grammar analogous to the [gc] proposal's
[`rectype`], none of these types are recursive.

## Fundamental Value Types

The value types in `valtype` can be broken into two categories: _fundamental_
value types and _specialized_ value types, where the latter are defined by
expansion into the former. The _fundamental value types_ have the following
sets of abstract values:
| Type | Values |
| ------------------------- | ------ |
| `bool` | `true` and `false` |
| `s8`, `s16`, `s32`, `s64` | integers in the range [-2<sup>N-1</sup>, 2<sup>N-1</sup>-1] |
| `u8`, `u16`, `u32`, `u64` | integers in the range [0, 2<sup>N</sup>-1] |
| `f32`, `f64` | [IEEE754] floating-point numbers, with a single NaN value |
| `char` | [Unicode Scalar Values] |
| `error-context` | an immutable, non-deterministic, host-defined value meant to aid in debugging |
| `record` | heterogeneous [tuples] of named values |
| `variant` | heterogeneous [tagged unions] of named values |
| `list` | homogeneous, variable- or fixed-length [sequences] of values |
| `own` | a unique, opaque address of a resource that will be destroyed when this value is dropped |
| `borrow` | an opaque address of a resource that must be dropped before the current export call returns |
| `stream` | an asynchronously-passed list of homogeneous values |
| `future` | an asynchronously-passed single value |

How these abstract values are produced and consumed from Core WebAssembly
values and linear memory is configured by the component via _canonical lifting
and lowering definitions_, which are introduced [below](#canonical-definitions).
For example, while abstract `variant`s contain a list of `case`s labelled by
name, canonical lifting and lowering map each case to an `i32` value starting
at `0`.

### Numeric Types

While core numeric types are defined in terms of sets of bit-patterns and
operations that interpret the bits in various ways, component-level numeric
types are defined in terms of sets of values. This allows the values to be
translated between source languages and protocols that use different
value representations.

Core integer types are just bit-patterns that don't distinguish between signed
and unsigned, while component-level integer types are sets of integers that
either include negative values or don't. Core floating-point types have many
distinct NaN bit-patterns, while component-level floating-point types have only
a single NaN value. And boolean values in core wasm are usually represented as
`i32`s where operations interpret all-zeros as `false`, while at the
component-level there is a `bool` type with `true` and `false` values.

### 🔀 Error Context Type

Values of `error-context` type are immutable, non-deterministic, host-defined
and meant to be propagated from failure sources to callers in order to aid in
debugging. Currently `error-context` values contain only a "debug message"
string whose contents are determined by the host. Core wasm can create
`error-context` values given a debug string, but the host is free to
arbitrarily transform (discard, preserve, prefix or suffix) this
wasm-provided string. In the future, `error-context` could be enhanced with
other additional or more-structured context (like a backtrace or a chain of
originating error contexts).

The intention of this highly-non-deterministic semantics is to provide hosts
the full range of flexibility to:

- append a basic callstack suitable for forensic debugging in production;
- optimize for performance in high-volume production scenarios by slicing or
  discarding debug messages;
- optimize for developer experience in debugging scenarios when debug metadata
  is present by appending expensive-to-produce symbolicated callstacks.

A consequence of this, however, is that components _must not_ depend on the
contents of `error-context` values for behavioral correctness. In particular,
case analysis of the contents of an `error-context` should not determine
_error recovery_; explicit `result` or `variant` types must be used in the
function return type instead (e.g.,
`(func (result (tuple (stream u8) (future $my-error)))`).

### Container Types

The `record`, `variant`, and `list` types allow for grouping, categorizing,
and sequencing contained values.

🔧 When the optional `<u32>` immediate of the `list` type constructor is present,
the list has a fixed length and the representation of the list in memory is
specialized to this length.

### Handle Types

The `own` and `borrow` value types are both _handle types_. Handles logically
contain the opaque address of a resource and avoid copying the resource when
passed across component boundaries. By way of metaphor to operating systems,
handles are analogous to file descriptors, which are stored in a table and may
only be used indirectly by untrusted user-mode processes via their integer
index in the table.

In the Component Model, handles are lifted-from and lowered-into `i32` values
that index an encapsulated per-component-instance _handle table_ that is
maintained by the canonical function definitions described
[below](#canonical-definitions). In the future, handles could be
backwards-compatibly lifted and lowered from [reference types] (via the
addition of a new `canonopt`, as introduced [below](#canonical-abi)).

The uniqueness and dropping conditions mentioned above are enforced at runtime
by the Component Model through these canonical definitions. The `typeidx`
immediate of a handle type must refer to a `resource` type (described below)
that statically classifies the particular kinds of resources the handle can
point to.

### Asynchronous Value Types

The `stream` and `future` value types are both _asynchronous value types_ that
are used to deliver values incrementally over the course of a single async
function call, instead of copying the values all-at-once as with other
(synchronous) value types like `list`. The mechanism for performing these
incremental copies avoids the need for intermediate buffering inside the
`stream` or `future` value itself and instead uses buffers of memory whose
size and allocation is controlled by the core wasm in the source and
destination components. Thus, in the abstract, `stream` and `future` can be
thought of as inter-component control-flow or synchronization mechanisms.

Just like with handles, in the Component Model, async value types are
lifted-from and lowered-into `i32` values that index an encapsulated
per-component-instance table that is maintained by the canonical ABI built-ins
[below](#canonical-definitions). The Component-Model-defined ABI for creating,
writing-to and reading-from `stream` and `future` values is meant to be bound
to analogous source-language features like promises, futures, streams,
iterators, generators and channels so that developers can use these familiar
high-level concepts when working directly with component types, without the
need to manually write low-level async glue code. For languages like C without
language-level concurrency support, these ABIs (described in detail in the
[Canonical ABI explainer]) can be exposed directly as function imports and used
like normal low-level Operation System I/O APIs.

A `stream<T>` asynchronously passes zero or more `T` values in one direction
between a source and destination, batched in chunks for efficiency. Streams
are useful for:

- improving latency by incrementally processing values as they arrive;
- delivering potentially-large lists of values that might OOM wasm if passed
  as a `list<T>`;
- long-running or infinite streams of events.

A `future` is a special case of `stream` and (in non-error scenarios) delivers
exactly one value before being automatically closed. Because all imports can
be [called asynchronously](Async.md), futures are not necessary to express a
traditional `async` function -- all functions are effectively `async`. Instead
futures are useful in more advanced scenarios where a parameter or result
value may not be ready at the same time as the other synchronous parameters or
results.

The `T` element type of `stream` and `future` is an optional `valtype`. As with
variant-case payloads and function results, when `T` is absent, the "value(s)"
being asynchronously passed can be thought of as [unit] values. In such cases,
there is no representation of the value in Core WebAssembly (pointers into
linear memory are ignored) however the _timing_ of completed reads and writes
and the number of elements they contain are observable and meaningful. Thus, empty futures and streams can be useful for
timing-related APIs.

Currently, validation rejects `(stream T)` and `(future T)` when `T`
transitively contains a `borrow`. This restriction could be relaxed in the
future by extending the call-scoping rules of `borrow` to streams and futures.

## Specialized Value Types

The sets of values allowed for the remaining _specialized value types_ are
defined by the following mapping:

```
                    (tuple <valtype>*) ↦ (record (field "𝒊" <valtype>)*) for 𝒊=0,1,...
                    (flags "<label>"*) ↦ (record (field "<label>" bool)*)
                     (enum "<label>"+) ↦ (variant (case "<label>")+)
                    (option <valtype>) ↦ (variant (case "none") (case "some" <valtype>))
(result <valtype>? (error <valtype>)?) ↦ (variant (case "ok" <valtype>?) (case "error" <valtype>?))
                                string ↦ (list char)
```

Specialized value types have the same set of semantic values as their
corresponding despecialized types, but have distinct type constructors
(which are not type-equal to the unspecialized type constructors) and
thus have distinct binary encodings. This allows specialized value types to
convey a more specific intent. For example, `result` isn't just a variant,
it's a variant that _means_ success or failure, so source-code bindings
can expose it via idiomatic source-language error reporting. Additionally,
this can sometimes allow values to be represented differently. For example,
`string` in the Canonical ABI uses various Unicode encodings while
`list<char>` uses a sequence of 4-byte `char` code points. Similarly,
`flags` in the Canonical ABI uses a bit-vector while an equivalent record
of boolean fields uses a sequence of boolean-valued bytes.

Note that, at least initially, variants are required to have a non-empty list of
cases. This could be relaxed in the future to allow an empty list of cases, with
the empty `(variant)` effectively serving as an [empty type] and indicating
unreachability.

## Definition Types

The remaining 4 type constructors in `deftype` use `valtype` to describe
shared-nothing functions, resources, components, and component instances:

The `func` type constructor describes a component-level function definition
that takes a list of `valtype` parameters with [strongly-unique] names and
optionally returns a `valtype`.

The `resource` type constructor creates a fresh type for each instance of the
containing component (with "freshness" and its interaction with general
type-checking described in more detail [below](#type-checking)). Resource types
can be referred to by handle types (such as `own` and `borrow`) as well as the
canonical built-ins described [below](#canonical-built-ins). The `rep`
immediate of a `resource` type specifies its _core representation type_, which
is currently fixed to `i32`, but will be relaxed in the future (to at least
include `i64`, but also potentially other types). When the last handle to a
resource is dropped, the resource's destructor function specified by the `dtor`
immediate will be called (if present), allowing the implementing component to
perform clean-up like freeing linear memory allocations. Destructors can be
declared `async`, with the same meaning for the `async` and `callback`
immediates as described below for `canon lift`.

The `instance` type constructor describes a list of named, typed definitions
that can be imported or exported by a component. Informally, instance types
correspond to the usual concept of an "interface" and instance types thus serve
as static interface descriptions. In addition to the S-Expression text format
defined here, which is meant to go inside component definitions, interfaces can
also be defined as standalone, human-friendly text files in the [`wit`](WIT.md)
[Interface Definition Language].

The `component` type constructor is symmetric to the core `module` type
constructor and contains _two_ lists of named definitions for the imports
and exports of a component, respectively. As suggested above, instance types
can show up in _both_ the import and export types of a component type.

## Declarators

The `importdecl` and `exportdecl` declarators correspond to component `import`
and `export` definitions, respectively, allowing an identifier to be bound for
use by subsequent declarators. The definitions of `label`, `importname` and
`exportname` are given in the [imports and exports](#import-and-export-definitions)
section below. Following the precedent of [`core:typeuse`], the text format
allows both references to out-of-line type definitions (via `(type <typeidx>)`)
and inline type expressions that the text format desugars into out-of-line type
definitions.

🪙 The `value` case of `externdesc` describes a runtime value that is imported or
exported at instantiation time as described in the
[value definitions](#value-definitions) section below.

The `type` case of `externdesc` describes an imported or exported type along
with its "bound":

The `sub` bound declares that the imported/exported type is an _abstract type_
which is a _subtype_ of some other type. Currently, the only supported bound is
`resource` which (following the naming conventions of the [GC] proposal) means
"any resource type". Thus, only resource types can be imported/exported
abstractly, not arbitrary value types. This allows type imports to always be
compiled independently of their arguments using a "universal representation" for
handle values (viz., `i32`, as defined by the [Canonical ABI](CanonicalABI.md)).
In the future, `sub` may be extended to allow referencing other resource types,
thereby allowing abstract resource subtyping.

The `eq` bound says that the imported/exported type must be structurally equal
to some preceding type definition. This allows:

- an imported abstract type to be re-exported;
- components to introduce another label for a preceding abstract type (which
  can be necessary when implementing multiple independent interfaces with the
  same resource); and
- components to attach transparent type aliases to structural types to be
  reflected in source-level bindings (e.g., `(export "bytes" (type (eq (list u64))))`
  could generate in C++ a `typedef std::vector<uint64_t> bytes` or in JS an
  exported field named `bytes` that aliases `Uint64Array`.

Relaxing the restrictions of `core:alias` declarators mentioned above, `alias`
declarators allow both `outer` and `export` aliases of `type` and `instance`
sorts. This allows the type exports of `instance`-typed import and export
declarators to be used by subsequent declarators in the type:

```wat
(component
  (import "fancy-fs" (instance $fancy-fs
    (export $fs "fs" (instance
      (export "file" (type (sub resource)))
      ;; ...
    ))
    (alias export $fs "file" (type $file))
    (export "fancy-op" (func (param "f" (borrow $file))))
  ))
)
```

The `type` declarator is restricted by validation to disallow `resource` type
definitions, thereby preventing "private" resource type definitions from
appearing in component types and avoiding the [avoidance problem]. Thus, the
only resource types possible in an `instancetype` or `componenttype` are
introduced by `importdecl` or `exportdecl`.

With what's defined so far, we can define component types using a mix of type
definitions:

```wat
(component $C
  (type $T (list (tuple string bool)))
  (type $U (option $T))
  (type $G (func (param "x" (list $T)) (result $U)))
  (type $D (component
    (alias outer $C $T (type $C_T))
    (type $L (list $C_T))
    (import "f" (func (param "x" $L) (result (list u8))))
    (import "g" (func (type $G)))
    (export "g2" (func (type $G)))
    (export "h" (func (result $U)))
    (import "T" (type $T (sub resource)))
    (import "i" (func (param "x" (list (own $T)))))
    (export "T2" (type $T' (eq $T)))
    (export "U" (type $U' (sub resource)))
    (export "j" (func (param "x" (borrow $T')) (result (own $U'))))
  ))
)
```

Note that the inline use of `$G` and `$U` are syntactic sugar for `outer`
aliases.

## Type Checking

Like core modules, components have an up-front validation phase in which the
definitions of a component are checked for basic consistency. Type checking
is a central part of validation and, e.g., occurs when validating that the
`with` arguments of an [`instantiate`](#instance-definitions) expression are
type-compatible with the `import`s of the component being instantiated.

To incrementally describe how type-checking works, we'll start by asking how
_type equality_ works for non-resource, non-handle, local type definitions and
build up from there.

Type equality for almost all types (except as described below) is purely
_structural_. In a structural setting, types are considered to be Abstract
Syntax Trees whose nodes are type constructors with types like `u8` and
`string` considered to be "nullary" type constructors that appear at leaves and
non-nullary type constructors like `list` and `record` appearing at parent
nodes. Then, type equality is defined to be AST equality. Importantly, these
type ASTs do _not_ contain any type indices or depend on index space layout;
these binary format details are consumed by decoding to produce the AST. For
example, in the following compound component:

```wat
(component $A
  (type $ListString1 (list string))
  (type $ListListString1 (list $ListString1))
  (type $ListListString2 (list $ListString1))
  (component $B
    (type $ListString2 (list string))
    (type $ListListString3 (list $ListString2))
    (type $ListString3 (alias outer $A $ListString1))
    (type $ListListString4 (list $ListString3))
    (type $ListListString5 (alias outer $A $ListListString1))
  )
)
```

all 5 variations of `$ListListStringX` are considered equal since, after
decoding, they all have the same AST.

Next, the type equality relation on ASTs is relaxed to a more flexible
[subtyping] relation. Currently, subtyping is only relaxed for `instance` and
`component` types, but may be relaxed for more type constructors in the future
to better support API Evolution (being careful to understand how subtyping
manifests itself in the wide variety of source languages so that
subtype-compatible updates don't inadvertently break source-level clients).

Component and instance subtyping allows a subtype to export more and import
less than is declared by the supertype, ignoring the exact order of imports and
exports and considering only names. For example, here, `$I1` is a subtype of
`$I2`:

```wat
(component
  (type $I1 (instance
    (export "foo" (func))
    (export "bar" (func))
    (export "baz" (func))
  ))
  (type $I2 (instance
    (export "bar" (func))
    (export "foo" (func))
  ))
)
```

and `$C1` is a subtype of `$C2`:

```wat
(component
  (type $C1 (component
    (import "a" (func))
    (export "x" (func))
    (export "y" (func))
  ))
  (type $C2 (component
    (import "a" (func))
    (import "b" (func))
    (export "x" (func))
  ))
)
```

When we next consider type imports and exports, there are two distinct
subcases of `typebound` to consider: `eq` and `sub`.

The `eq` bound adds a type equality rule (extending the built-in set of
subtyping rules mentioned above) saying that the imported type is structurally
equivalent to the type referenced in the bound. For example, in the component:

```wat
(component
  (type $L1 (list u8))
  (import "L2" (type $L2 (eq $L1)))
  (import "L3" (type $L2 (eq $L1)))
  (import "L4" (type $L2 (eq $L3)))
)
```

all four `$L*` types are equal (in subtyping terms, they are all subtypes of
each other).

In contrast, the `sub` bound introduces a new _abstract_ type which the rest of
the component must conservatively assume can be _any_ type that is a subtype of
the bound. What this means for type-checking is that each subtype-bound type
import/export introduces a _fresh_ abstract type that is unequal to every
preceding type definition. Currently (and likely in the MVP), the only
supported type bound is `resource` (which means "any resource type") and thus
the only abstract types are abstract _resource_ types. As an example, in the
following component:

```wat
(component
  (import "T1" (type $T1 (sub resource)))
  (import "T2" (type $T2 (sub resource)))
)
```

the types `$T1` and `$T2` are not equal.

Once a type is imported, it can be referred to by subsequent equality-bound
type imports, thereby adding more types that it is equal to. For example, in
the following component:

```wat
(component $C
  (import "T1" (type $T1 (sub resource)))
  (import "T2" (type $T2 (sub resource)))
  (import "T3" (type $T3 (eq $T2)))
  (type $ListT1 (list (own $T1)))
  (type $ListT2 (list (own $T2)))
  (type $ListT3 (list (own $T3)))
)
```

the types `$T2` and `$T3` are equal to each other but not to `$T1`. By the
above transitive structural equality rules, the types `$List2` and `$List3` are
equal to each other but not to `$List1`.

Handle types (`own` and `borrow`) are structural types (like `list`) but, since
they refer to resource types, transitively "inherit" the freshness of abstract
resource types. For example, in the following component:

```wat
(component
  (import "T" (type $T (sub resource)))
  (import "U" (type $U (sub resource)))
  (type $Own1 (own $T))
  (type $Own2 (own $T))
  (type $Own3 (own $U))
  (type $ListOwn1 (list $Own1))
  (type $ListOwn2 (list $Own2))
  (type $ListOwn3 (list $Own3))
  (type $Borrow1 (borrow $T))
  (type $Borrow2 (borrow $T))
  (type $Borrow3 (borrow $U))
  (type $ListBorrow1 (list $Borrow1))
  (type $ListBorrow2 (list $Borrow2))
  (type $ListBorrow3 (list $Borrow3))
)
```

the types `$Own1` and `$Own2` are equal to each other but not to `$Own3` or
any of the `$Borrow*`. Similarly, `$Borrow1` and `$Borrow2` are equal to
each other but not `$Borrow3`. Transitively, the types `$ListOwn1` and
`$ListOwn2` are equal to each other but not `$ListOwn3` or any of the
`$ListBorrow*`. These type-checking rules for type imports mirror the
_introduction_ rule of [universal types] (∀T).

The above examples all show abstract types in terms of _imports_, but the same
"freshness" condition applies when aliasing the _exports_ of another component
as well. For example, in this component:

```wat
(component
  (import "C" (component $C
    (export "T1" (type (sub resource)))
    (export "T2" (type $T2 (sub resource)))
    (export "T3" (type (eq $T2)))
  ))
  (instance $c (instantiate $C))
  (alias export $c "T1" (type $T1))
  (alias export $c "T2" (type $T2))
  (alias export $c "T3" (type $T3))
)
```

the types `$T2` and `$T3` are equal to each other but not to `$T1`. These
type-checking rules for aliases of type exports mirror the _elimination_ rule
of [existential types] (∃T).

Next, we consider resource type _definitions_ which are a _third_ source of
abstract types. Unlike the abstract types introduced by type imports and
exports, resource type definitions provide canonical built-ins for setting and
getting a resource's private representation value (that are introduced
[below](#canonical-built-ins)). These built-ins are necessarily scoped to the
component instance that generated the resource type, thereby hiding access to a
resource type's representation from the outside world. Because each component
instantiation generates fresh resource types distinct from all preceding
instances of the same component, resource types are ["generative"].

For example, in the following example component:

```wat
(component
  (type $R1 (resource (rep i32)))
  (type $R2 (resource (rep i32)))
  (func $f1 (result (own $R1)) (canon lift ...))
  (func $f2 (param (own $R2)) (canon lift ...))
)
```

the types `$R1` and `$R2` are unequal and thus the return type of `$f1`
is incompatible with the parameter type of `$f2`.

The generativity of resource type definitions matches the abstract typing rules
of type exports mentioned above, which force all clients of the component to
bind a fresh abstract type. For example, in the following component:

```wat
(component
  (component $C
    (type $r1 (export "r1") (resource (rep i32)))
    (type $r2 (export "r2") (resource (rep i32)))
  )
  (instance $c1 (instantiate $C))
  (instance $c2 (instantiate $C))
  (type $c1r1 (alias export $c1 "r1"))
  (type $c1r2 (alias export $c1 "r2"))
  (type $c2r1 (alias export $c2 "r1"))
  (type $c2r2 (alias export $c2 "r2"))
)
```

all four types aliases in the outer component are unequal, reflecting the fact
that each instance of `$C` generates two fresh resource types.

If a single resource type definition is exported more than once, the exports
after the first are equality-bound to the first export. For example, the
following component:

```wat
(component
  (type $r (resource (rep i32)))
  (export "r1" (type $r))
  (export "r2" (type $r))
)
```

is assigned the following `componenttype`:

```wat
(component
  (export "r1" (type $r1 (sub resource)))
  (export "r2" (type (eq $r1)))
)
```

Thus, from an external perspective, `r1` and `r2` are two labels for the same
type.

If a component wants to hide this fact and force clients to assume `r1` and
`r2` are distinct types (thereby allowing the implementation to actually use
separate types in the future without breaking clients), an explicit type can be
ascribed to the export that replaces the `eq` bound with a less-precise `sub`
bound (using syntax introduced [below](#import-and-export-definitions)).

```wat
(component
  (type $r (resource (rep i32)))
  (export "r1" (type $r))
  (export "r2" (type $r) (type (sub resource)))
)
```

This component is assigned the following `componenttype`:

```wat
(component
  (export "r1" (type (sub resource)))
  (export "r2" (type (sub resource)))
)
```

The assignment of this type to the above component mirrors the _introduction_
rule of [existential types] (∃T).

When supplying a resource type (imported _or_ defined) to a type import via
`instantiate`, type checking performs a substitution, replacing all uses of the
`import` in the instantiated component with the actual type supplied via
`with`. For example, the following component validates:

```wat
(component $P
  (import "C1" (component $C1
    (import "T" (type $T (sub resource)))
    (export "foo" (func (param (own $T))))
  ))
  (import "C2" (component $C2
    (import "T" (type $T (sub resource)))
    (import "foo" (func (param (own $T))))
  ))
  (type $R (resource (rep i32)))
  (instance $c1 (instantiate $C1 (with "T" (type $R))))
  (alias export $c1 "foo" (func $foo))
  (instance $c2 (instantiate $C2 (with "T" (type $R)) (with "foo" (func $foo))))
)
```

This depends critically on the `T` imports of `$C1` and `$C2` having been
replaced by `$R` when validating the instantiations of `$c1` and `$c2`. These
type-checking rules for instantiating type imports mirror the _elimination_
rule of [universal types] (∀T).

Importantly, this type substitution performed by the parent is not visible to
the child at validation- or run-time. In particular, there are no runtime
casts that can "see through" to the original type parameter, avoiding
the usual [type-exposure problems with dynamic casts][non-parametric parametricity].

In summary: all type constructors are _structural_ with the exception of
`resource`, which is _abstract_ and _generative_. Type imports and exports that
have a subtype bound also introduce abstract types and follow the standard
introduction and elimination rules of universal and existential types.

Lastly, since "nominal" is often taken to mean "the opposite of structural", a
valid question is whether any of the above is "nominal typing". Inside a
component, resource types act "nominally": each resource type definition
produces a new local "name" for a resource type that is distinct from all
preceding resource types. The interesting case is when resource type equality
is considered from _outside_ the component, particularly when a single
component is instantiated multiple times. In this case, a single resource type
definition that is exported with a single `exportname` will get a fresh type
with each component instance, with the abstract typing rules mentioned above
ensuring that each of the component's instance's resource types are kept
distinct. Thus, in a sense, the generativity of resource types _generalizes_
traditional name-based nominal typing, providing a finer granularity of
isolation than otherwise achievable with a shared global namespace.

## References

[`core:importdesc`]: https://webassembly.github.io/spec/core/text/modules.html#text-importdesc
[`core:valtype`]: https://webassembly.github.io/spec/core/text/types.html#value-types
[`core:typeuse`]: https://webassembly.github.io/spec/core/text/modules.html#type-uses
[`rectype`]: https://webassembly.github.io/gc/core/text/types.html#text-rectype
[GC]: https://github.com/WebAssembly/gc/blob/main/proposals/gc/MVP.md
[type-imports]: https://github.com/WebAssembly/proposal-type-imports/blob/master/proposals/type-imports/Overview.md
[IEEE754]: https://en.wikipedia.org/wiki/IEEE_754
[Unicode Scalar Values]: https://unicode.org/glossary/#unicode_scalar_value
[tuples]: https://en.wikipedia.org/wiki/Tuple
[tagged unions]: https://en.wikipedia.org/wiki/Tagged_union
[sequences]: https://en.wikipedia.org/wiki/Sequence
[reference types]: https://github.com/WebAssembly/reference-types/blob/master/proposals/reference-types/Overview.md
[unit]: https://en.wikipedia.org/wiki/Unit_type
[empty type]: https://en.wikipedia.org/w/index.php?title=Empty_type
[strongly-unique]: #name-uniqueness
[Interface Definition Language]: https://en.wikipedia.org/wiki/Interface_description_language
[subtyping]: https://en.wikipedia.org/wiki/Subtyping
[universal types]: https://en.wikipedia.org/wiki/System_F
[existential types]: https://en.wikipedia.org/wiki/System_F
["generative"]: https://www.researchgate.net/publication/2426300_A_Syntactic_Theory_of_Type_Generativity_and_Sharing
[avoidance problem]: https://counterexamples.org/avoidance.html
[non-parametric parametricity]: https://people.mpi-sws.org/~dreyer/papers/npp/main.pdf
[Canonical ABI explainer]: CanonicalABI.md
