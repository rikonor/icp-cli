# WIT Worlds

WIT packages can contain `world` definitions at the top-level in addition to
[`interface`][interfaces] definitions. A world is a complete description of
both imports and exports of a component. A world can be thought of as an
equivalent of a `component` type in the component model. For example this
world:

```wit
package local:demo;

world my-world {
    import host: interface {
      log: func(param: string);
    }

    export run: func();
}
```

can be thought of as this component type:

```wat
(type $my-world (component
  (import "host" (instance
    (export "log" (func (param "param" string)))
  ))
  (export "run" (func))
))
```

Worlds describe a concrete component and are the basis of bindings generation. A
guest language will use a `world` to determine what functions are imported, what
they're named, and what functions are exported, in addition to their names.

Worlds can contain any number of imports and exports, and can be either a
function or an interface.

```wit
package local:demo;

world command {
    import wasi:filesystem/filesystem;
    import wasi:random/random;
    import wasi:clocks/monotonic-clock;
    // ...

    export main: func(args: list<string>);
}
```

More information about the `wasi:random/random` syntax is available below in the
description of [`use`][use].

An imported or exported interface corresponds to an imported or exported
instance in the component model. Functions are equivalent to bare component
functions. Additionally interfaces can be defined inline with an explicit
[plain name] that avoids the need to have an out-of-line definition.

```wit
package local:demo;

interface out-of-line {
    the-function: func();
}

world your-world {
    import out-of-line;
    // ... is roughly equivalent to ...
    import out-of-line: interface {
      the-function: func();
    }
}
```

The plain name of an `import` or `export` statement is used as the plain name
of the final component `import` or `export` definition.

In the component model imports to a component either use a plain or interface
name, and in WIT this is reflected in the syntax:

```wit
package local:demo;

interface my-interface {
    // ..
}

world command {
    // generates an import of the name `local:demo/my-interface`
    import my-interface;

    // generates an import of the name `wasi:filesystem/types`
    import wasi:filesystem/types;

    // generates an import of the plain name `foo`
    import foo: func();

    // generates an import of the plain name `bar`
    import bar: interface {
      // ...
    }
}
```

Each name must be case-insensitively unique in the scope in which it is
declared. In the case of worlds, all imported names are in the same scope,
but separate from all the export names, and thus the same name can _not_ be
imported twice, but _can_ be both imported and exported.

[Plain Name]: ../explainer/explainer-component-definitions.md#import-and-export-definitions
[interfaces]: interfaces.md
[use]: packages-and-use.md

### Union of Worlds with `include`

A World can be created by taking the union of two or more worlds. This operation allows world builders to form larger worlds from smaller worlds.

Below is a simple example of a world that includes two other worlds.

```wit
package local:demo;

// definitions of a, b, c, foo, bar, baz are omitted

world my-world-a {
    import a;
    import b;
    export c;
}

world my-world-b {
    import foo;
    import bar;
    export baz;
}

world union-my-world {
    include my-world-a;
    include my-world-b;
}
```

The `include` statement is used to include the imports and exports of another World to the current World. It says that the new World should be able to run all components that target the included worlds and more.

The `union-my-world` World defined above is equivalent to the following World:

```wit
world union-my-world {
    import a;
    import b;
    export c;
    import foo;
    import bar;
    export baz;
}
```

### De-duplication of interfaces

If two worlds share an imported or exported [interface name], then the union of
the two worlds will only contain one copy of that imported or exported name.
For example, the following two worlds `union-my-world-a` and `union-my-world-b`
are equivalent:

```wit
package local:demo;

world my-world-a {
    import a1;
    import b1;
}

world my-world-b {
    import a1;
    import b1;
}

world union-my-world-a {
    include my-world-a;
    include my-world-b;
}

world union-my-world-b {
    import a1;
    import b1;
}
```

### Name Conflicts and `with`

When two or more included Worlds have the same name for an import or export
with a _plain_ name, automatic de-duplication cannot be used (because the two
same-named imports/exports might have different meanings in the different
worlds) and thus the conflict has to be resolved manually using the `with`
keyword.

The following example shows how to resolve name conflicts where
`union-my-world-a` and `union-my-world-b` are equivalent:

```wit
package local:demo;

world world-one { import a: func(); }
world world-two { import a: func(); }

world union-my-world-a {
    include world-one;
    include world-two with { a as b }
}

world union-my-world-b {
    import a: func();
    import b: func();
}
```

`with` cannot be used to rename interface names, however, so the following
world would be invalid:

```wit
package local:demo;

interface a {
    foo: func();
}

world world-using-a {
    import a;
}

world invalid-union-world {
    include my-using-a with { a as b }  // invalid: 'a', which is short for 'local:demo/a', is an interface name
}

```

### A Note on Subtyping

In the future, when `optional` export is supported, the world author may explicitly mark exports as optional to make a component targeting an included World a subtype of the union World.

For now, we are not following the subtyping rules for the `include` statement. That is, the `include` statement does not imply any subtyping relationship between the included worlds and the union world.

[interface name]: ../explainer/explainer-component-definitions.md#import-and-export-definitions
