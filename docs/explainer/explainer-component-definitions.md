# Component and Instance Definitions

This document explains component definitions, instance definitions, and alias definitions in the WebAssembly Component Model.

## Component Definitions

At the top-level, a `component` is a sequence of definitions of various kinds:

```ebnf
component  ::= (component <id>? <definition>*)
definition ::= core-prefix(<core:module>)
             | core-prefix(<core:instance>)
             | core-prefix(<core:type>)
             | <component>
             | <instance>
             | <alias>
             | <type>
             | <canon>
             | <start> 🪺
             | <import>
             | <export>
             | <value> 🪙

where core-prefix(X) parses '(' 'core' Y ')' when X parses '(' Y ')'
```

Components are like Core WebAssembly modules in that their contained
definitions are acyclic: definitions can only refer to preceding definitions
(in the AST, text format and binary format). However, unlike modules,
components can arbitrarily interleave different kinds of definitions.

The `core-prefix` meta-function transforms a grammatical rule for parsing a
Core WebAssembly definition into a grammatical rule for parsing the same
definition, but with a `core` token added right after the leftmost paren.
For example, `core:module` accepts `(module (func))` so
`core-prefix(<core:module>)` accepts `(core module (func))`. Note that the
inner `func` doesn't need a `core` prefix; the `core` token is used to mark the
_transition_ from parsing component definitions into core definitions.

The [`core:module`] production is unmodified by the Component Model and thus
components embed Core WebAssembly (text and binary format) modules as currently
standardized, allowing reuse of an unmodified Core WebAssembly implementation.
The next production, `core:instance`, is not currently included in Core
WebAssembly, but would be if Core WebAssembly adopted the [module-linking]
proposal. This new core definition is introduced below, alongside its
component-level counterpart. Finally, the existing [`core:type`] production is
extended below to add core module types as proposed for module-linking. Thus,
the overall idea is to represent core definitions (in the AST, binary and text
format) as-if they had already been added to Core WebAssembly so that, if they
eventually are, the implementation of decoding and validation can be shared in
a layered fashion.

The next kind of definition is, recursively, a component itself. Thus,
components form trees with all other kinds of definitions only appearing at the
leaves. For example, with what's defined so far, we can write the following
component:

```wat
(component
  (component
    (core module (func (export "one") (result i32) (i32.const 1)))
    (core module (func (export "two") (result f32) (f32.const 2)))
  )
  (core module (func (export "three") (result i64) (i64.const 3)))
  (component
    (component
      (core module (func (export "four") (result f64) (f64.const 4)))
    )
  )
  (component)
)
```

This top-level component roots a tree with 4 modules and 1 component as
leaves. However, in the absence of any `instance` definitions (introduced
next), nothing will be instantiated or executed at runtime; everything here is
dead code.

## Instance Definitions

Whereas modules and components represent immutable _code_, instances associate
code with potentially-mutable _state_ (e.g., linear memory) and thus are
necessary to create before being able to _run_ the code. Instance definitions
create module or component instances by selecting a module or component to
**instantiate** and then supplying a set of named _arguments_ which satisfy all
the named _imports_ of the selected module or component. This low-level
instantiation mechanism allows the Component Model to simultaneously support
multiple different styles of traditional [linking](Linking.md).

The syntax for defining a core module instance is:

```ebnf
core:instance       ::= (instance <id>? <core:instancexpr>)
core:instanceexpr   ::= (instantiate <core:moduleidx> <core:instantiatearg>*)
                      | <core:inlineexport>*
core:instantiatearg ::= (with <core:name> (instance <core:instanceidx>))
                      | (with <core:name> (instance <core:inlineexport>*))
core:sortidx        ::= (<core:sort> <u32>)
core:sort           ::= func
                      | table
                      | memory
                      | global
                      | type
                      | module
                      | instance
core:inlineexport   ::= (export <core:name> <core:sortidx>)
```

When instantiating a module via `instantiate`, the two-level imports of the
core modules are resolved as follows:

1. The first `core:name` of the import is looked up in the named list of
   `core:instantiatearg` to select a core module instance. (In the future,
   other `core:sort`s could be allowed if core wasm adds single-level
   imports.)
2. The second `core:name` of the import is looked up in the named list of
   exports of the core module instance found by the first step to select the
   imported core definition.

Each `core:sort` corresponds 1:1 with a distinct [index space] that contains
only core definitions of that _sort_. The `u32` field of `core:sortidx`
indexes into the sort's associated index space to select a definition.

Based on this, we can link two core modules `$A` and `$B` together with the
following component:

```wat
(component
  (core module $A
    (func (export "one") (result i32) (i32.const 1))
  )
  (core module $B
    (func (import "a" "one") (result i32))
  )
  (core instance $a (instantiate $A))
  (core instance $b (instantiate $B (with "a" (instance $a))))
)
```

To see examples of other sorts, we'll need `alias` definitions, which are
introduced in the next section.

The `<core:inlineexport>*` form of `core:instanceexpr` allows module instances
to be created by directly tupling together preceding definitions, without the
need to `instantiate` a helper module. The `<core:inlineexport>*` form of
`core:instantiatearg` is syntactic sugar that is expanded during text format
parsing into an out-of-line instance definition referenced by `with`. To show
an example of these, we'll also need the `alias` definitions introduced in the
next section.

The syntax for defining component instances is symmetric to core module
instances, but with an expanded component-level definition of `sort`:

```ebnf
instance       ::= (instance <id>? <instanceexpr>)
instanceexpr   ::= (instantiate <componentidx> <instantiatearg>*)
                 | <inlineexport>*
instantiatearg ::= (with <name> <sortidx>)
                 | (with <name> (instance <inlineexport>*))
name           ::= <core:name>
sortidx        ::= (<sort> <u32>)
sort           ::= core <core:sort>
                 | func
                 | value 🪙
                 | type
                 | component
                 | instance
inlineexport   ::= (export <exportname> <sortidx>)
```

Because component-level function, type and instance definitions are different
than core-level function, type and instance definitions, they are put into
disjoint index spaces which are indexed separately. Components may import
and export various core definitions (when they are compatible with the
[shared-nothing] model, which currently means only `module`, but may in the
future include `data`). Thus, component-level `sort` injects the full set
of `core:sort`, so that they may be referenced (leaving it up to validation
rules to throw out the core sorts that aren't allowed in various contexts).

The `name` production reuses the `core:name` quoted-string-literal syntax of
Core WebAssembly (which appears in core module imports and exports and can
contain any valid UTF-8 string).

🪙 The `value` sort refers to a value that is provided and consumed during
instantiation. How this works is described in the
[value definitions](#value-definitions) section.

To see a non-trivial example of component instantiation, we'll first need to
introduce a few other definitions below that allow components to import, define
and export component functions.

## Alias Definitions

Alias definitions project definitions out of other components' index spaces and
into the current component's index spaces. As represented in the AST below,
there are three kinds of "targets" for an alias: the `export` of a component
instance, the `core export` of a core module instance and a definition of an
`outer` component (containing the current component):

```ebnf
alias            ::= (alias <aliastarget> (<sort> <id>?))
aliastarget      ::= export <instanceidx> <name>
                   | core export <core:instanceidx> <core:name>
                   | outer <u32> <u32>
```

If present, the `id` of the alias is bound to the new index added by the alias
and can be used anywhere a normal `id` can be used.

In the case of `export` aliases, validation ensures `name` is an export in the
target instance and has a matching sort.

In the case of `outer` aliases, the `u32` pair serves as a [de Bruijn
index], with first `u32` being the number of enclosing components/modules to
skip and the second `u32` being an index into the target's sort's index space.
In particular, the first `u32` can be `0`, in which case the outer alias refers
to the current component. To maintain the acyclicity of module instantiation,
outer aliases are only allowed to refer to _preceding_ outer definitions.

Components containing outer aliases effectively produce a [closure] at
instantiation time, including a copy of the outer-aliased definitions. Because
of the prevalent assumption that components are immutable values, outer aliases
are restricted to only refer to immutable definitions: non-resource types,
modules and components. (In the future, outer aliases to all sorts of
definitions could be allowed by recording the statefulness of the resulting
component in its type via some kind of "`stateful`" type attribute.)

Both kinds of aliases come with syntactic sugar for implicitly declaring them
inline:

For `export` aliases, the inline sugar extends the definition of `sortidx`
and the various sort-specific indices:

```ebnf
sortidx     ::= (<sort> <u32>)          ;; as above
              | <inlinealias>
Xidx        ::= <u32>                   ;; as above
              | <inlinealias>
inlinealias ::= (<sort> <u32> <name>+)
```

If `<sort>` refers to a `<core:sort>`, then the `<u32>` of `inlinealias` is a
`<core:instanceidx>`; otherwise it's an `<instanceidx>`. For example, the
following snippet uses two inline function aliases:

```wat
(instance $j (instantiate $J (with "f" (func $i "f"))))
(export "x" (func $j "g" "h"))
```

which are desugared into:

```wat
(alias export $i "f" (func $f_alias))
(instance $j (instantiate $J (with "f" (func $f_alias))))
(alias export $j "g" (instance $g_alias))
(alias export $g_alias "h" (func $h_alias))
(export "x" (func $h_alias))
```

For `outer` aliases, the inline sugar is simply the identifier of the outer
definition, resolved using normal lexical scoping rules. For example, the
following component:

```wat
(component
  (component $C ...)
  (component
    (instance (instantiate $C))
  )
)
```

is desugared into:

```wat
(component $Parent
  (component $C ...)
  (component
    (alias outer $Parent $C (component $Parent_C))
    (instance (instantiate $Parent_C))
  )
)
```

Lastly, for symmetry with [imports][func-import-abbrev], aliases can be written
in an inverted form that puts the sort first:

```wat
    (func $f (import "i" "f") ...type...) ≡ (import "i" "f" (func $f ...type...))   (WebAssembly 1.0)
          (func $f (alias export $i "f")) ≡ (alias export $i "f" (func $f))
   (core module $m (alias export $i "m")) ≡ (alias export $i "m" (core module $m))
(core func $f (alias core export $i "f")) ≡ (alias core export $i "f" (core func $f))
```

With what's defined so far, we're able to link modules with arbitrary renamings:

```wat
(component
  (core module $A
    (func (export "one") (result i32) (i32.const 1))
    (func (export "two") (result i32) (i32.const 2))
    (func (export "three") (result i32) (i32.const 3))
  )
  (core module $B
    (func (import "a" "one") (result i32))
  )
  (core instance $a (instantiate $A))
  (core instance $b1 (instantiate $B
    (with "a" (instance $a))                      ;; no renaming
  ))
  (core func $a_two (alias core export $a "two")) ;; ≡ (alias core export $a "two" (core func $a_two))
  (core instance $b2 (instantiate $B
    (with "a" (instance
      (export "one" (func $a_two))                ;; renaming, using out-of-line alias
    ))
  ))
  (core instance $b3 (instantiate $B
    (with "a" (instance
      (export "one" (func $a "three"))            ;; renaming, using <inlinealias>
    ))
  ))
)
```

To show analogous examples of linking components, we'll need component-level
type and function definitions which are introduced in the next two sections.

## References

[`core:module`]: https://webassembly.github.io/spec/core/text/modules.html#text-module
[`core:type`]: https://webassembly.github.io/spec/core/text/modules.html#types
[module-linking]: https://github.com/WebAssembly/module-linking/blob/main/proposals/module-linking/Explainer.md
[index space]: https://webassembly.github.io/spec/core/syntax/modules.html#indices
[de Bruijn index]: https://en.wikipedia.org/wiki/De_Bruijn_index
[closure]: https://en.wikipedia.org/wiki/Closure_(computer_programming)
[func-import-abbrev]: https://webassembly.github.io/spec/core/text/modules.html#text-func-abbrev
[shared-nothing]: ../high-level/Choices.md
