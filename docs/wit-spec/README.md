# The `wit` format

The Wasm Interface Type (WIT) format is an [IDL] to provide tooling for the
[WebAssembly Component Model][components] in two primary ways:

- WIT is a developer-friendly format to describe the imports and exports to a
  component. It is easy to read and write and provides the foundational basis
  for producing components from guest languages as well as consuming components
  in host languages.

- WIT packages are the basis of sharing types and definitions in an ecosystem of
  components. Authors can import types from other WIT packages when generating a
  component, publish a WIT package representing a host embedding, or collaborate
  on a WIT definition of a shared set of APIs between platforms.

A WIT package is a collection of WIT [`interface`s][interfaces] and
[`world`s][worlds] defined in files in the same directory that all use the
file extension `wit`, for example `foo.wit`. Files are encoded as valid utf-8
bytes. Types can be imported between interfaces within a package using
unqualified names and additionally from other packages through namespace-
and-package-qualified names.

This document will go through the purpose of the syntactic constructs of a WIT
document, a pseudo-formal [grammar specification][lexical-structure], and
additionally a specification of the [package format][package-format] of a WIT
package suitable for distribution.

See [Gated Features] for an explanation of 🔧.

[IDL]: https://en.wikipedia.org/wiki/Interface_description_language
[components]: https://github.com/webassembly/component-model
[Gated Features]: feature-gates.md

## Table of Contents

- [Package Names](package-names.md)
- [WIT Interfaces](interfaces.md)
- [WIT Worlds](worlds.md)
- [WIT Packages and `use`](packages-and-use.md)
- [Filesystem Structure](filesystem-structure.md)
- [Lexical Structure](lexical-structure.md)
- [Feature Gates](feature-gates.md)
- [Type Definitions](type-definitions.md)
- [Built-in Types](built-in-types.md)
- [Handles](handles.md)
- [Name Resolution](name-resolution.md)
- [Package Format](package-format.md)

[interfaces]: interfaces.md
[worlds]: worlds.md
[lexical-structure]: lexical-structure.md
[package-format]: package-format.md
