# Filesystem structure

WIT supports multiple `package`s and the ability to define a single package
across many files, and this section is intended to set a number of conventions
for WIT-processing tooling to conform to.

This won't go into the specific details of any one particular tool and you
should consult tooling-specific documentation for more detailed information
about exactly how to configure a WIT parser. This will, however use the Rust
guest `wit-bindgen` crate as an example to have a concrete example to link to,
but this is intended to be translatable to other examples and bindings
generators as well.

## Specifying a "Root Package"

To start out when processing WIT a package needs to be conceptually considered
the "root package". This is used down below in `world` selection and the
conventional processing of WIT is intended to currently generally have a package
as the "default" for lookups. A root package is specified via a path on the
filesystem to either a file or directory. Lookup of dependencies and of this
package happen differently depending if it's a file or directory.

### Root Package: A File

When the root package is a single file then it means that file contains all WIT
that is going to be parsed. No further file discovery on the filesystem will
happen and after the file is read then no more filesystem interaction will be
happening.

```rust
wit_bindgen::generate!("./my.wit");
```

To be a valid WIT file the file being parsed must have a leading `package ...;`
statement meaning that it's now the "root package". Dependencies of this package
must be specified inline with `package ... { ... }` blocks when using this
format.

Some tooling may support the ability to load multiple "roots" which means that
the final root is used for `world` selection and the other roots are used to
load dependencies. This can be used when you don't necessarily have full control
over filesystem structure and need to load dependencies from a possibly
non-standard location.

```rust
// here `deps.wit` will be available when parsing `my.wit` for dependency
// resolution.
wit_bindgen::generate!({
    path: ["./deps.wit", "./my.wit"],
});
```

Note that specifying a file is not the only option for organizing WIT bindings.
Below can be a more maintainable strategy with WIT files separate from each
other. A single file can be useful when tooling manages WIT for you, but
handwritten WIT may often prefer to use a directory.

### Root Package: A Directory

When the root package is a directory then it means the filesystem structure of
that directory will be traversed to look for WIT to load. A directory not only
supports splitting a single package across multiple files on the filesystem but
it also enables having all dependencies located within the directory as well.

```rust
wit_bindgen::generate!("./wit");
```

This example will parse the directory `./wit` and look for WIT files. The
parsing process first looks at all `*.wit` files inside the directory itself.
This collection of `*.wit` files will be combined together to form the "root
package". No other files will be considered for the "root package". For example
though you could have this filesystem structure.

```rust
wit/
    types.wit
    world.wit
    my-interface.wit
```

Here `types.wit`, `world.wit`, and `my-interface.wit` would all be parsed
together as a single package.

Dependencies in the directory format of the filesystem are specified in a `deps`
folder within the root folder. Above for example dependencies would be specified
in `wit/deps`. Dependencies are specified in a flat format where each dependency
may itself be a file or a directory, but directories do not have recursive
`deps` folders. The name of files/folders used for organization within a
directory are not used during parsing and are purely meant for human-read
organization.

For example we can extend our above `wit/` folder like so:

```rust
wit/
    types.wit
    world.wit
    my-interface.wit

    deps/
        my-dependency.wit
        wasi:clocks/
            types.wit
            world.wit
        wasi:clocks@0.3.0-pre/
            types.wit
            world.wit
```

The name `my-dependency` in `my-dependency.wit`, as well as `wasi:clocks` in
`wasi:clocks/`, is arbitrary. This distinguishes one dependency from another but
is only used for uniqueness on the filesystem.

All dependencies in `deps` will be loaded and processed in topological order.
The `my-dependency.wit` file may, for example, depend on `wasi:clocks/`.
Additionally `my-dependency.wit` may have its own inline `package .. { ... }`
blocks too which define packages available for dependency resolution. Any
package which is duplicated across dependencies must have the same contents.

## Specifying a World

The primary unit of bindings generation for WIT tooling is a `world` which means
that various phases of the process must take a `world` input. For example when
generating guest bindings within a language you'll be specifying a `world` as
the unit of bindings generation. WIT tooling should follow these conventions for
selecting a world:

- Inputs to world selection are a "root package" (what was parsed from the WIT
  path specified) as well as an optional world string.
- If the world string is not present, then the root package must have a single
  world in it. If so that world is used for bindings generation.
- If the world string is a WIT identifier, then it specifies the name of a world
  in the root package to use for bindings generation.
- If the world string is a WIT path, such as `a:b/c`, then that is a
  fully-qualified path which can be used to select a world in the dependencies
  for bindings generation as well.

If the above heuristics all fail then bindings generation fails and a different
combination of arguments must be passed to select a world for bindings
generation.
