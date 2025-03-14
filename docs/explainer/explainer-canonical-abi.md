# Canonical Definitions and ABI

This document explains the canonical definitions and ABI in the WebAssembly Component Model, including canonical function definitions and built-ins.

### Canonical Definitions

From the perspective of Core WebAssembly running inside a component, the
Component Model is an [embedder]. As such, the Component Model defines the
Core WebAssembly imports passed to [`module_instantiate`] and how Core
WebAssembly exports are called via [`func_invoke`]. This allows the Component
Model to specify how core modules are linked together (as shown above) but it
also allows the Component Model to arbitrarily synthesize Core WebAssembly
functions (via [`func_alloc`]) that are imported by Core WebAssembly. These
synthetic core functions are created via one of several *canonical definitions*
defined below.

#### Canonical ABI

To implement or call a component-level function, we need to cross a
shared-nothing boundary. Traditionally, this problem is solved by defining a
serialization format. The Component Model MVP uses roughly this same approach,
defining a linear-memory-based [ABI] called the "Canonical ABI" which
specifies, for any `functype`, a [corresponding](CanonicalABI.md#flattening)
`core:functype` and [rules](CanonicalABI.md#lifting-and-lowering) for copying
values into and out of linear memory. The Component Model differs from
traditional approaches, though, in that the ABI is configurable, allowing
multiple different memory representations of the same abstract value. In the
MVP, this configurability is limited to the small set of `canonopt` shown
below. However, Post-MVP, [adapter functions] could be added to allow far more
programmatic control.

The Canonical ABI is explicitly applied to "wrap" existing functions in one of
two directions:
* `lift` wraps a core function (of type `core:functype`) to produce a component
  function (of type `functype`) that can be passed to other components.
* `lower` wraps a component function (of type `functype`) to produce a core
  function (of type `core:functype`) that can be imported and called from Core
  WebAssembly code inside the current component.

Canonical definitions specify one of these two wrapping directions, the function
to wrap and a list of configuration options:
```ebnf
canon    ::= (canon lift core-prefix(<core:funcidx>) <canonopt>* bind-id(<externdesc>))
           | (canon lower <funcidx> <canonopt>* (core func <id>?))
canonopt ::= string-encoding=utf8
           | string-encoding=utf16
           | string-encoding=latin1+utf16
           | (memory <core:memidx>)
           | (realloc <core:funcidx>)
           | (post-return <core:funcidx>)
           | async 🔀
           | (callback <core:funcidx>) 🔀
           | always-task-return 🔀
```
While the production `externdesc` accepts any `sort`, the validation rules
for `canon lift` would only allow the `func` sort. In the future, other sorts
may be added (viz., types), hence the explicit sort.

The `string-encoding` option specifies the encoding the Canonical ABI will use
for the `string` type. The `latin1+utf16` encoding captures a common string
encoding across Java, JavaScript and .NET VMs and allows a dynamic choice
between either Latin-1 (which has a fixed 1-byte encoding, but limited Code
Point range) or UTF-16 (which can express all Code Points, but uses either
2 or 4 bytes per Code Point). If no `string-encoding` option is specified, the
default is `utf8`. It is a validation error to include more than one
`string-encoding` option.

The `(memory ...)` option specifies the memory that the Canonical ABI will
use to load and store values. If the Canonical ABI needs to load or store,
validation requires this option to be present (there is no default).

The `(realloc ...)` option specifies a core function that is validated to
have the following core function type:
```wat
(func (param $originalPtr i32)
      (param $originalSize i32)
      (param $alignment i32)
      (param $newSize i32)
      (result i32))
```
The Canonical ABI will use `realloc` both to allocate (passing `0` for the
first two parameters) and reallocate. If the Canonical ABI needs `realloc`,
validation requires this option to be present (there is no default).

The `(post-return ...)` option may only be present in `canon lift` when
`async` is not present and specifies a core function to be called with the
original return values after they have finished being read, allowing memory to
be deallocated and destructors called. This immediate is always optional but,
if present, is validated to have parameters matching the callee's return type
and empty results.

🔀 The `async` option specifies that the component wants to make (for imports)
or support (for exports) multiple concurrent (asynchronous) calls. This option
can be applied to any component-level function type and changes the derived
Canonical ABI significantly. See the [async explainer](Async.md) for more
details. When a function signature contains a `future` or `stream`, validation
of `canon lower` requires the `async` option to be set (since a synchronous
call to a function using these types is highly likely to deadlock).

🔀 The `(callback ...)` option may only be present in `canon lift` when the
`async` option has also been set and specifies a core function that is
validated to have the following core function type:
```wat
(func (param $ctx i32)
      (param $event i32)
      (param $payload i32)
      (result $done i32))
```
Again, see the [async explainer](Async.md) for more details.

🔀 The `always-task-return` option may only be present in `canon lift` when
`post-return` is not set and specifies that even synchronously-lifted functions
will call `canon task.return` to return their results instead of returning
them as core function results. This is a simpler alternative to `post-return`
for freeing memory after lifting and thus `post-return` may be deprecated in
the future.

Based on this description of the AST, the [Canonical ABI explainer] gives a
detailed walkthrough of the static and dynamic semantics of `lift` and `lower`.

One high-level consequence of the dynamic semantics of `canon lift` given in
the Canonical ABI explainer is that component functions are different from core
functions in that all control flow transfer is explicitly reflected in their
type. For example, with Core WebAssembly [exception-handling] and
[stack-switching], a core function with type `(func (result i32))` can return
an `i32`, throw, suspend or trap. In contrast, a component function with type
`(func (result string))` may only return a `string` or trap. To express
failure, component functions can return `result` and languages with exception
handling can bind exceptions to the `error` case. Similarly, the forthcoming
addition of [future and stream types] would explicitly declare patterns of
stack-switching in component function signatures.

Similar to the `import` and `alias` abbreviations shown above, `canon`
definitions can also be written in an inverted form that puts the sort first:
```wat
(func $f (import "i" "f") ...type...) ≡ (import "i" "f" (func $f ...type...))       (WebAssembly 1.0)
(func $g ...type... (canon lift ...)) ≡ (canon lift ... (func $g ...type...))
(core func $h (canon lower ...))      ≡ (canon lower ... (core func $h))
```
Note: in the future, `canon` may be generalized to define other sorts than
functions (such as types), hence the explicit `sort`.

Using canonical function definitions, we can finally write a non-trivial
component that takes a string, does some logging, then returns a string.
```wat
(component
  (import "logging" (instance $logging
    (export "log" (func (param string)))
  ))
  (import "libc" (core module $Libc
    (export "mem" (memory 1))
    (export "realloc" (func (param i32 i32) (result i32)))
  ))
  (core instance $libc (instantiate $Libc))
  (core func $log (canon lower
    (func $logging "log")
    (memory (core memory $libc "mem")) (realloc (func $libc "realloc"))
  ))
  (core module $Main
    (import "libc" "memory" (memory 1))
    (import "libc" "realloc" (func (param i32 i32) (result i32)))
    (import "logging" "log" (func $log (param i32 i32)))
    (func (export "run") (param i32 i32) (result i32)
      ... (call $log) ...
    )
  )
  (core instance $main (instantiate $Main
    (with "libc" (instance $libc))
    (with "logging" (instance (export "log" (func $log))))
  ))
  (func $run (param string) (result string) (canon lift
    (core func $main "run")
    (memory (core memory $libc "mem")) (realloc (func $libc "realloc"))
  ))
  (export "run" (func $run))
)
```
This example shows the pattern of splitting out a reusable language runtime
module (`$Libc`) from a component-specific, non-reusable module (`$Main`). In
addition to reducing code size and increasing code-sharing in multi-component
scenarios, this separation allows `$libc` to be created first, so that its
exports are available for reference by `canon lower`. Without this separation
(if `$Main` contained the `memory` and allocation functions), there would be a
cyclic dependency between `canon lower` and `$Main` that would have to be
broken using an auxiliary module performing `call_indirect`.

#### Canonical Built-ins

In addition to the `lift` and `lower` canonical function definitions which
adapt *existing* functions, there are also a set of canonical "built-ins" that
define core functions out of nothing that can be imported by core modules to
dynamically interact with Canonical ABI entities like resources and
[tasks][Future and Stream Types] 🔀.
```ebnf
canon ::= ...
        | (canon resource.new <typeidx> (core func <id>?))
        | (canon resource.drop <typeidx> async? (core func <id>?))
        | (canon resource.rep <typeidx> (core func <id>?))
        | (canon context.get <valtype> <u32> (core func <id>?)) 🔀
        | (canon context.set <valtype> <u32> (core func <id>?)) 🔀
        | (canon backpressure.set (core func <id>?)) 🔀
        | (canon task.return (result <valtype>)? <canonopt>* (core func <id>?)) 🔀
        | (canon yield async? (core func <id>?)) 🔀
        | (canon waitable-set.new (core func <id>?)) 🔀
        | (canon waitable-set.wait async? (memory <core:memidx>) (core func <id>?)) 🔀
        | (canon waitable-set.poll async? (memory <core:memidx>) (core func <id>?)) 🔀
        | (canon waitable-set.drop (core func <id>?)) 🔀
        | (canon waitable.join (core func <id>?)) 🔀
        | (canon subtask.drop (core func <id>?)) 🔀
        | (canon stream.new <typeidx> (core func <id>?)) 🔀
        | (canon stream.read <typeidx> <canonopt>* (core func <id>?)) 🔀
        | (canon stream.write <typeidx> <canonopt>* (core func <id>?)) 🔀
        | (canon stream.cancel-read <typeidx> async? (core func <id>?)) 🔀
        | (canon stream.cancel-write <typeidx> async? (core func <id>?)) 🔀
        | (canon stream.close-readable <typeidx> (core func <id>?)) 🔀
        | (canon stream.close-writable <typeidx> (core func <id>?)) 🔀
        | (canon future.new <typeidx> (core func <id>?)) 🔀
        | (canon future.read <typeidx> <canonopt>* (core func <id>?)) 🔀
        | (canon future.write <typeidx> <canonopt>* (core func <id>?)) 🔀
        | (canon future.cancel-read <typeidx> async? (core func <id>?)) 🔀
        | (canon future.cancel-write <typeidx> async? (core func <id>?)) 🔀
        | (canon future.close-readable <typeidx> (core func <id>?)) 🔀
        | (canon future.close-writable <typeidx> (core func <id>?)) 🔀
        | (canon error-context.new <canonopt>* (core func <id>?))
        | (canon error-context.debug-message <canonopt>* (core func <id>?))
        | (canon error-context.drop (core func <id>?))
        | (canon thread.spawn_ref <typeidx> (core func <id>?)) 🧵
        | (canon thread.spawn_indirect <typeidx> <core:tableidx> (core func <id>?)) 🧵
        | (canon thread.available_parallelism (core func <id>?)) 🧵
```

##### Resource built-ins

###### `resource.new`

| Synopsis                   |                            |
| -------------------------- | -------------------------- |
| Approximate WIT signature  | `func<T>(rep: T.rep) -> T` |
| Canonical ABI signature    | `[rep:i32] -> [i32]`       |

The `resource.new` built-in creates a new resource (of resource type `T`) with
`rep` as its representation, and returns a new handle pointing to the new
resource. Validation only allows `resource.rep T` to be used within the
component that defined `T`.

In the Canonical ABI, `T.rep` is defined to be the `$rep` in the
`(type $T (resource (rep $rep) ...))` type definition that defined `T`. While
it's designed to allow different types in the future, it is currently
hard-coded to always be `i32`.

(See also [`canon_resource_new`] in the Canonical ABI explainer.)

###### `resource.drop`

When the `async` immediate is false:

| Synopsis                   |                                    |
| -------------------------- | ---------------------------------- |
| Approximate WIT signature  | `func<T>(t: T)`                    |
| Canonical ABI signature    | `[t:i32] -> []`                    |

When the `async` immediate is true:

| Synopsis                   |                                    |
| -------------------------- | ---------------------------------- |
| Approximate WIT signature  | `func<T>(t: T) -> option<subtask>` |
| Canonical ABI signature    | `[t:i32] -> [i32]`                 |

The `resource.drop` built-in drops a resource handle `t` (with resource type
`T`). If the dropped handle owns the resource, the resource's `dtor` is called,
if present. Validation only allows `resource.rep T` to be used within the
component that defined `T`.

When the `async` immediate is true, the returned value indicates whether the
drop completed eagerly, or if not, identifies the in-progress drop.

In the Canonical ABI, the returned `i32` is either `0` (if the drop completed
eagerly) or the index of the in-progress drop subtask (representing the
in-progress `dtor` call). (See also [`canon_resource_drop`] in the Canonical
ABI explainer.)

###### `resource.rep`

| Synopsis                   |                          |
| -------------------------- | ------------------------ |
| Approximate WIT signature  | `func<T>(t: T) -> T.rep` |
| Canonical ABI signature    | `[t:i32] -> [i32]`       |

The `resource.rep` built-in returns the representation of the resource (with
resource type `T`) pointed to by the handle `t`. Validation only allows
`resource.rep T` to be used within the component that defined `T`.

In the Canonical ABI, `T.rep` is defined to be the `$rep` in the
`(type $T (resource (rep $rep) ...))` type definition that defined `T`. While
it's designed to allow different types in the future, it is currently
hard-coded to always be `i32`.

As an example, the following component imports the `resource.new` built-in,
allowing it to create and return new resources to its client:
```wat
(component
  (import "Libc" (core module $Libc ...))
  (core instance $libc (instantiate $Libc))
  (type $R (resource (rep i32) (dtor (func $libc "free"))))
  (core func $R_new (param i32) (result i32)
    (canon resource.new $R)
  )
  (core module $Main
    (import "canon" "R_new" (func $R_new (param i32) (result i32)))
    (func (export "make_R") (param ...) (result i32)
      (return (call $R_new ...))
    )
  )
  (core instance $main (instantiate $Main
    (with "canon" (instance (export "R_new" (func $R_new))))
  ))
  (export $R' "r" (type $R))
  (func (export "make-r") (param ...) (result (own $R'))
    (canon lift (core func $main "make_R"))
  )
)
```
Here, the `i32` returned by `resource.new`, which is an index into the
component's handle-table, is immediately returned by `make_R`, thereby
transferring ownership of the newly-created resource to the export's caller.
(See also [`canon_resource_rep`] in the Canonical ABI explainer.)

##### 🔀 Async built-ins

See the [async explainer](Async.md) for high-level context and terminology and
the [Canonical ABI explainer] for detailed runtime semantics.

###### 🔀 `context.get`

| Synopsis                   |                    |
| -------------------------- | ------------------ |
| Approximate WIT signature  | `func<T,i>() -> T` |
| Canonical ABI signature    | `[] -> [T]`        |

The `context.get` built-in returns the `i`th element of the [current task]'s
[context-local storage] array. Validation currently restricts `i` to be less
than 2 and `t` to be `i32`, but will be relaxed in the future (as described
[here][context-local storage]). (See also [`canon_context_get`] in the
Canonical ABI explainer for details.)

###### 🔀 `context.set`

| Synopsis                   |                   |
| -------------------------- | ----------------- |
| Approximate WIT signature  | `func<T,i>(v: T)` |
| Canonical ABI signature    | `[T] -> []`       |

The `context.set` built-in sets the `i`th element of the [current task]'s
[context-local storage] array to the value `v`. Validation currently
restricts `i` to be less than 2 and `t` to be `i32`, but will be relaxed in the
future (as described [here][context-local storage]). (See also
[`canon_context_set`] in the Canonical ABI explainer for details.)

###### 🔀 `backpressure.set`

| Synopsis                   |                       |
| -------------------------- | --------------------- |
| Approximate WIT signature  | `func(enable: bool)`  |
| Canonical ABI signature    | `[enable:i32] -> []`  |

The `backpressure.set` built-in allows the async-lifted callee to toggle a
per-component-instance flag that, when set, prevents new incoming export calls
to the component (until the flag is unset). This allows the component to exert
[backpressure]. (See also [`canon_backpressure_set`] in the Canonical ABI
explainer for details.)

###### 🔀 `task.return`

The `task.return` built-in takes as parameters the result values of the
currently-executing task. This built-in must be called exactly once per export
activation. The `canon task.return` definition takes component-level return
type and the list of `canonopt` to be used to lift the return value. When
called, the declared return type and the `string-encoding` and `memory`
`canonopt`s are checked to exactly match those of the current task. (See also
"[Returning]" in the async explainer and [`canon_task_return`] in the Canonical
ABI explainer.)

###### 🔀 `yield`

| Synopsis                   |                    |
| -------------------------- | ------------------ |
| Approximate WIT signature  | `func<async?>()`   |
| Canonical ABI signature    | `[] -> []`         |

The `yield` built-in allows the runtime to switch to other tasks, enabling a
long-running computation to cooperatively interleave execution. If the `async`
immediate is present, the runtime can switch to other tasks in the *same*
component instance, which the calling core wasm must be prepared to handle. If
`async` is not present, only tasks in *other* component instances may be
switched to. (See also [`canon_yield`] in the Canonical ABI explainer for
details.)

###### 🔀 `waitable-set.new`

| Synopsis                   |                          |
| -------------------------- | ------------------------ |
| Approximate WIT signature  | `func() -> waitable-set` |
| Canonical ABI signature    | `[] -> [i32]`            |

The `waitable-set.new` built-in returns the `i32` index of a new [waitable
set]. The `waitable-set` type is not a true WIT-level type but instead serves
to document associated built-ins below. Waitable sets start out empty and are
populated explicitly with [waitables] by `waitable.join`. (See also
[`canon_waitable_set_new`] in the Canonical ABI explainer for details.)

###### 🔀 `waitable-set.wait`

| Synopsis                   |                                                |
| -------------------------- | ---------------------------------------------- |
| Approximate WIT signature  | `func<async?>(s: waitable-set) -> event`       |
| Canonical ABI signature    | `[s:i32 payload-addr:i32] -> [event-code:i32]` |

where `event`, `event-code`, and `payload` are defined in WIT as:
```wit
record event {
    kind: event-code,
    payload: payload,
}
enum event-code {
    none,
    call-starting,
    call-started,
    call-returned,
    stream-read,
    stream-write,
    future-read,
    future-write,
}
record payload {
    payload1: u32,
    payload2: u32,
}
```

The `waitable-set.wait` built-in waits for any one of the [waitables] in the
given [waitable set] `s` to make progress and then returns an `event`
describing the event. The `event-code` `none` is never returned. Waitable sets
may be `wait`ed upon when empty, in which case the caller will necessarily
block until another task adds a waitable to the set that can make progress.

If the `async` immediate is present, other tasks in the same component instance
can be started (via export call) or resumed while the current task blocks. If
`async` is not present, the current component instance will not execute any
code until `wait` returns (however, *other* component instances may execute
code in the interim).

In the Canonical ABI, the return value provides the `event-code`, and the
`payload` value is stored at the address passed as the `payload-addr`
parameter. (See also [`canon_waitable_set_wait`] in the Canonical ABI explainer
for details.)

###### 🔀 `waitable-set.poll`

| Synopsis                   |                                                |
| -------------------------- | ---------------------------------------------- |
| Approximate WIT signature  | `func<async?>(s: waitable-set) -> event`       |
| Canonical ABI signature    | `[s:i32 payload-addr:i32] -> [event-code:i32]` |

where `event`, `event-code`, and `payload` are defined as in
[`waitable-set.wait`](#-waitable-setwait).

The `waitable-set.poll` built-in returns the `event-code` `none` if no event
was available without blocking. `poll` implicitly performs a `yield`, allowing
other tasks to be scheduled before `poll` returns. The `async?` immediate is
passed to `yield`, determining whether other code in the same component
instance may execute.

The Canonical ABI of `waitable-set.poll` is the same as `waitable-set.wait`
(with the `none` case indicated by returning `0`). (See also
[`canon_waitable_set_poll`] in the Canonical ABI explainer for details.)

###### 🔀 `waitable-set.drop`

| Synopsis                   |                          |
| -------------------------- | ------------------------ |
| Approximate WIT signature  | `func(s: waitable-set)` |
| Canonical ABI signature    | `[s:i32] -> []`    |

The `waitable-set.drop` built-in removes the indicated [waitable set] from the
current instance's table of waitable sets, trapping if the waitable set is not
empty or if another task is concurrently `wait`ing on it. (See also
[`canon_waitable_set_drop`] in the Canonical ABI explainer for details.)

###### 🔀 `waitable.join`

| Synopsis                   |                                                      |
| -------------------------- | ---------------------------------------------------- |
| Approximate WIT signature  | `func(w: waitable, maybe_set: option<waitable-set>)` |
| Canonical ABI signature    | `[w:i32, maybe_set:i32] -> []`                       |

The `waitable.join` built-in may be called given a [waitable] and an optional
[waitable set]. `join` first removes `w` from any waitable set that it is a
member of and then, if `maybe_set` is not `none`, `w` is added to that set.
Thus, `join` can be used to arbitrarily add, change and remove waitables from
waitable sets in the same component instance, preserving the invariant that a
waitable can be in at most one set.

In the Canonical ABI, `w` is an index into the component instance's [waitables]
table and can be any type of waitable (`subtask` or
`{readable,writable}-{stream,future}-end`). A value of `0` represents a `none`
`maybe_set`, since `0` is not a valid table index. (See also
[`canon_waitable_join`] in the Canonical ABI explainer for details.)

###### 🔀 `subtask.drop`

| Synopsis                   |                          |
| -------------------------- | ------------------------ |
| Approximate WIT signature  | `func(subtask: subtask)` |
| Canonical ABI signature    | `[subtask:i32] -> []`    |

The `subtask.drop` built-in removes the indicated [subtask] from the current
instance's table of [waitables], trapping if the subtask hasn't returned. (See
[`canon_subtask_drop`] in the Canonical ABI explainer for details.)

###### 🔀 `stream.new` and `future.new`

| Synopsis                                   |                                       |
| ------------------------------------------ | ------------------------------------- |
| Approximate WIT signature for `stream.new` | `func<T>() -> writable-stream-end<T>` |
| Approximate WIT signature for `future.new` | `func<T>() -> writable-future-end<T>` |
| Canonical ABI signature                    | `[] -> [writable-end:i32]`            |

The `stream.new` and `future.new` built-ins return the [writable end] of a new
`stream<T>` or `future<T>`. (See also [`canon_stream_new`] in the Canonical ABI
explainer for details.)

The types `readable-stream-end<T>` and `writable-stream-end<T>` are not WIT
types; they are the conceptual lower-level types that describe how the
canonical built-ins use the readable and writable ends of a `stream<T>`.
`writable-stream-end<T>`s are obtained from `stream.new`. A
`readable-stream-end<T>` is created by calling `stream.new` to create a fresh
"unpaired" `writable-stream<T>` and then lifting it as the `stream<T>`
parameter of an import call or the `stream<T>` result of an export call. This
lifted `stream<T>` value is then lowered by the receiving component into a
`readable-stream-end<T>` that is "paired" with the original
`writable-stream-end<T>`.

An analogous relationship exists among `readable-future-end<T>`,
`writable-future-end<T>`, and the WIT `future<T>`.

###### 🔀 `stream.read` and `stream.write`

| Synopsis                                     |                                                                             |
| -------------------------------------------- | --------------------------------------------------------------------------- |
| Approximate WIT signature for `stream.read`  | `func<T>(e: readable-stream-end<T>, b: writable-buffer<T>) -> read-status`  |
| Approximate WIT signature for `stream.write` | `func<T>(e: writable-stream-end<T>, b: readable-buffer<T>) -> write-status` |
| Canonical ABI signature                      | `[stream-end:i32 ptr:i32 num:i32] -> [i32]`                                 |

where `read-status` is defined in WIT as:
```wit
enum read-status {
    // The operation completed and read this many elements.
    complete(u32),

    // The operation did not complete immediately, so callers must wait for
    // the operation to complete by using `task.wait` or by returning to the
    // event loop.
    blocked,

    // The end of the stream has been reached.
    closed(option<error-context>),
}
```

and `write-status` is the same as `read-status` except without the optional
error on `closed`, so it is defined in WIT as:
```wit
enum write-status {
    // The operation completed and wrote this many elements.
    complete(u32),

    // The operation did not complete immediately, so callers must wait for
    // the operation to complete by using `task.wait` or by returning to the
    // event loop.
    blocked,

    // The reader is no longer reading data.
    closed,
}
```

The `stream.read` and `stream.write` built-ins take the matching [readable or
writable end] of a stream as the first parameter and a buffer for the `T`
values to be read from or written to. The return value is either the number of
elements (possibly zero) that have been eagerly read or written, a sentinel
indicating that the operation did not complete yet (`blocked`), or a sentinel
indicating that the stream is closed (`closed`). For reads, `closed` has an
optional error context describing the error that caused to the stream to close.

In the Canonical ABI, the buffer is passed as a pointer to a buffer in linear
memory and the size in elements of the buffer. (See [`canon_stream_read`] in
the Canonical ABI explainer for details.)

`read-status` and `write-status` are lowered in the Canonical ABI as:
 - The value `0xffff_ffff` represents `blocked`.
 - Otherwise, if the bit `0x8000_0000` is set, the value represents `closed`.
   For `read-status`, the remaining bits `0x7fff_ffff` contain the index of an
   `error-context` in the instance's `error-context` table.
 - Otherwise, the value represents `complete` and contains the number of
   element read or written.

(See [`pack_async_copy_result`] in the Canonical ABI explainer for details.)

###### 🔀 `future.read` and `future.write`

| Synopsis                                     |                                                                                |
| -------------------------------------------- | ------------------------------------------------------------------------------ |
| Approximate WIT signature for `future.read`  | `func<T>(e: readable-future-end<T>, b: writable-buffer<T; 1>) -> read-status`  |
| Approximate WIT signature for `future.write` | `func<T>(e: writable-future-end<T>, b: readable-buffer<T; 1>) -> write-status` |
| Canonical ABI signature                      | `[future-end:i32 ptr:i32] -> [i32]`                                            |

where `read-status` and `write-status` are defined as in
[`stream.read` and `stream.write`](#-streamread-and-streamwrite).

The `future.{read,write}` built-ins take the matching [readable or writable
end] of a future as the first parameter, and a buffer for a single `T` value to
read into or write from. The return value is either `complete` if the future
value was eagerly read or written, a sentinel indicating that the operation did
not complete yet (`blocked`), or a sentinel indicating that the future is
closed (`closed`).

The number of elements returned when the value is `complete` is at most `1`.

The `<T; 1>` in the buffer types indicates that these buffers may hold at most
one `T` element.

In the Canonical ABI, the buffer is passed as a pointer to a buffer in linear
memory. (See [`canon_future_read`] in the Canonical ABI explainer for details.)

###### 🔀 `stream.cancel-read`, `stream.cancel-write`, `future.cancel-read`, and `future.cancel-write`

| Synopsis                                            |                                                      |
| --------------------------------------------------- | ---------------------------------------------------- |
| Approximate WIT signature for `stream.cancel-read`  | `func<T>(e: readable-stream-end<T>) -> read-status`  |
| Approximate WIT signature for `stream.cancel-write` | `func<T>(e: writable-stream-end<T>) -> write-status` |
| Approximate WIT signature for `future.cancel-read`  | `func<T>(e: readable-future-end<T>) -> read-status`  |
| Approximate WIT signature for `future.cancel-write` | `func<T>(e: writable-future-end<T>) -> write-status` |
| Canonical ABI signature                             | `[e: i32] -> [i32]`                                  |

where `read-status` and `write-status` are defined as in
[`stream.read` and `stream.write`](#-streamread-and-streamwrite).

The `stream.cancel-read`, `stream.cancel-write`, `future.cancel-read`, and
`future.cancel-write` built-ins take the matching [readable or writable end] of
a stream or future that has an outstanding `blocked` read or write. If
cancellation finished eagerly, the return value is `complete`, and provides the
number of elements read or written into the given buffer (`0` or `1` for a
`future`). If cancellation blocks, the return value is `blocked` and the caller
must `task.wait`. If the stream or future is closed, the return value is
`closed`.

For `future.*`, the number of elements returned when the value is `complete`
is at most `1`.

In the Canonical ABI with the `callback` option, returning to the event loop is
equivalent to a `task.wait`, and a `{STREAM,FUTURE}_{READ,WRITE}` event will be
delivered to indicate the completion of the `read` or `write`. (See
[`canon_stream_cancel_read`] in the Canonical ABI explainer for details.)

###### 🔀 `stream.close-readable`, `stream.close-writable`, `future.close-readable`, and `future.close-writable`

| Synopsis                                              |                                                                  |
| ----------------------------------------------------- | ---------------------------------------------------------------- |
| Approximate WIT signature for `stream.close-readable` | `func<T>(e: readable-stream-end<T>, err: option<error-context>)` |
| Approximate WIT signature for `stream.close-writable` | `func<T>(e: writable-stream-end<T>, err: option<error-context>)` |
| Approximate WIT signature for `future.close-readable` | `func<T>(e: readable-future-end<T>, err: option<error-context>)` |
| Approximate WIT signature for `future.close-writable` | `func<T>(e: writable-future-end<T>, err: option<error-context>)` |
| Canonical ABI signature                               | `[end:i32 err:i32] -> []`                                        |

The `{stream,future}.close-{readable,writable}` built-ins remove the indicated
[stream or future] from the current component instance's table of [waitables],
trapping if the stream or future has a mismatched direction or type or are in
the middle of a `read` or `write`.

In the Canonical ABI, an `err` value of `0` represents `none`, and a non-zero
value represents `some` of the index of an `error-context` in the instance's
table. (See also [the `close` built-ins] in the Canonical ABI explainer.)

##### 🔀 Error Context built-ins

###### `error-context.new`

| Synopsis                         |                                          |
| -------------------------------- | ---------------------------------------- |
| Approximate WIT signature        | `func(message: string) -> error-context` |
| Canonical ABI signature          | `[ptr:i32 len:i32] -> [i32]`             |

The `error-context.new` built-in returns a new `error-context` value. The given
string is non-deterministically transformed to produce the `error-context`'s
internal [debug message](#error-context-type).

In the Canonical ABI, the returned value is an index into a
per-component-instance table. (See also [`canon_error_context_new`] in the
Canonical ABI explainer.)

###### `error-context.debug-message`

| Synopsis                         |                                         |
| -------------------------------- | --------------------------------------- |
| Approximate WIT signature        | `func(errctx: error-context) -> string` |
| Canonical ABI signature          | `[errctxi:i32 ptr:i32] -> []`           |

The `error-context.debug-message` built-in returns the
[debug message](#error-context-type) of the given `error-context`.

In the Canonical ABI, it writes the debug message into `ptr` as an 8-byte
(`ptr`, `length`) pair, according to the Canonical ABI for `string`, given the
`<canonopt>*` immediates. (See also [`canon_error_context_debug_message`] in
the Canonical ABI explainer.)

###### `error-context.drop`

| Synopsis                         |                               |
| -------------------------------- | ----------------------------- |
| Approximate WIT signature        | `func(errctx: error-context)` |
| Canonical ABI signature          | `[errctxi:i32] -> []`         |

The `error-context.drop` built-in drops the given `error-context` value from
the component instance.

In the Canonical ABI, `errctxi` is an index into a per-component-instance
table. (See also [`canon_error_context_drop`] in the Canonical ABI explainer.)

##### 🧵 Threading built-ins

The [shared-everything-threads] proposal adds component model built-ins for
thread management. These are specified as built-ins and not core WebAssembly
instructions because browsers expect this functionality to come from existing
Web/JS APIs.

###### 🧵 `thread.spawn_ref`

| Synopsis                   |                                                            |
| -------------------------- | ---------------------------------------------------------- |
| Approximate WIT signature  | `func<FuncT>(f: FuncT, c: FuncT.params[0]) -> bool`        |
| Canonical ABI signature    | `[f:(ref null (shared (func (param i32))) c:i32] -> [i32]` |

The `thread.spawn_ref` built-in spawns a new thread by invoking the shared
function `f` while passing `c` to it, returning whether a thread was
successfully spawned. While it's designed to allow different types in the
future, the type of `c` is currently hard-coded to always be `i32`.

(See also [`canon_thread_spawn_ref`] in the Canonical ABI explainer.)


###### 🧵 `thread.spawn_indirect`

| Synopsis                   |                                                   |
| -------------------------- | ------------------------------------------------- |
| Approximate WIT signature  | `func<FuncT>(i: u32, c: FuncT.params[0]) -> bool` |
| Canonical ABI signature    | `[i:i32 c:i32] -> [i32]`                          |

The `thread.spawn_indirect` built-in spawns a new thread by retrieving the
shared function `f` from a table using index `i` and traps if the type of `f` is
not equal to `FuncT` (much like the `call_indirect` core instruction). Once `f`
is retrieved, this built-in operates like `thread.spawn_ref` above, including
the limitations on `f`'s parameters.

(See also [`canon_thread_spawn_indirect`] in the Canonical ABI explainer.)

###### 🧵 `thread.available_parallelism`

| Synopsis                   |                 |
| -------------------------- | --------------- |
| Approximate WIT signature  | `func() -> u32` |
| Canonical ABI signature    | `[] -> [i32]`   |

The `thread.available_parallelism` built-in returns the number of threads that
can be expected to execute in parallel.

The concept of "available parallelism" corresponds is sometimes referred to
as "hardware concurrency", such as in [`navigator.hardwareConcurrency`] in
JavaScript.

(See also [`canon_thread_available_parallelism`] in the Canonical ABI
explainer.)


## References

[ABI]: https://en.wikipedia.org/wiki/Application_binary_interface
[adapter functions]: FutureFeatures.md#custom-abis-via-adapter-functions
[Canonical ABI explainer]: CanonicalABI.md
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
