# Built-in Types

As mentioned previously the intention of `wit` is to allow defining types
corresponding to the interface types specification. Many of the top-level items
above are introducing new named types but "anonymous" types are also supported,
such as built-ins. For example:

```wit
type number = u32;
type fallible-function-result = result<u32, string>;
type headers = list<string>;
```

Specifically the following types are available:

```ebnf
ty ::= 'u8' | 'u16' | 'u32' | 'u64'
     | 's8' | 's16' | 's32' | 's64'
     | 'f32' | 'f64'
     | 'char'
     | 'bool'
     | 'string'
     | tuple
     | list
     | option
     | result
     | handle
     | future
     | stream
     | id

tuple ::= 'tuple' '<' tuple-list '>'
tuple-list ::= ty
             | ty ',' tuple-list?

list ::= 'list' '<' ty '>'
       | 'list' '<' ty ',' uint '>' 🔧

uint ::= [1-9][0-9]*

option ::= 'option' '<' ty '>'

result ::= 'result' '<' ty ',' ty '>'
         | 'result' '<' '_' ',' ty '>'
         | 'result' '<' ty '>'
         | 'result'

future ::= 'future' '<' ty '>'
         | 'future'

stream ::= 'stream' '<' ty '>'
         | 'stream'
```

The `tuple` type is semantically equivalent to a `record` with numerical fields,
but it frequently can have language-specific meaning so it's provided as a
first-class type.

🔧 A `list` with a fixed length provides the low-level memory representation of a
homogeneous `tuple` of the same length, but with the dynamic indexing of a
list. E.g., the following two functions have the same low-level (Core
WebAssembly) representation, but will naturally produce different source-level
bindings:

```wit
get-ipv4-address1: func() -> list<u8, 4>;
get-ipv4-address2: func() -> tuple<u8, u8, u8, u8>;
```

The `option` and `result` types are semantically equivalent to the variants:

```wit
variant option {
    none,
    some(ty),
}

variant result {
    ok(ok-ty),
    err(err-ty),
}
```

These types are so frequently used and frequently have language-specific
meanings though so they're also provided as first-class types.

The `future` and `stream` types are described as part of the [async
explainer](Async.md#streams-and-futures).

Finally the last case of a `ty` is simply an `id` which is intended to refer to
another type or resource defined in the document. Note that definitions can come
through a `use` statement or they can be defined locally.
