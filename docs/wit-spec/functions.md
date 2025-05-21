# WIT Functions

Functions are defined in an [`interface`][interfaces] or are listed as an
`import` or `export` from a [`world`][worlds]. Parameters to a function must all
be named and have case-insensitively unique names:

```wit
package local:demo;

interface foo {
    a1: func();
    a2: func(x: u32);
    a3: func(y: u64, z: f32);
}
```

Functions can optionally return a type:

```wit
package local:demo;

interface foo {
    a1: func() -> u32;
    a2: func() -> string;
}
```

Multiple return values can be achieved via `tuple` or `record` type:

```wit
package local:demo;

interface foo {
    record r {
      a: u32,
      b: f32
    }

    a1: func() -> r;
    a2: func() -> tuple<u32, f32>;
}
```

[interfaces]: interfaces.md
[worlds]: worlds.md
