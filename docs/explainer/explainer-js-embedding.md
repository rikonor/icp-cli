# JavaScript Embedding

This document explains the JavaScript embedding of the WebAssembly Component Model, including the JS API and ESM integration.

## JavaScript Embedding

### JS API

The [JS API] currently provides `WebAssembly.compile(Streaming)` which take
raw bytes from an `ArrayBuffer` or `Response` object and produces
`WebAssembly.Module` objects that represent decoded and validated modules. To
natively support the Component Model, the JS API would be extended to allow
these same JS API functions to accept component binaries and produce new
`WebAssembly.Component` objects that represent decoded and validated
components. The [binary format of components](Binary.md) is designed to allow
modules and components to be distinguished by the first 8 bytes of the binary
(splitting the 32-bit [`core:version`] field into a 16-bit `version` field and
a 16-bit `layer` field with `0` for modules and `1` for components).

Once compiled, a `WebAssembly.Component` could be instantiated using the
existing JS API `WebAssembly.instantiate(Streaming)`. Since components have the
same basic import/export structure as modules, this means extending the [*read
the imports*] logic to support single-level imports as well as imports of
modules, components and instances. Since the results of instantiating a
component is a record of JavaScript values, just like an instantiated module,
`WebAssembly.instantiate` would always produce a `WebAssembly.Instance` object
for both module and component arguments.

Types are a new sort of definition that are not ([yet][type-imports]) present
in Core WebAssembly and so the [*read the imports*] and [*create an exports
object*] steps need to be expanded to cover them:

For type exports, each type definition would export a JS constructor function.
This function would be callable iff a `[constructor]`-annotated function was
also exported. All `[method]`- and `[static]`-annotated functions would be
dynamically installed on the constructor's prototype chain. In the case of
re-exports and multiple exports of the same definition, the same constructor
function object would be exported (following the same rules as WebAssembly
Exported Functions today). In pathological cases (which, importantly, don't
concern the global namespace, but involve the same actual type definition being
imported and re-exported by multiple components), there can be collisions when
installing constructors, methods and statics on the same constructor function
object. In such cases, a conservative option is to undo the initial
installation and require all clients to instead use the full explicit names
as normal instance exports.

For type imports, the constructors created by type exports would naturally
be importable. Additionally, certain JS- and Web-defined objects that correspond
to types (e.g., the `RegExp` and `ArrayBuffer` constructors or any Web IDL
[interface object]) could be imported. The `ToWebAssemblyValue` checks on
handle values mentioned below can then be defined to perform the associated
[internal slot] type test, thereby providing static type guarantees for
outgoing handles that can avoid runtime dynamic type tests.

Lastly, when given a component binary, the compile-then-instantiate overloads
of `WebAssembly.instantiate(Streaming)` would inherit the compound behavior of
the abovementioned functions (again, using the `layer` field to eagerly
distinguish between modules and components).

For example, the following component:

```wat
;; a.wasm
(component
  (import "one" (func))
  (import "two" (value string)) 🪙
  (import "three" (instance
    (export "four" (instance
      (export "five" (core module
        (import "six" "a" (func))
        (import "six" "b" (func))
      ))
    ))
  ))
  ...
)
```

and module:

```wat
;; b.wasm
(module
  (import "six" "a" (func))
  (import "six" "b" (func))
  ...
)
```

could be successfully instantiated via:

```js
WebAssembly.instantiateStreaming(fetch('./a.wasm'), {
  one: () => (),
  two: "hi", 🪙
  three: {
    four: {
      five: await WebAssembly.compileStreaming(fetch('./b.wasm'))
    }
  }
});
```

The other significant addition to the JS API would be the expansion of the set
of WebAssembly types coerced to and from JavaScript values (by [`ToJSValue`]
and [`ToWebAssemblyValue`]) to include all of [`valtype`](#type-definitions).
At a high level, the additional coercions would be:

| Type               | `ToJSValue`                                                                                  | `ToWebAssemblyValue`                                                                 |
| ------------------ | -------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------ |
| `bool`             | `true` or `false`                                                                            | `ToBoolean`                                                                          |
| `s8`, `s16`, `s32` | as a Number value                                                                            | `ToInt8`, `ToInt16`, `ToInt32`                                                       |
| `u8`, `u16`, `u32` | as a Number value                                                                            | `ToUint8`, `ToUint16`, `ToUint32`                                                    |
| `s64`              | as a BigInt value                                                                            | `ToBigInt64`                                                                         |
| `u64`              | as a BigInt value                                                                            | `ToBigUint64`                                                                        |
| `f32`, `f64`       | as a Number value                                                                            | `ToNumber`                                                                           |
| `char`             | same as [`USVString`]                                                                        | same as [`USVString`], throw if the USV length is not 1                              |
| `record`           | TBD: maybe a [JS Record]?                                                                    | same as [`dictionary`]                                                               |
| `variant`          | see below                                                                                    | see below                                                                            |
| `list`             | create a typed array copy for number types; otherwise produce a JS array (like [`sequence`]) | same as [`sequence`]                                                                 |
| `string`           | same as [`USVString`]                                                                        | same as [`USVString`]                                                                |
| `tuple`            | TBD: maybe a [JS Tuple]?                                                                     | TBD                                                                                  |
| `flags`            | TBD: maybe a [JS Record]?                                                                    | same as [`dictionary`] of optional `boolean` fields with default values of `false`   |
| `enum`             | same as [`enum`]                                                                             | same as [`enum`]                                                                     |
| `option`           | same as [`T?`]                                                                               | same as [`T?`]                                                                       |
| `result`           | same as `variant`, but coerce a top-level `error` return value to a thrown exception         | same as `variant`, but coerce uncaught exceptions to top-level `error` return values |
| `own`, `borrow`    | see below                                                                                    | see below                                                                            |

Notes:

- Function parameter names are ignored since JavaScript doesn't have named
  parameters.
- If a function's result type list is empty, the JavaScript function returns
  `undefined`. If the result type list contains a single unnamed result, then
  the return value is specified by `ToJSValue` above. Otherwise, the function
  result is wrapped into a JS object whose field names are taken from the result
  names and whose field values are specified by `ToJSValue` above.
- In lieu of an existing standard JS representation for `variant`, the JS API
  would need to define its own custom binding built from objects. As a sketch,
  the JS values accepted by `(variant (case "a" u32) (case "b" string))` could
  include `{ tag: 'a', value: 42 }` and `{ tag: 'b', value: "hi" }`.
- For `option`, when Web IDL doesn't support particular type
  combinations (e.g., `(option (option u32))`), the JS API would fall back to
  the JS API of the unspecialized `variant` (e.g.,
  `(variant (case "some" (option u32)) (case "none"))`, despecializing only
  the problematic outer `option`).
- When coercing `ToWebAssemblyValue`, `own` and `borrow` handle types would
  dynamically guard that the incoming JS value's dynamic type was compatible
  with the imported resource type referenced by the handle type. For example,
  if a component contains `(import "Object" (type $Object (sub resource)))` and
  is instantiated with the JS `Object` constructor, then `(own $Object)` and
  `(borrow $Object)` could accept JS `object` values.
- When coercing `ToJSValue`, handle values would be wrapped with JS objects
  that are instances of the handles' resource type's exported constructor
  (described above). For `own` handles, a [`FinalizationRegistry`] would be
  used to drop the `own` handle (thereby calling the resource destructor) when
  its wrapper object was unreachable from JS. For `borrow` handles, the wrapper
  object would become dynamically invalid (throwing on any access) at the end
  of the export call.
- The forthcoming addition of [future and stream types] would allow `Promise`
  and `ReadableStream` values to be passed directly to and from components
  without requiring handles or callbacks.
- When an imported JavaScript function is a built-in function wrapping a Web
  IDL function, the specified behavior should allow the intermediate JavaScript
  call to be optimized away when the types are sufficiently compatible, falling
  back to a plain call through JavaScript when the types are incompatible or
  when the engine does not provide a separate optimized call path.

### ESM-integration

Like the JS API, [ESM-integration] can be extended to load components in all
the same places where modules can be loaded today, branching on the `layer`
field in the binary format to determine whether to decode as a module or a
component.

For URL import names, the embedded URL would be used as the [Module Specifier].
For plain names, the whole plain name would be used as the [Module Specifier]
(and an import map would be needed to map the string to a URL). For locked and
unlocked dependency names, ESM-integration would likely simply fail loading the
module, requiring a bundler to map these registry-relative names to URLs.

TODO: ESM-integration for interface imports and exports is still being
worked out in detail.

The main remaining question is how to deal with component imports having a
single string as well as the new importable component, module and instance
types. Going through these one by one:

For component imports of module type, we need a new way to request that the ESM
loader parse or decode a module without _also_ instantiating that module.
Recognizing this same need from JavaScript, there is a TC39 proposal called
[Import Reflection] that adds the ability to write, in JavaScript:

```js
import Foo from "./foo.wasm" as "wasm-module";
assert(Foo instanceof WebAssembly.Module);
```

With this extension to JavaScript and the ESM loader, a component import
of module type can be treated the same as `import ... as "wasm-module"`.

Component imports of component type would work the same way as modules,
potentially replacing `"wasm-module"` with `"wasm-component"`.

In all other cases, the (single) string imported by a component is first
resolved to a [Module Record] using the same process as resolving the
[Module Specifier] of a JavaScript `import`. After this, the handling of the
imported Module Record is determined by the import type:

For imports of instance type, the ESM loader would treat the exports of the
instance type as if they were the [Named Imports] of a JavaScript `import`.
Thus, single-level imports of instance type act like the two-level imports
of Core WebAssembly modules where the first-level has been factored out. Since
the exports of an instance type can themselves be instance types, this process
must be performed recursively.

Otherwise, function or value imports are treated like an [Imported Default Binding]
and the Module Record is converted to its default value. This allows the following
component:

```wat
;; bar.wasm
(component
  (import "./foo.js" (func (result string)))
  ...
)
```

to be satisfied by a JavaScript module via ESM-integration:

```js
// foo.js
export default () => "hi";
```

when `bar.wasm` is loaded as an ESM:

```html
<script src="bar.wasm" type="module"></script>
```

## References

[JS API]: https://webassembly.github.io/spec/js-api/index.html
[*read the imports*]: https://webassembly.github.io/spec/js-api/index.html#read-the-imports
[*create an exports object*]: https://webassembly.github.io/spec/js-api/index.html#create-an-exports-object
[Interface Object]: https://webidl.spec.whatwg.org/#interface-object
[`ToJSValue`]: https://webassembly.github.io/spec/js-api/index.html#tojsvalue
[`ToWebAssemblyValue`]: https://webassembly.github.io/spec/js-api/index.html#towebassemblyvalue
[`USVString`]: https://webidl.spec.whatwg.org/#es-USVString
[`sequence`]: https://webidl.spec.whatwg.org/#es-sequence
[`dictionary`]: https://webidl.spec.whatwg.org/#es-dictionary
[`enum`]: https://webidl.spec.whatwg.org/#es-enumeration
[`T?`]: https://webidl.spec.whatwg.org/#es-nullable-type
[`Get`]: https://tc39.es/ecma262/#sec-get-o-p
[Import Reflection]: https://github.com/tc39-transfer/proposal-import-reflection
[Module Record]: https://tc39.es/ecma262/#sec-abstract-module-records
[Module Specifier]: https://tc39.es/ecma262/multipage/ecmascript-language-scripts-and-modules.html#prod-ModuleSpecifier
[Named Imports]: https://tc39.es/ecma262/multipage/ecmascript-language-scripts-and-modules.html#prod-NamedImports
[Imported Default Binding]: https://tc39.es/ecma262/multipage/ecmascript-language-scripts-and-modules.html#prod-ImportedDefaultBinding
[JS Tuple]: https://github.com/tc39/proposal-record-tuple
[JS Record]: https://github.com/tc39/proposal-record-tuple
[Internal Slot]: https://tc39.es/ecma262/#sec-object-internal-methods-and-internal-slots
[Built-in Modules]: https://github.com/tc39/proposal-built-in-modules
[`FinalizationRegistry`]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/FinalizationRegistry
[`WebAssembly.instantiate()`]: https://developer.mozilla.org/en-US/docs/WebAssembly/JavaScript_interface/instantiate
