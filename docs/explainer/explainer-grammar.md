# Component Model Grammar

This document defines the grammar of the WebAssembly Component Model, providing the foundation for understanding component syntax and structure.

## Grammar

This section defines components using an EBNF grammar that parses something in
between a pure Abstract Syntax Tree (like the Core WebAssembly spec's
[Structure Section]) and a complete text format (like the Core WebAssembly
spec's [Text Format Section]). The goal is to balance completeness with
succinctness, with just enough detail to write examples and define a [binary
format](Binary.md) in the style of the [Binary Format Section], deferring full
precision to the [formal specification](../../spec/).

The main way the grammar hand-waves is regarding definition uses, where indices
referring to `X` definitions (written `<Xidx>`) should, in the real text
format, explicitly allow identifiers (`<id>`), checking at parse time that the
identifier resolves to an `X` definition and then embedding the resolved index
into the AST.

Additionally, standard [abbreviations] defined by the Core WebAssembly text
format (e.g., inline export definitions) are assumed but not explicitly defined
below.

## Index Spaces

[Like Core WebAssembly][Core Indices], the Component Model places each
`definition` into one of a fixed set of _index spaces_, allowing the
definition to be referred to by subsequent definitions (in the text and binary
format) via a nonnegative integral _index_. When defining, validating and
executing a component, there are 5 component-level index spaces:

- (component) functions
- (component) values
- (component) types
- component instances
- components

5 core index spaces that also exist in WebAssembly 1.0:

- (core) functions
- (core) tables
- (core) memories
- (core) globals
- (core) types

and 2 additional core index spaces that contain core definition introduced by
the Component Model that are not in WebAssembly 1.0 (yet: the [module-linking]
proposal would add them):

- module instances
- modules

for a total of 12 index spaces that need to be maintained by an implementation
when, e.g., validating a component. These 12 index spaces correspond 1:1 with
the terminals of the `sort` production defined below and thus "sort" and
"index space" can be used interchangeably.

Also [like Core WebAssembly][Core Identifiers], the Component Model text format
allows _identifiers_ to be used in place of these indices, which are resolved
when parsing into indices in the AST (upon which validation and execution is
defined). Thus, the following two components are equivalent:

```wat
(component
  (core module (; empty ;))
  (component   (; empty ;))
  (core module (; empty ;))
  (export "C" (component 0))
  (export "M1" (core module 0))
  (export "M2" (core module 1))
)
```

```wat
(component
  (core module $M1 (; empty ;))
  (component $C    (; empty ;))
  (core module $M2 (; empty ;))
  (export "C" (component $C))
  (export "M1" (core module $M1))
  (export "M2" (core module $M2))
)
```

## References

[Structure Section]: https://webassembly.github.io/spec/core/syntax/index.html
[Text Format Section]: https://webassembly.github.io/spec/core/text/index.html
[Binary Format Section]: https://webassembly.github.io/spec/core/binary/index.html
[Core Indices]: https://webassembly.github.io/spec/core/syntax/modules.html#indices
[Core Identifiers]: https://webassembly.github.io/spec/core/text/values.html#text-id
[abbreviations]: https://webassembly.github.io/spec/core/text/conventions.html#abbreviations
[module-linking]: https://github.com/WebAssembly/module-linking/blob/main/proposals/module-linking/Explainer.md
