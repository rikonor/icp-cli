//! WebAssembly text format (WAT) templates for testing component interfaces

/// Basic template with valid library interfaces for both import and export
pub const BASIC_LIB_TEMPLATE: &str = r#"
(component
  (type $add_ty (func (param "x" u32) (param "y" u32) (result u32)))
  (import "test:math/lib" (instance $math
    (export "add" (func (type $add_ty)))))

  ;; Core module implementation
  (core module $impl
    (memory (export "mem") 1)
    (func $multiply (param i32 i32) (result i32)
      local.get 0
      local.get 1
      i32.mul)
    (export "multiply" (func $multiply)))

  ;; Create the core module instance
  (core instance $instance (instantiate $impl))

  ;; Define multiply function type
  (type $multiply_ty (func (param "x" u32) (param "y" u32) (result u32)))

  ;; Lift core function to component function
  (func $multiply_lifted (type $multiply_ty)
    (canon lift (core func $instance "multiply")))

  ;; Export library interface
  (export "test:calc/lib" (instance
    (export "multiply" (func $multiply_lifted))))
)"#;

/// Template with non-library (invalid) interfaces
pub const INVALID_INTERFACE_TEMPLATE: &str = r#"
(component
  ;; Define log function type
  (type $log_ty (func (param "msg" string)))

  ;; Import a non-library interface
  (import "test:math/helper" (instance $helper
    (export "log" (func (type $log_ty)))))

  ;; Core module implementation
  (core module $impl
    (memory (export "mem") 1)
    (func $square (param i32) (result i32)
      local.get 0
      local.get 0
      i32.mul)
    (export "square" (func $square)))

  ;; Create the core module instance
  (core instance $instance (instantiate $impl))

  ;; Define square function type
  (type $square_ty (func (param "x" u32) (result u32)))

  ;; Lift core function to component function
  (func $square_lifted (type $square_ty)
    (canon lift (core func $instance "square")))

  ;; Export non-library interface
  (export "test:calc/utils" (instance
    (export "square" (func $square_lifted))))
)"#;

/// Template with multiple library interfaces
pub const MULTI_LIB_TEMPLATE: &str = r#"
(component
  ;; Define function types
  (type $add_ty (func (param "x" u32) (param "y" u32) (result u32)))
  (type $concat_ty (func (param "a" string) (param "b" string) (result string)))

  ;; Import library interfaces
  (import "test:math/lib" (instance $math
    (export "add" (func (type $add_ty)))))

  (import "test:string/lib" (instance $str
    (export "concat" (func (type $concat_ty)))))

  ;; Core module implementation
  (core module $impl
    (memory (export "mem") 1)
    (func $multiply (param i32 i32) (result i32)
      local.get 0
      local.get 1
      i32.mul)
    (func $number_to_string (param i32) (result i32)
      local.get 0)
    (export "multiply" (func $multiply))
    (export "number_to_string" (func $number_to_string)))

  ;; Create the core module instance
  (core instance $instance (instantiate $impl))

  ;; Define result function types
  (type $multiply_ty (func (param "x" u32) (param "y" u32) (result u32)))
  (type $number_to_string_ty (func (param "x" u32) (result string)))

  ;; Lift core functions to component functions
  (func $multiply_lifted (type $multiply_ty)
    (canon lift (core func $instance "multiply")))

  (func $number_to_string_lifted (type $number_to_string_ty)
    (canon lift (core func $instance "number_to_string")))

  ;; Export library interfaces
  (export "test:calc/lib" (instance
    (export "multiply" (func $multiply_lifted))))

  (export "test:format/lib" (instance
    (export "number_to_string" (func $number_to_string_lifted))))
)"#;
