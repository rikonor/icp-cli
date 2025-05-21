# WIT Identifiers

Identifiers in WIT can be defined with two different forms. The first is the
[kebab-case] [`label`](../explainer/explainer-component-definitions.md#import-and-export-names) production in the
Component Model text format.

```wit
foo: func(bar: u32);

red-green-blue: func(r: u32, g: u32, b: u32);

resource XML { ... }
parse-XML-document: func(s: string) -> XML;
```

This form can't lexically represent WIT [keywords](#keywords), so the second form is the
same syntax with the same restrictions as the first, but prefixed with '%':

```wit
%foo: func(%bar: u32);

%red-green-blue: func(%r: u32, %g: u32, %b: u32);

// This form also supports identifiers that would otherwise be keywords.
%variant: func(%enum: s32);
```

[kebab-case]: https://en.wikipedia.org/wiki/Letter_case#Kebab_case
[keywords]: lexical-structure.md#keywords
