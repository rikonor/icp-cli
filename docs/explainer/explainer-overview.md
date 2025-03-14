# Component Model Overview

This document provides an introduction to the WebAssembly Component Model, including gated features and high-level concepts.

## Introduction

The Component Model is an extension to WebAssembly that enables the creation of reusable, composable components with well-defined interfaces. It addresses the need for better modularity, interoperability, and language-agnostic interfaces in WebAssembly.

Components are designed to be:

- **Shared-nothing**: Components do not share memory or other state by default
- **Interface-driven**: Components define clear interfaces for imports and exports
- **Language-agnostic**: Components can be written in any language that compiles to WebAssembly
- **Composable**: Components can be combined to create larger applications

## Gated Features

The Component Model includes several features that may be gated behind feature flags or not yet fully implemented:

- 🪙: Value imports/exports and component-level start function
- 🪺: Nested namespaces and packages in import/export names
- 🔀: Async functionality
  - 🚝: Marking some builtins as `async`
  - 🚟: Using `async` with `canon lift` without `callback` (stackful lift)
- 🧵: Threading built-ins
- 🔧: Fixed-length lists

## Key Concepts

### Components vs. Modules

- **Modules** are the basic unit of WebAssembly code, with a flat function table and shared linear memory
- **Components** are higher-level constructs that encapsulate one or more modules and provide interface-based composition

### Interface Types

The Component Model introduces high-level interface types that allow for language-agnostic communication:

- Primitive types: `bool`, `s8`, `u8`, `s16`, `u16`, `s32`, `u32`, `s64`, `u64`, `f32`, `f64`, `char`, `string`
- Container types: `record`, `variant`, `list`, `tuple`
- Resource types: `resource`, `own`, `borrow`
- Asynchronous types: `stream`, `future`

### Canonical ABI

The Canonical ABI defines how values are passed between components, providing a standard way to:

- Lift values from core WebAssembly to component-level types
- Lower values from component-level types to core WebAssembly

This document is part of a series that explains the Component Model in detail. For more specific information, refer to the other documents in this directory.
