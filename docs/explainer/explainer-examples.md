# Component Model Examples and Invariants

This document provides examples of the WebAssembly Component Model in action and explains component invariants.

## Component Invariants

As a consequence of the shared-nothing design described above, all calls into
or out of a component instance necessarily transit through a component function
definition. Thus, component functions form a "membrane" around the collection
of core module instances contained by a component instance, allowing the
Component Model to establish invariants that increase optimizability and
composability in ways not otherwise possible in the shared-everything setting
of Core WebAssembly. The Component Model proposes establishing the following
three runtime invariants:

1. Components define a "lockdown" state that prevents continued execution
   after a trap. This both prevents continued execution with corrupt state and
   also allows more-aggressive compiler optimizations (e.g., store reordering).
   This was considered early in Core WebAssembly standardization but rejected
   due to the lack of clear trapping boundary. With components, each component
   instance is given a mutable "lockdown" state that is set upon trap and
   implicitly checked at every execution step by component functions. Thus,
   after a trap, it's no longer possible to observe the internal state of a
   component instance.
2. The Component Model disallows reentrance by trapping if a callee's
   component-instance is already on the stack when the call starts.
   (For details, see [`trap_if_on_stack`](CanonicalABI.md#task-state)
   in the Canonical ABI explainer.) This default prevents obscure
   composition-time bugs and also enables more-efficient non-reentrant
   runtime glue code. This rule will be relaxed by an opt-in
   function type attribute in the [future](Async.md#todo).

## Examples

For some use-case-focused, worked examples, see:

- [Link-time virtualization example](examples/LinkTimeVirtualization.md)
- [Shared-everything dynamic linking example](examples/SharedEverythingDynamicLinking.md)
- [Component Examples presentation](https://docs.google.com/presentation/d/11lY9GBghZJ5nCFrf4MKWVrecQude0xy_buE--tnO9kQ)

[Structure Section]: https://webassembly.github.io/spec/core/syntax/index.html
[Text Format Section]: https://webassembly.github.io/spec/core/text/index.html
[Binary Format Section]: https://webassembly.github.io/spec/core/binary/index.html
[Core Indices]: https://webassembly.github.io/spec/core/syntax/modules.html#indices
[Core Identifiers]: https://webassembly.github.io/spec/core/text/values.html#text-id
[Index Space]: https://webassembly.github.io/spec/core/syntax/modules.html#indices
[Abbreviations]: https://webassembly.github.io/spec/core/text/conventions.html#abbreviations
[`core:i64`]: https://webassembly.github.io/spec/core/text/values.html#text-int
[`core:f64`]: https://webassembly.github.io/spec/core/syntax/values.html#floating-point
[`core:stringchar`]: https://webassembly.github.io/spec/core/text/values.html#text-string
[`core:name`]: https://webassembly.github.io/spec/core/syntax/values.html#syntax-name
[`core:module`]: https://webassembly.github.io/spec/core/text/modules.html#text-module
[`core:type`]: https://webassembly.github.io/spec/core/text/modules.html#types
[`core:importdesc`]: https://webassembly.github.io/spec/core/text/modules.html#text-importdesc
[`core:externtype`]: https://webassembly.github.io/spec/core/syntax/types.html#external-types
[`core:valtype`]: https://webassembly.github.io/spec/core/text/types.html#value-types
[`core:typeuse`]: https://webassembly.github.io/spec/core/text/modules.html#type-uses
[`core:functype`]: https://webassembly.github.io/spec/core/text/types.html#function-types
[`core:datastring`]: https://webassembly.github.io/spec/core/text/modules.html#text-datastring
[func-import-abbrev]: https://webassembly.github.io/spec/core/text/modules.html#text-func-abbrev
[`core:version`]: https://webassembly.github.io/spec/core/binary/modules.html#binary-version
[Embedder]: https://webassembly.github.io/spec/core/appendix/embedding.html
[`module_instantiate`]: https://webassembly.github.io/spec/core/appendix/embedding.html#mathrm-module-instantiate-xref-exec-runtime-syntax-store-mathit-store-xref-syntax-modules-syntax-module-mathit-module-xref-exec-runtime-syntax-externval-mathit-externval-ast-xref-exec-runtime-syntax-store-mathit-store-xref-exec-runtime-syntax-moduleinst-mathit-moduleinst-xref-appendix-embedding-embed-error-mathit-error
[`func_invoke`]: https://webassembly.github.io/spec/core/appendix/embedding.html#mathrm-func-invoke-xref-exec-runtime-syntax-store-mathit-store-xref-exec-runtime-syntax-funcaddr-mathit-funcaddr-xref-exec-runtime-syntax-val-mathit-val-ast-xref-exec-runtime-syntax-store-mathit-store-xref-exec-runtime-syntax-val-mathit-val-ast-xref-appendix-embedding-embed-error-mathit-error
[`func_alloc`]: https://webassembly.github.io/spec/core/appendix/embedding.html#mathrm-func-alloc-xref-exec-runtime-syntax-store-mathit-store-xref-syntax-types-syntax-functype-mathit-functype-xref-exec-runtime-syntax-hostfunc-mathit-hostfunc-xref-exec-runtime-syntax-store-mathit-store-xref-exec-runtime-syntax-funcaddr-mathit-funcaddr
[`WebAssembly.instantiate()`]: https://developer.mozilla.org/en-US/docs/WebAssembly/JavaScript_interface/instantiate
[`FinalizationRegistry`]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/FinalizationRegistry
[Fetching]: https://fetch.spec.whatwg.org/
[Parsing]: https://url.spec.whatwg.org/#url-parsing
[Base URL]: https://url.spec.whatwg.org/#concept-base-url
[`integrity-metadata`]: https://www.w3.org/TR/SRI/#the-integrity-attribute
[Semantic Versioning 2.0]: https://semver.org/spec/v2.0.0.html
[Delimit The URL]: https://www.rfc-editor.org/rfc/rfc3986#appendix-C
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
[Kebab Case]: https://en.wikipedia.org/wiki/Letter_case#Kebab_case
[De Bruijn Index]: https://en.wikipedia.org/wiki/De_Bruijn_index
[Closure]: https://en.wikipedia.org/wiki/Closure_(computer_programming)
[Empty Type]: https://en.wikipedia.org/w/index.php?title=Empty_type
[IEEE754]: https://en.wikipedia.org/wiki/IEEE_754
[Unicode Scalar Values]: https://unicode.org/glossary/#unicode_scalar_value
[Tuples]: https://en.wikipedia.org/wiki/Tuple
[Tagged Unions]: https://en.wikipedia.org/wiki/Tagged_union
[Sequences]: https://en.wikipedia.org/wiki/Sequence
[ABI]: https://en.wikipedia.org/wiki/Application_binary_interface
[Environment Variables]: https://en.wikipedia.org/wiki/Environment_variable
[Linear]: https://en.wikipedia.org/wiki/Substructural_type_system#Linear_type_systems
[Interface Definition Language]: https://en.wikipedia.org/wiki/Interface_description_language
[Subtyping]: https://en.wikipedia.org/wiki/Subtyping
[Universal Types]: https://en.wikipedia.org/wiki/System_F
[Existential Types]: https://en.wikipedia.org/wiki/System_F
[Unit]: https://en.wikipedia.org/wiki/Unit_type
[Generative]: https://www.researchgate.net/publication/2426300_A_Syntactic_Theory_of_Type_Generativity_and_Sharing
[Avoidance Problem]: https://counterexamples.org/avoidance.html
[Non-Parametric Parametricity]: https://people.mpi-sws.org/~dreyer/papers/npp/main.pdf
[module-linking]: https://github.com/WebAssembly/module-linking/blob/main/proposals/module-linking/Explainer.md
[interface-types]: https://github.com/WebAssembly/interface-types/blob/main/proposals/interface-types/Explainer.md
[type-imports]: https://github.com/WebAssembly/proposal-type-imports/blob/master/proposals/type-imports/Overview.md
[exception-handling]: https://github.com/WebAssembly/exception-handling/blob/main/proposals/exception-handling/Exceptions.md
[stack-switching]: https://github.com/WebAssembly/stack-switching/blob/main/proposals/stack-switching/Explainer.md
[esm-integration]: https://github.com/WebAssembly/esm-integration/tree/main/proposals/esm-integration
[gc]: https://github.com/WebAssembly/gc/blob/main/proposals/gc/MVP.md
[`rectype`]: https://webassembly.github.io/gc/core/text/types.html#text-rectype
[shared-everything-threads]: https://github.com/WebAssembly/shared-everything-threads
[WASI Preview 2]: https://github.com/WebAssembly/WASI/tree/main/wasip2#readme
[reference types]: https://github.com/WebAssembly/reference-types/blob/master/proposals/reference-types/Overview.md
[Strongly-unique]: #name-uniqueness
[Adapter Functions]: FutureFeatures.md#custom-abis-via-adapter-functions
[Canonical ABI explainer]: CanonicalABI.md
[`canon_context_get`]: CanonicalABI.md#-canon-contextget
[`canon_context_set`]: CanonicalABI.md#-canon-contextset
[`canon_backpressure_set`]: CanonicalABI.md#-canon-backpressureset
[`canon_task_return`]: CanonicalABI.md#-canon-taskreturn
[`canon_yield`]: CanonicalABI.md#-canon-yield
[`canon_waitable_set_new`]: CanonicalABI.md#-canon-waitable-setnew
[`canon_waitable_set_wait`]: CanonicalABI.md#-canon-waitable-setwait
[`canon_waitable_set_poll`]: CanonicalABI.md#-canon-waitable-setpoll
[`canon_waitable_set_drop`]: CanonicalABI.md#-canon-waitable-setdrop
[`canon_waitable_join`]: CanonicalABI.md#-canon-waitablejoin
[`canon_stream_new`]: CanonicalABI.md#-canon-streamfuturenew
[`canon_stream_read`]: CanonicalABI.md#-canon-streamfuturereadwrite
[`canon_future_read`]: CanonicalABI.md#-canon-streamfuturereadwrite
[`canon_stream_cancel_read`]: CanonicalABI.md#-canon-streamfuturecancel-readwrite
[`canon_subtask_drop`]: CanonicalABI.md#-canon-subtaskdrop
[`canon_resource_new`]: CanonicalABI.md#canon-resourcenew
[`canon_resource_drop`]: CanonicalABI.md#canon-resourcedrop
[`canon_resource_rep`]: CanonicalABI.md#canon-resourcerep
[`canon_error_context_new`]: CanonicalABI.md#-canon-error-contextnew
[`canon_error_context_debug_message`]: CanonicalABI.md#-canon-error-contextdebug-message
[`canon_error_context_drop`]: CanonicalABI.md#-canon-error-contextdrop
[`canon_thread_spawn_ref`]: CanonicalABI.md#-canon-threadspawn_ref
[`canon_thread_spawn_indirect`]: CanonicalABI.md#-canon-threadspawn_indirect
[`canon_thread_available_parallelism`]: CanonicalABI.md#-canon-threadavailable_parallelism
[`pack_async_copy_result`]: CanonicalABI.md#-canon-streamfuturereadwrite
[the `close` built-ins]: CanonicalABI.md#-canon-streamfutureclose-readablewritable
[Shared-Nothing]: ../high-level/Choices.md
[Use Cases]: ../high-level/UseCases.md
[Host Embeddings]: ../high-level/UseCases.md#hosts-embedding-components
[Task]: Async.md#task
[Current Task]: Async.md#current-task
[Context-Local Storage]: Async.md#context-local-storage
[Subtask]: Async.md#subtask
[Stream or Future]: Async.md#streams-and-futures
[Readable or Writable End]: Async.md#streams-and-futures
[Writable End]: Async.md#streams-and-futures
[Waiting]: Async.md#waiting
[Waitables]: Async.md#waiting
[Waitable Set]: Async.md#waiting
[Backpressure]: Async.md#backpressure
[Returning]: Async.md#returning
[Component Model Documentation]: https://component-model.bytecodealliance.org
[`wizer`]: https://github.com/bytecodealliance/wizer
[`warg`]: https://warg.io
[SemVerRange]: https://semver.npmjs.com/
[OCI Registry]: https://github.com/opencontainers/distribution-spec
[Scoping and Layering]: https://docs.google.com/presentation/d/1PSC3Q5oFsJEaYyV5lNJvVgh-SNxhySWUqZ6puyojMi8
[Future and Stream Types]: https://docs.google.com/presentation/d/1MNVOZ8hdofO3tI0szg_i-Yoy0N2QPU2C--LzVuoGSlE
[`navigator.hardwareConcurrency`]: https://developer.mozilla.org/en-US/docs/Web/API/Navigator/hardwareConcurrency

## References

[Link-time virtualization example]: examples/LinkTimeVirtualization.md
[Shared-everything dynamic linking example]: examples/SharedEverythingDynamicLinking.md
