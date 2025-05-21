# Lexical structure

The `wit` format is a curly-braced-based format where whitespace is optional (but
recommended). A `wit` document is parsed as a unicode string, and when stored in
a file is expected to be encoded as utf-8.

Additionally, wit files must not contain any bidirectional override scalar
values, control codes other than newline, carriage return, and horizontal tab,
or codepoints that Unicode officially deprecates or strongly discourages.

The current structure of tokens are:

```ebnf
token ::= whitespace
        | operator
        | keyword
        | integer
        | identifier
```

Whitespace and comments are ignored when parsing structures defined elsewhere
here.

### Whitespace

A `whitespace` token in `wit` is a space, a newline, a carriage return, a
tab character, or a comment:

```ebnf
whitespace ::= ' ' | '\n' | '\r' | '\t' | comment
```

### Comments

A `comment` token in `wit` is either a line comment preceded with `//` which
ends at the next newline (`\n`) character or it's a block comment which starts
with `/*` and ends with `*/`. Note that block comments are allowed to be nested
and their delimiters must be balanced

```ebnf
comment ::= '//' character-that-isnt-a-newline*
          | '/*' any-unicode-character* '*/'
```

### Operators

There are some common operators in the lexical structure of `wit` used for
various constructs. Note that delimiters such as `{` and `(` must all be
balanced.

```ebnf
operator ::= '=' | ',' | ':' | ';' | '(' | ')' | '{' | '}' | '<' | '>' | '*' | '->' | '/' | '.' | '@'
```

### Keywords

Certain identifiers are reserved for use in WIT documents and cannot be used
bare as an identifier. These are used to help parse the format, and the list of
keywords is still in flux at this time but the current set is:

```ebnf
keyword ::= 'as'
          | 'async'
          | 'bool'
          | 'borrow'
          | 'char'
          | 'constructor'
          | 'enum'
          | 'export'
          | 'f32'
          | 'f64'
          | 'flags'
          | 'from'
          | 'func'
          | 'future'
          | 'import'
          | 'include'
          | 'interface'
          | 'list'
          | 'option'
          | 'own'
          | 'package'
          | 'record'
          | 'resource'
          | 'result'
          | 's16'
          | 's32'
          | 's64'
          | 's8'
          | 'static'
          | 'stream'
          | 'string'
          | 'tuple'
          | 'type'
          | 'u16'
          | 'u32'
          | 'u64'
          | 'u8'
          | 'use'
          | 'variant'
          | 'with'
          | 'world'
```

### Integers

Integers are currently only used for package versions and are a contiguous
sequence of digits:

```ebnf
integer ::= [0-9]+
```

## Top-level items

A `wit` document is a sequence of items specified at the top level. These items
come one after another and it's recommended to separate them with newlines for
readability but this isn't required.

Concretely, the structure of a `wit` file is:

```ebnf
wit-file ::= (package-decl ';')? (package-items | nested-package-definition)*

nested-package-definition ::= package-decl '{' package-items* '}'

package-items ::= toplevel-use-item | interface-item | world-item
```

Essentially, these top level items are [worlds], [interfaces], [use statements][use] and other package definitions.

[worlds]: worlds.md
[interfaces]: interfaces.md
[use statements]: packages-and-use.md
