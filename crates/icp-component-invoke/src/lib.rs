//! # ICP Component Invoke
//!
//! This crate provides the shared `Val` enum used for representing dynamic values
//! when invoking WebAssembly component exports directly within the ICP CLI and its
//! extensions.
//!
//! ## Purpose
//!
//! The primary goal of this crate is to centralize the definition of `Val`, which
//! mirrors `wasmtime::component::Val` but is designed to be usable in environments
//! that may not have a direct dependency on `wasmtime` or its `std` requirements
//! (e.g., Wasm components/extensions themselves).
//!
//! ## Features
//!
//! - `wasmtime-conversions`: Enables conversion logic to and from
//!   `wasmtime::component::Val`. This feature is intended for use by the host
//!   environment (e.g., `icp-cli`) that interacts directly with `wasmtime`.
//!   Extensions typically should *not* enable this feature.
use serde::{Deserialize, Serialize};

#[cfg(feature = "wasmtime-conversions")]
use wasmtime::component::Val as WasmVal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Val {
    // Primitive types
    Bool(bool),

    // Signed integers
    S8(i8),
    S16(i16),
    S32(i32),
    S64(i64),

    // Unsigned integers
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),

    // Floating point numbers
    Float32(f32),
    Float64(f64),

    // Text
    Char(char),
    String(String),

    // Containers
    Enum(String), // Name of the enum case
    List(Vec<Val>),
    Option(Option<Box<Val>>),
    Record(Vec<(String, Val)>), // List of (name, value) pairs
    Result(Result<Option<Box<Val>>, Option<Box<Val>>>),
    Tuple(Vec<Val>),
    Variant(String, Option<Box<Val>>), // (case_name, optional_payload)

    // Other
    Flags(Vec<String>), // List of set flag names
                        // TODO: Figure out how to represent Resource type if needed directly in this enum
                        // Resource(u32), // Or some other way to represent a handle/ID
}

#[cfg(feature = "wasmtime-conversions")]
impl From<Val> for WasmVal {
    fn from(value: Val) -> Self {
        match value {
            Val::Bool(v) => WasmVal::Bool(v),
            Val::S8(v) => WasmVal::S8(v),
            Val::S16(v) => WasmVal::S16(v),
            Val::S32(v) => WasmVal::S32(v),
            Val::S64(v) => WasmVal::S64(v),
            Val::U8(v) => WasmVal::U8(v),
            Val::U16(v) => WasmVal::U16(v),
            Val::U32(v) => WasmVal::U32(v),
            Val::U64(v) => WasmVal::U64(v),
            Val::Float32(v) => WasmVal::Float32(v),
            Val::Float64(v) => WasmVal::Float64(v),
            Val::Char(v) => WasmVal::Char(v),
            Val::String(v) => WasmVal::String(v),
            Val::Enum(v) => WasmVal::Enum(v),
            Val::List(vals) => WasmVal::List(vals.into_iter().map(WasmVal::from).collect()),
            Val::Option(val) => {
                if let Some(val) = val {
                    WasmVal::Option(Some(Box::new(WasmVal::from(*val))))
                } else {
                    WasmVal::Option(None)
                }
            }
            Val::Record(items) => WasmVal::Record(
                items
                    .into_iter()
                    .map(|(k, v)| (k, WasmVal::from(v)))
                    .collect(),
            ),
            Val::Result(val) => match val {
                Ok(v) => WasmVal::Result(Ok(v.map(|v_box| Box::new(WasmVal::from(*v_box))))),
                Err(e) => WasmVal::Result(Err(e.map(|e_box| Box::new(WasmVal::from(*e_box))))),
            },
            Val::Tuple(vals) => WasmVal::Tuple(vals.into_iter().map(WasmVal::from).collect()),
            Val::Variant(k, val) => {
                if let Some(val) = val {
                    WasmVal::Variant(k, Some(Box::new(WasmVal::from(*val))))
                } else {
                    WasmVal::Variant(k, None)
                }
            }
            Val::Flags(items) => WasmVal::Flags(items),
        }
    }
}

#[cfg(feature = "wasmtime-conversions")]
impl From<WasmVal> for Val {
    fn from(value: WasmVal) -> Self {
        match value {
            WasmVal::Bool(v) => Val::Bool(v),
            WasmVal::S8(v) => Val::S8(v),
            WasmVal::S16(v) => Val::S16(v),
            WasmVal::S32(v) => Val::S32(v),
            WasmVal::S64(v) => Val::S64(v),
            WasmVal::U8(v) => Val::U8(v),
            WasmVal::U16(v) => Val::U16(v),
            WasmVal::U32(v) => Val::U32(v),
            WasmVal::U64(v) => Val::U64(v),
            WasmVal::Float32(v) => Val::Float32(v),
            WasmVal::Float64(v) => Val::Float64(v),
            WasmVal::Char(v) => Val::Char(v),
            WasmVal::String(v) => Val::String(v),
            WasmVal::Enum(v) => Val::Enum(v),
            WasmVal::List(vals) => Val::List(vals.into_iter().map(Val::from).collect()),
            WasmVal::Option(val) => {
                if let Some(val) = val {
                    Val::Option(Some(Box::new(Val::from(*val))))
                } else {
                    Val::Option(None)
                }
            }
            WasmVal::Record(items) => {
                Val::Record(items.into_iter().map(|(k, v)| (k, Val::from(v))).collect())
            }
            WasmVal::Result(val) => match val {
                Ok(v) => Val::Result(Ok(v.map(|v_box| Box::new(Val::from(*v_box))))),
                Err(e) => Val::Result(Err(e.map(|e_box| Box::new(Val::from(*e_box))))),
            },
            WasmVal::Tuple(vals) => Val::Tuple(vals.into_iter().map(Val::from).collect()),
            WasmVal::Variant(k, val) => {
                if let Some(val) = val {
                    Val::Variant(k, Some(Box::new(Val::from(*val))))
                } else {
                    Val::Variant(k, None)
                }
            }
            WasmVal::Flags(items) => Val::Flags(items),
            WasmVal::Resource(_) => unimplemented!(
                "Resource type conversion from WasmVal to Val is not yet implemented"
            ),
        }
    }
}
