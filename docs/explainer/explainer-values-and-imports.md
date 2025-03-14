# Value Definitions, Start Definitions, and Import/Export Definitions

This document explains value definitions, start definitions, and import/export definitions in the WebAssembly Component Model.

### 🪙 Value Definitions

Value definitions (in the value index space) are like immutable `global` definitions
in Core WebAssembly except that validation requires them to be consumed exactly
once at instantiation-time (i.e., they are [linear]).

Components may define values in the value index space using following syntax:

```ebnf
value    ::= (value <id>? <valtype> <val>)
val      ::= false | true
           | <core:i64>
           | <f64canon>
           | nan
           | '<core:stringchar>'
           | <core:name>
           | (record <val>+)
           | (variant "<label>" <val>?)
           | (list <val>*)
           | (tuple <val>+)
           | (flags "<label>"*)
           | (enum "<label>")
           | none | (some <val>)
           | ok | (ok <val>) | error | (error <val>)
           | (binary <core:datastring>)
f64canon ::= <core:f64> without the `nan:0x` case.
```

The validation rules for `value` require the `val` to match the `valtype`.

The `(binary ...)` expression form provides an alternative syntax allowing the binary contents
of the value definition to be written directly in the text format, analogous to data segments,
avoiding the need to understand type information when encoding or decoding.

For example:
```wat
(component
  (value $a bool true)
  (value $b u8  1)
  (value $c u16 2)
  (value $d u32 3)
  (value $e u64 4)
  (value $f s8  5)
  (value $g s16 6)
  (value $h s32 7)
  (value $i s64 8)
  (value $j f32 9.1)
  (value $k f64 9.2)
  (value $l char 'a')
  (value $m string "hello")
  (value $n (record (field "a" bool) (field "b" u8)) (record true 1))
  (value $o (variant (case "a" bool) (case "b" u8)) (variant "b" 1))
  (value $p (list (result (option u8)))
    (list
      error
      (ok (some 1))
      (ok none)
      error
      (ok (some 2))
    )
  )
  (value $q (tuple u8 u16 u32) (tuple 1 2 3))

  (type $abc (flags "a" "b" "c"))
  (value $r $abc (flags "a" "c"))

  (value $s (enum "a" "b" "c") (enum "b"))

  (value $t bool (binary "\00"))
  (value $u string (binary "\07example"))

  (type $complex
    (tuple
      (record
        (field "a" (option string))
        (field "b" (tuple (option u8) string))
      )
      (list char)
      $abc
      string
    )
  )
  (value $complex1 (type $complex)
    (tuple
      (record
        none
        (tuple none "empty")
      )
      (list)
      (flags)
      ""
    )
  )
  (value $complex2 (type $complex)
    (tuple
      (record
        (some "example")
        (tuple (some 42) "hello")
      )
      (list 'a' 'b' 'c')
      (flags "b" "a")
      "hi"
    )
  )
)
```

As with all definition sorts, values may be imported and exported by
components. As an example value import:
```wat
(import "env" (value $env (record (field "locale" (option string)))))
```
As this example suggests, value imports can serve as generalized [environment
variables], allowing not just `string`, but the full range of `valtype`.

Values can also be exported.  For example:
```wat
(component
  (import "system-port" (value $port u16))
  (value $url string "https://example.com")
  (export "default-url" (value $url))
  (export "default-port" (value $port))
)
```
The inferred type of this component is:
```wat
(component
  (import "system-port" (value $port u16))
  (value $url string "https://example.com")
  (export "default-url" (value (eq $url)))
  (export "default-port" (value (eq $port)))
)
```
Thus, by default, the precise constant or import being exported is propagated
into the component's type and thus its public interface.  In this way, value exports
can act as semantic configuration data provided by the component to the host
or other client tooling.
Components can also keep the exact value being exported abstract (so that the
precise value is not part of the type and public interface) using the "type ascription"
feature mentioned in the [imports and exports](#import-and-export-definitions) section below.

### 🪙 Start Definitions

Like modules, components can have start functions that are called during
instantiation. Unlike modules, components can call start functions at multiple
points during instantiation with each such call having parameters and results.
Thus, `start` definitions in components look like function calls:
```ebnf
start ::= (start <funcidx> (value <valueidx>)* (result (value <id>?))*)
```
The `(value <valueidx>)*` list specifies the arguments passed to `funcidx` by
indexing into the *value index space*. The arity and types of the two value lists are
validated to match the signature of `funcidx`.

With this, we can define a component that imports a string and computes a new
exported string at instantiation time:
```wat
(component
  (import "name" (value $name string))
  (import "libc" (core module $Libc
    (export "memory" (memory 1))
    (export "realloc" (func (param i32 i32 i32 i32) (result i32)))
  ))
  (core instance $libc (instantiate $Libc))
  (core module $Main
    (import "libc" ...)
    (func (export "start") (param i32 i32) (result i32)
      ... general-purpose compute
    )
  )
  (core instance $main (instantiate $Main (with "libc" (instance $libc))))
  (func $start (param string) (result string) (canon lift
    (core func $main "start")
    (memory (core memory $libc "mem")) (realloc (func $libc "realloc"))
  ))
  (start $start (value $name) (result (value $greeting)))
  (export "greeting" (value $greeting))
)
```
As this example shows, start functions reuse the same Canonical ABI machinery
as normal imports and exports for getting component-level values into and out
of core linear memory.


### Import and Export Definitions

Both import and export definitions append a new element to the index space of
the imported/exported `sort` which can be optionally bound to an identifier in
the text format. In the case of imports, the identifier is bound just like Core
WebAssembly, as part of the `externdesc` (e.g., `(import "x" (func $x))` binds
the identifier `$x`). In the case of exports, the `<id>?` right after the
`export` is bound while the `<id>` inside the `<sortidx>` is a reference to the
preceding definition being exported (e.g., `(export $x "x" (func $f))` binds a
new identifier `$x`).
```ebnf
import ::= (import "<importname>" bind-id(<externdesc>))
export ::= (export <id>? "<exportname>" <sortidx> <externdesc>?)
```
All import names are required to be [strongly-unique]. Separately, all export
names are also required to be [strongly-unique]. The rest of the grammar for
imports and exports defines a structured syntax for the contents of import and
export names. Syntactically, these names appear inside quoted string literals.
The grammar thus restricts the contents of these string literals to provide
more structured information that can be mechanically interpreted by toolchains
and runtimes to support idiomatic developer workflows and source-language
bindings. The rules defining this structured name syntax below are to be
interpreted as a *lexical* grammar defining a single token and thus whitespace
is not automatically inserted, all terminals are single-quoted, and everything
unquoted is a meta-character.
```ebnf
exportname    ::= <plainname>
                | <interfacename>
importname    ::= <exportname>
                | <depname>
                | <urlname>
                | <hashname>
plainname     ::= <label>
                | '[async]' <label> 🔀
                | '[constructor]' <label>
                | '[method]' <label> '.' <label>
                | '[async method]' <label> '.' <label> 🔀
                | '[static]' <label> '.' <label>
                | '[async static]' <label> '.' <label> 🔀
label         ::= <fragment>
                | <label> '-' <fragment>
fragment      ::= <word>
                | <acronym>
word          ::= [a-z] [0-9a-z]*
acronym       ::= [A-Z] [0-9A-Z]*
interfacename ::= <namespace> <label> <projection> <version>?
                | <namespace>+ <label> <projection>+ <version>? 🪺
namespace     ::= <words> ':'
words         ::= <word>
                | <words> '-' <word>
projection    ::= '/' <label>
version       ::= '@' <valid semver>
depname       ::= 'unlocked-dep=<' <pkgnamequery> '>'
                | 'locked-dep=<' <pkgname> '>' ( ',' <hashname> )?
pkgnamequery  ::= <pkgpath> <verrange>?
pkgname       ::= <pkgpath> <version>?
pkgpath       ::= <namespace> <words>
                | <namespace>+ <words> <projection>* 🪺
verrange      ::= '@*'
                | '@{' <verlower> '}'
                | '@{' <verupper> '}'
                | '@{' <verlower> ' ' <verupper> '}'
verlower      ::= '>=' <valid semver>
verupper      ::= '<' <valid semver>
urlname       ::= 'url=<' <nonbrackets> '>' (',' <hashname>)?
nonbrackets   ::= [^<>]*
hashname      ::= 'integrity=<' <integrity-metadata> '>'
```
Components provide six options for naming imports:
* a **plain name** that leaves it up to the developer to "read the docs"
  or otherwise figure out what to supply for the import;
* an **interface name** that is assumed to uniquely identify a higher-level
  semantic contract that the component is requesting an *unspecified* wasm
  or native implementation of;
* a **URL name** that the component is requesting be resolved to a *particular*
  wasm implementation by [fetching] the URL.
* a **hash name** containing a content-hash of the bytes of a *particular*
  wasm implementation but not specifying location of the bytes.
* a **locked dependency name** that the component is requesting be resolved via
  some contextually-supplied registry to a *particular* wasm implementation
  using the given hierarchical name and version; and
* an **unlocked dependency name** that the component is requesting be resolved
  via some contextually-supplied registry to *one of a set of possible* of wasm
  implementations using the given hierarchical name and version range.

Not all hosts are expected to support all six import naming options and, in
general, build tools may need to wrap a to-be-deployed component with an outer
component that only uses import names that are understood by the target host.
For example:
* an offline host may only implement a fixed set of interface names, requiring
  a build tool to **bundle** URL, dependency and hash names (replacing the
  imports with nested definitions);
* browsers may only support plain and URL names (with plain names resolved via
  import map or [JS API]), requiring the build process to publish or bundle
  dependencies, converting dependency names into nested definitions or URL
  names;
* a production server environment may only allow deployment of components
  importing from a fixed set of interface and locked dependency names, thereby
  requiring all dependencies to be locked and deployed beforehand;
* host embeddings without a direct developer interface (such as the JS API or
  import maps) may reject all plain names, requiring the build process to
  resolve these beforehand;
* hosts without content-addressable storage may reject hash names (as they have
  no way to locate the contents).

The grammar and validation of URL names allows the embedded URLs to contain any
sequence of UTF-8 characters (other than angle brackets, which are used to
[delimit the URL]), leaving the well-formedness of the URL to be checked as
part of the process of [parsing] the URL in preparation for [fetching] the URL.
The [base URL] operand passed to the URL spec's parsing algorithm is determined
by the host and may be absent, thereby disallowing relative URLs. Thus, the
parsing and fetching of a URL import are host-defined operations that happen
after the decoding and validation of a component, but before instantiation of
that component.

When a particular implementation is indicated via URL or dependency name,
`importname` allows the component to additionally specify a cryptographic hash
of the expected binary representation of the wasm implementation, reusing the
[`integrity-metadata`] production defined by the W3C Subresource Integrity
specification. When this hash is present, a component can express its intention
to reuse another component or core module with the same degree of specificity
as if the component or core module was nested directly, thereby allowing
components to factor out common dependencies without compromising runtime
behavior. When *only* the hash is present (in a `hashname`), the host must
locate the contents using the hash (e.g., using an [OCI Registry]).

The "registry" referred to by dependency names serves to map a hierarchical
name and version to a particular module, component or exported definition. For
example, in the full generality of nested namespaces and packages (🪺), in a
registry name `a:b:c/d/e/f`, `a:b:c` traverses a path through namespaces `a`
and `b` to a component `c` and `/d/e/f` traverses the exports of `c` (where `d`
and `e` must be component exports but `f` can be anything). Given this abstract
definition, a number of concrete data sources can be interpreted by developer
tooling as "registries":
* a live registry (perhaps accessed via [`warg`])
* a local filesystem directory (perhaps containing vendored dependencies)
* a fixed set of host-provided functionality (see also the [built-in modules] proposal)
* a programmatically-created tree data structure (such as the `importObject`
  parameter of [`WebAssembly.instantiate()`])

The `valid semver` production is as defined by the [Semantic Versioning 2.0]
spec and is meant to be interpreted according to that specification. The
`verrange` production embeds a minimal subset of the syntax for version ranges
found in common package managers like `npm` and `cargo` and is meant to be
interpreted with the same [semantics][SemVerRange]. (Mostly this
interpretation is the usual SemVer-spec-defined ordering, but note the
particular behavior of pre-release tags.)

The `plainname` production captures several language-neutral syntactic hints
that allow bindings generators to produce more idiomatic bindings in their
target language. At the top-level, a `plainname` allows functions to be
annotated as being a constructor, method or static function of a preceding
resource and/or being asynchronous.

When a function is annotated with `constructor`, `method` or `static`, the
first `label` is the name of the resource and the second `label` is the logical
field name of the function. This additional nesting information allows bindings
generators to insert the function into the nested scope of a class, abstract
data type, object, namespace, package, module or whatever resources get bound
to. For example, a function named `[method]C.foo` could be bound in C++ to a
member function `foo` in a class `C`. The JS API [below](#JS-API) describes how
the native JavaScript bindings could look. Validation described in
[Binary.md](Binary.md) inspects the contents of `plainname` and ensures that
the function has a compatible signature.

When a function is annotated with `async`, bindings generators are expected to
emit whatever asynchronous language construct is appropriate (such as an
`async` function in JS, Python or Rust). Note the absence of
`[async constructor]`. See the [async
explainer](Async.md#sync-and-async-functions) for more details.

The `label` production used inside `plainname` as well as the labels of
`record` and `variant` types are required to have [kebab case]. The reason for
this particular form of casing is to unambiguously separate words and acronyms
(represented as all-caps words) so that source language bindings can convert a
`label` into the idiomatic casing of that language. (Indeed, because hyphens
are often invalid in identifiers, kebab case practically forces language
bindings to make such a conversion.) For example, the `label` `is-XML` could be
mapped to `isXML`, `IsXml`, `is_XML` or `is_xml`, depending on the target
language/convention. The highly-restricted character set ensures that
capitalization is trivial and does not require consulting Unicode tables.

Components provide two options for naming exports, symmetric to the first two
options for naming imports:
* a **plain name** that leaves it up to the developer to "read the docs"
  or otherwise figure out what the export does and how to use it; and
* an **interface name** that is assumed to uniquely identify a higher-level
  semantic contract that the component is claiming to implement with the
  given exported definition.

As an example, the following component uses all 9 cases of imports and exports:
```wat
(component
  (import "custom-hook" (func (param string) (result string)))
  (import "wasi:http/handler" (instance
    (export "request" (type $request (sub resource)))
    (export "response" (type $response (sub resource)))
    (export "handle" (func (param (own $request)) (result (own $response))))
  ))
  (import "url=<https://mycdn.com/my-component.wasm>" (component ...))
  (import "url=<./other-component.wasm>,integrity=<sha256-X9ArH3k...>" (component ...))
  (import "locked-dep=<my-registry:sqlite@1.2.3>,integrity=<sha256-H8BRh8j...>" (component ...))
  (import "unlocked-dep=<my-registry:imagemagick@{>=1.0.0}>" (instance ...))
  (import "integrity=<sha256-Y3BsI4l...>" (component ...))
  ... impl
  (export "wasi:http/handler" (instance $http_handler_impl))
  (export "get-JSON" (func $get_json_impl))
)
```
Here, `custom-hook` and `get-JSON` are plain names for functions whose semantic
contract is particular to this component and not defined elsewhere. In
contrast, `wasi:http/handler` is the name of a separately-defined interface,
allowing the component to request the ability to make outgoing HTTP requests
(through imports) and receive incoming HTTP requests (through exports) in a way
that can be mechanically interpreted by hosts and tooling.

The remaining 4 imports show the different ways that a component can import
external implementations. Here, the URL and locked dependency imports use
`component` types, allowing this component to privately create and wire up
instances using `instance` definitions. In contrast, the unlocked dependency
import uses an `instance` type, anticipating a subsequent tooling step (likely
the one that performs dependency resolution) to select, instantiate and provide
the instance.

Validation of `export` requires that all transitive uses of resource types in
the types of exported functions or values refer to resources that were either
imported or exported (concretely, via the type index introduced by an `import`
or `export`). The optional `<externdesc>?` in `export` can be used to
explicitly ascribe a type to an export which is validated to be a supertype of
the definition's type, thereby allowing a private (non-exported) type
definition to be replaced with a public (exported) type definition.

For example, in the following component:
```wat
(component
  (import "R1" (type $R1 (sub resource)))
  (type $R2 (resource (rep i32)))
  (export $R2' "R2" (type $R2))
  (func $f1 (result (own $R1)) (canon lift ...))
  (func $f2 (result (own $R2)) (canon lift ...))
  (func $f2' (result (own $R2')) (canon lift ...))
  (export "f1" (func $f1))
  ;; (export "f2" (func $f2)) -- invalid
  (export "f2" (func $f2) (func (result (own $R2'))))
  (export "f2" (func $f2'))
)
```
the commented-out `export` is invalid because its type transitively refers to
`$R2`, which is a private type definition. This requirement is meant to address
the standard [avoidance problem] that appears in module systems with abstract
types. In particular, it ensures that a client of a component is able to
externally define a type compatible with the exports of the component.

Similar to type exports, value exports may also ascribe a type to keep the precise
value from becoming part of the type and public interface.

For example:
```wat
(component
  (value $url string "https://example.com")
  (export "default-url" (value $url) (value string))
)
```

The inferred type of this component is:
```wat
(component
  (export "default-url" (value string))
)
```

Note, that the `url` value definition is absent from the component type

### Name Uniqueness

The goal of the `label`, `exportname` and `importname` productions defined and
used above is to allow automated bindings generators to map these names into
something more idiomatic to the language. For example, the `plainname`
`[method]my-resource.my-method` might get mapped to a method named `myMethod`
nested inside a class `MyResource`. To unburden bindings generators from having
to consider pathological cases where two unique-in-the-component names get
mapped to the same source-language identifier, Component Model validation
imposes a stronger form of uniquness than simple string equality on all the
names that appear within the same scope.

To determine whether two names (defined as sequences of [Unicode Scalar
Values]) are **strongly-unique**:
* If one name is `l` and the other name is `[constructor]l` (for the same
  `label` `l`), they are strongly-unique.
* Otherwise:
  * Lowercase all the `acronym`s (uppercase letters) in both names.
  * Strip any `[...]` annotation prefix from both names.
  * The names are strongly-unique if the resulting strings are unequal.

Thus, the following names are strongly-unique:
* `foo`, `foo-bar`, `[constructor]foo`, `[method]foo.bar`, `[method]foo.baz`

but attempting to add *any* of the following names would be a validation error:
* `foo`, `foo-BAR`, `[constructor]foo-BAR`, `[async]foo`, `[method]foo.BAR`

Note that additional validation rules involving types apply to names with
annotations. For example, the validation rules for `[constructor]foo` require
`foo` to be a resource type. See [Binary.md](Binary.md#import-and-export-definitions)
for details.



## References

[Kebab Case]: https://en.wikipedia.org/wiki/Letter_case#Kebab_case
[Linear]: https://en.wikipedia.org/wiki/Substructural_type_system#Linear_type_systems
[Environment Variables]: https://en.wikipedia.org/wiki/Environment_variable
[Unicode Scalar Values]: https://unicode.org/glossary/#unicode_scalar_value
[Semantic Versioning 2.0]: https://semver.org/spec/v2.0.0.html
[SemVerRange]: https://semver.npmjs.com/
[Fetching]: https://fetch.spec.whatwg.org/
[Parsing]: https://url.spec.whatwg.org/#url-parsing
[Base URL]: https://url.spec.whatwg.org/  echept-base-url
