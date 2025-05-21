# Feature Gates

Various WIT items can be "gated", to reflect the fact that the item is part of
an unstable feature, that the item was added as part of a minor version
update and shouldn't be used when targeting an earlier minor version, or that a
feature has been deprecated and should no longer be used.

For example, the following interface has 4 items, 3 of which are gated:

```wit
interface foo {
    a: func();

    @since(version = 0.2.1)
    b: func();

    @since(version = 0.2.2)
    c: func();

    @unstable(feature = fancier-foo)
    d: func();

    @since(version = 0.2.0)
    @deprecated(version = 0.2.2)
    e: func();
}
```

The `@since` gate indicates that `b` and `c` were added as part of the `0.2.1`
and `0.2.2` releases, resp. Thus, when building a component targeting, e.g.,
`0.2.1`, `b` can be used, but `c` cannot. An important expectation set by the
`@since` gate is that, once applied to an item, the item is not modified
incompatibly going forward (according to general semantic versioning rules).

In contrast, the `@unstable` gate on `d` indicates that `d` is part of the
`fancier-foo` feature that is still under active development and thus `d` may
change type or be removed at any time. An important expectation set by the
`@unstable` gate is that toolchains will not expose `@unstable` features by
default unless explicitly opted-into by the developer.

Finally, the `@deprecated` gate on `e` indicates that `e` should no longer be
used starting version `0.2.2`. Both toolchains and host runtimes may warn users
if they detect an `@deprecated` API is being used. A `@deprecated` gate is
required to always be paired up with either a `@since` or `@deprecated` gate.

Together, these gates support a development flow in which new features start
with an `@unstable` gate while the details are still being hashed out. Then,
once the feature is stable (and, in a WASI context, voted upon), the
`@unstable` gate is switched to a `@since` gate.

#### Feature gate syntax

The grammar that governs feature gate syntax is:

```wit
gate ::= gate-item*
gate-item ::= unstable-gate
            | since-gate
            | deprecated-gate

unstable-gate ::= '@unstable' '(' feature-field ')'
since-gate ::= '@since' '(' version-field ')'
deprecated-gate ::= '@deprecated' '(' version-field ')'

feature-field ::= 'feature' '=' id
version-field ::= 'version' '=' <valid semver>
```

#### Rules for feature gate usage

As part of WIT validation, any item that refers to another gated item must also
be compatibly gated. For example, this is an error:

```wit
interface i {
    @since(version = 1.0.1)
    type t1 = u32;

    type t2 = t1; // error
}
```

Additionally, if an item is _contained_ by a gated item, it must also be
compatibly gated. For example, this is an error:

```wit
@since(version = 1.0.2)
interface i {
    foo: func();  // error: no gate

    @since(version = 1.0.1)
    bar: func();  // also error: weaker gate
}
```

The following rules apply to the use of feature gates:

- Either `@since` _or_ `@unstable` should be used, but not both (exclusive or).
- If a package contains a feature gate, it's version must be specified (i.e. `namespace:package@x.y.z`)

#### Scenario: Stabilization of a new feature

This section lays out the basic flow and expected usage of feature gate machinery
when stabilizing new features and deprecating old ones.

Assume the following WIT package as the initial interface:

```wit
package examples:fgates-calc@0.1.0;

@since(version = 0.1.0)
interface calc {
    @since(version = 0.1.0)
    variant calc-error {
      integer-overflow,
      integer-underflow,
      unexpected,
    }

    @since(version = 0.1.0)
    add: func(x: s32, y: s32) -> result<s32, calc-error>;
}
```

**First, add new items under an `@unstable` annotation with a `feature` specified:**

```wit
package examples:fgates-calc@0.1.1;

@since(version = 0.1.0)
interface calc {
    @since(version = 0.1.0)
    variant calc-error {
      integer-overflow,
      integer-underflow,
      unexpected,
    }

    @since(version = 0.1.0)
    add: func(x: s32, y: s32) -> result<s32, calc-error>;

    /// By convention, feature flags should be prefixed with package name to reduce chance of collisions
    ///
    /// see: https://github.com/WebAssembly/WASI/blob/main/Contributing.md#filing-changes-to-existing-phase-3-proposals
    @unstable(feature = fgates-calc-minus)
    sub: func(x: s32, y: s32) -> result<s32, calc-error>;
}
```

At this point, consumers of the WIT can enable feature `fgates-calc-minus` through their relevant tooling and get access to the `sub` function.

Note that, at least until subtyping is relaxed in the Component Model, if we had to _add_ a new case to `calc-error`, this would be a _breaking change_ and require either a new major version or adding a second, distinct `variant` definition used by new functions.

**Second, when the feature is ready to be stabilized, switch to a `@since` annotation:**

```wit
package examples:fgates-calc@0.1.2;

@since(version = 0.1.0)
interface calc {
    @since(version = 0.1.0)
    variant calc-error {
      integer-overflow,
      integer-underflow,
      unexpected,
    }

    @since(version = 0.1.0)
    add: func(x: s32, y: s32) -> result<s32, calc-error>;

    @since(version = 0.1.2)
    sub: func(x: s32, y: s32) -> result<s32, calc-error>;
}
```

#### Scenario: Deprecation of an existing stable feature

This section lays out the basic flow and expected usage of feature gate machinery when stabilizing a new feature.

Assume the following WIT package as the initial interface:

```wit
package examples:fgates-deprecation@0.1.1;

@since(version = 0.1.0)
interface calc {
    @since(version = 0.1.0)
    variant calc-error {
      integer-overflow,
      integer-underflow,
      unexpected,
    }

    @since(version = 0.1.0)
    add-one: func(x: s32) -> result<s32, calc-error>;

    @since(version = 0.1.1)
    add: func(x: s32, y: s32) -> result<s32, calc-error>;
}
```

**First: Add the `@deprecated` annotation to the relevant item in a new version**

```wit
package examples:fgates-deprecation@0.1.2;

@since(version = 0.1.0)
interface calc {
    @since(version = 0.1.0)
    variant calc-error {
      integer-overflow,
      integer-underflow,
      unexpected,
    }

    @deprecated(version = 0.1.2)
    add-one: func(x: s32) -> result<s32, calc-error>;

    @since(version = 0.1.1)
    add: func(x: s32, y: s32) -> result<s32, calc-error>;
}
```

At this point, tooling consuming this WIT will be able to appropriately alert users to the now-deprecated `add-one` function.

**Second: completely remove the deprecated item in some future SemVer-compliant major version**

```wit
package examples:fgates-deprecation@0.2.0;

@since(version = 0.1.0)
interface calc {
    @since(version = 0.1.0)
    variant calc-error {
      integer-overflow,
      integer-underflow,
      unexpected,
    }

    @since(version = 0.1.1)
    add: func(x: s32, y: s32) -> result<s32, calc-error>;
}
```

In this new "major" version (this is considered a major version under SemVer 0.X rules) -- the `add-one` function can be fully removed.
