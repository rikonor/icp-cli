# Package Names

All WIT packages are assigned a _package name_. Package names look like
`foo:bar@1.0.0` and have three fields:

- A _namespace field_, for example `foo` in `foo:bar`. This namespace is
  intended to disambiguate between registries, top-level organizations, etc.
  For example WASI interfaces use the `wasi` namespace.

- A _package field_, for example `clocks` in `wasi:clocks`. A "package" groups
  together a set of interfaces and worlds that would otherwise be named with a
  common prefix.

- An optional _version field_, specified as [full semver](https://semver.org/).

🪺 With "nested namespaces and packages", package names are generalized to look
like `foo:bar:baz/quux`, where `bar` is a nested namespace of `foo` and `quux`
is a nested package of `baz`. See the [package declaration] section for more
details.

Package names are specified at the top of a WIT file via a `package`
declaration:

```wit
package wasi:clocks;
```

or

```wit
package wasi:clocks@1.2.0;
```

WIT packages can be defined in a collection of files. At least one of these
files must specify a package name. Multiple files can specify the `package`,
though they must all agree on what the package name is.

Additionally, many packages can be declared consecutively in one or more files, if the following nested package notation is used:

```wit
package local:a {
    interface foo {}
}

package local:b {
    interface bar {}
}
```

It is worth noting that defining nested packages does not remove the need for the "root" package declaration above. These nested package definitions simply provide the contents of other packages inline so that they don't have to be otherwise resolved via the filesystem or a registry.

Package names are used to generate the [names of imports and exports]
in the Component Model's representation of [`interface`s][interfaces] and
[`world`s][worlds] as described [below](#package-format).

[names of imports and exports]: ../explainer/explainer-component-definitions.md#import-and-export-definitions
[package declaration]: #package-declaration
[interfaces]: interfaces.md
[worlds]: worlds.md
