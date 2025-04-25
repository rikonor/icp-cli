//! WebAssembly text format (WAT) templates for testing component interfaces
//!
//! This file contains a collection of WebAssembly Component Model templates in WAT format
//! that are used for testing interface detection and other component model features.
//! Each template demonstrates different aspects of the component model and is designed
//! to test specific functionality.

/// Empty component template with no imports or exports
///
/// Purpose:
/// - Tests the minimal valid component structure
/// - Verifies that empty components are handled correctly
///
/// Structure:
/// - Core module with memory and realloc function
/// - No imports or exports at the component level
///
/// Key Features:
/// - Minimal valid component structure
/// - Memory and realloc exports from core module
///
/// Test Expectations:
/// - Should detect zero imports and zero exports
pub const EMPTY_COMPONENT_TEMPLATE: &str = r#"
(component
  ;; Core module implementation
  (core module $impl
    (memory (export "mem") 1)

    ;; Realloc function required for string handling
    (func $realloc (param i32 i32 i32 i32) (result i32)
      (i32.const 0))  ;; Dummy implementation for testing
    (export "realloc" (func $realloc))

    ;; Empty core implementation with no exports
    (func $internal_func (param i32) (result i32)
      local.get 0)
  )

  ;; Create core instance
  (core instance $instance (instantiate $impl))

  ;; No imports or exports defined
)
"#;

/// Basic template with valid library interfaces for both import and export
///
/// Purpose:
/// - Tests the basic functionality of importing and exporting library interfaces
/// - Demonstrates the minimal required structure for a valid component with interfaces
///
/// Structure:
/// - Imports a math library with an "add" function
/// - Defines a core module that implements a "multiply" function
/// - Lifts the core function to a component function
/// - Exports a calc library with the multiply function
///
/// Key Features:
/// - Function type definitions with named parameters
/// - Canon lifting from core functions to component functions
/// - Memory and realloc handling for the canonical ABI
///
/// Common Issues:
/// - Requires proper memory and realloc exports from the core module
/// - Function signatures must match between core and component levels
///
/// Test Expectations:
/// - Should detect one import: "test:math/lib" with function "add"
/// - Should detect one export: "test:calc/lib" with function "multiply"
pub const BASIC_LIB_TEMPLATE: &str = r#"
(component
  ;; Import interface type and instance
  (type $add_ty (func (param "x" u32) (param "y" u32) (result u32)))
  (import "test:math/lib"
    (instance $math
      (export "add" (func (type $add_ty)))))

  ;; Core module implementation
  (core module $impl
    (memory (export "mem") 1)

    ;; Realloc function required for string handling
    (func $realloc (param i32 i32 i32 i32) (result i32)
      (i32.const 0))  ;; Dummy implementation for testing
    (export "realloc" (func $realloc))

    ;; Core implementation
    (func $multiply (param i32 i32) (result i32)
      local.get 0
      local.get 1
      i32.mul)
    (export "multiply" (func $multiply)))

  ;; Create core instance
  (core instance $instance (instantiate $impl))

  ;; Define multiply function type
  (type $multiply_ty (func (param "x" u32) (param "y" u32) (result u32)))

  ;; Lift core function to component function
  (func $multiply_lifted (type $multiply_ty)
    (canon lift
      (core func $instance "multiply")
      (memory $instance "mem")
      (realloc (func $instance "realloc"))))

  ;; Define calc library instance with exports
  (instance $calc
    (export "multiply" (func $multiply_lifted)))

  ;; Export the calc library instance
  (export "test:calc/lib" (instance $calc))
)"#;

/// Minimal valid extension template that implements the icp-cli world.wit requirements
///
/// Imports:
/// - local:host/misc with print, rand, time
///
/// Exports:
/// - local:extension/cli with spec and run
///
/// Features:
/// - Compliant with icp-cli's component model
/// - Dummy implementations for all required functions
/// - Includes memory and realloc for canonical ABI
pub const EXTENSION_MINIMAL_TEMPLATE: &str = r#"
(component
  ;; Import required host interfaces
  (import "local:host/misc" (instance $misc
    (export "print" (func (param "s" string)))
    (export "rand" (func (result u8)))
    (export "time" (func (result u64)))
  ))

  ;; Core module with required infrastructure
  (core module $impl
    (memory (export "mem") 1)

    ;; Realloc implementation (simplified for testing)
    (func $realloc (param i32 i32 i32 i32) (result i32)
      (i32.const 0)
    )
    (export "realloc" (func $realloc))

    ;; spec implementation - returns empty string via memory
    (func $spec (result i32)
      ;; Store ptr (0) and len (0) at fixed memory locations
      (i32.store (i32.const 0) (i32.const 0))  ;; ptr
      (i32.store (i32.const 4) (i32.const 0))  ;; len
      (i32.const 0)  ;; return pointer to (ptr, len) pair
    )
    (export "spec" (func $spec))

    ;; run implementation - returns success (0)
    (func $run (param i32 i32) (result i32)
      (i32.const 0)
    )
    (export "run" (func $run))
  )

  (core instance $instance (instantiate $impl))

  ;; Lift spec function to component-level
  (func $spec_lifted (result string)
    (canon lift
      (core func $instance "spec")
      (memory $instance "mem")
      (realloc (func $instance "realloc"))
    )
  )

  ;; Lift run function to component-level
  (func $run_lifted (param "args" (list string)) (result u8)
    (canon lift
      (core func $instance "run")
      (memory $instance "mem")
      (realloc (func $instance "realloc"))
    )
  )

  ;; Export the CLI interface
  (instance $cli
    (export "spec" (func $spec_lifted))
    (export "run" (func $run_lifted))
  )
  (export "local:extension/cli" (instance $cli))
)
"#;

/// Template with many interfaces (stress test)
pub const MANY_INTERFACES_TEMPLATE: &str = r#"
(component
  ;; Define function types
  (type $add_ty (func (param "x" u32) (param "y" u32) (result u32)))
  (type $subtract_ty (func (param "x" u32) (param "y" u32) (result u32)))
  (type $multiply_ty (func (param "x" u32) (param "y" u32) (result u32)))
  (type $divide_ty (func (param "x" u32) (param "y" u32) (result u32)))
  (type $log_ty (func (param "msg" string)))
  (type $concat_ty (func (param "a" string) (param "b" string) (result string)))
  (type $length_ty (func (param "s" string) (result u32)))
  (type $to_upper_ty (func (param "s" string) (result string)))
  (type $to_lower_ty (func (param "s" string) (result string)))

  ;; Import multiple library interfaces
  (import "test:math/lib" (instance $math
    (export "add" (func (type $add_ty)))
    (export "subtract" (func (type $subtract_ty)))
    (export "multiply" (func (type $multiply_ty)))
    (export "divide" (func (type $divide_ty)))))

  (import "test:string/lib" (instance $str
    (export "concat" (func (type $concat_ty)))
    (export "length" (func (type $length_ty)))
    (export "to-upper" (func (type $to_upper_ty)))
    (export "to-lower" (func (type $to_lower_ty)))))

  (import "test:logger/lib" (instance $logger
    (export "log" (func (type $log_ty)))))

  ;; Core module implementation
  (core module $impl
    (memory (export "mem") 1)

    ;; Realloc function required for string handling
    (func $realloc (param i32 i32 i32 i32) (result i32)
      (i32.const 0))  ;; Dummy implementation for testing
    (export "realloc" (func $realloc))

    ;; Core implementations
    (func $square (param i32) (result i32)
      local.get 0
      local.get 0
      i32.mul)
    (export "square" (func $square))

    (func $cube (param i32) (result i32)
      local.get 0
      local.get 0
      i32.mul
      local.get 0
      i32.mul)
    (export "cube" (func $cube))

    (func $is_even (param i32) (result i32)
      local.get 0
      i32.const 2
      i32.rem_u
      i32.eqz)
    (export "is-even" (func $is_even))

    (func $is_positive (param i32) (result i32)
      local.get 0
      i32.const 0
      i32.gt_s)
    (export "is-positive" (func $is_positive))
  )

  ;; Create the core module instance
  (core instance $instance (instantiate $impl))

  ;; Define result function types
  (type $square_ty (func (param "x" u32) (result u32)))
  (type $cube_ty (func (param "x" u32) (result u32)))
  (type $is_even_ty (func (param "x" u32) (result bool)))
  (type $is_positive_ty (func (param "x" s32) (result bool)))

  ;; Lift core functions to component functions
  (func $square_lifted (type $square_ty)
    (canon lift
      (core func $instance "square")
      (memory $instance "mem")
      (realloc (func $instance "realloc"))))

  (func $cube_lifted (type $cube_ty)
    (canon lift
      (core func $instance "cube")
      (memory $instance "mem")
      (realloc (func $instance "realloc"))))

    ;; For boolean return values
    (func $is_even_lifted (type $is_even_ty)
      (canon lift
        (core func $instance "is-even")
        (memory $instance "mem")
        (realloc (func $instance "realloc"))))

    (func $is_positive_lifted (type $is_positive_ty)
      (canon lift
        (core func $instance "is-positive")
        (memory $instance "mem")
        (realloc (func $instance "realloc"))))

  ;; Define library instances with exports
  (instance $math_utils
    (export "square" (func $square_lifted))
    (export "cube" (func $cube_lifted)))

  (instance $number_utils
    (export "is-even" (func $is_even_lifted))
    (export "is-positive" (func $is_positive_lifted)))

  ;; Export library interfaces
  (export "test:math-utils/lib" (instance $math_utils))
  (export "test:number-utils/lib" (instance $number_utils))
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

    ;; Realloc function required for string handling
    (func $realloc (param i32 i32 i32 i32) (result i32)
      (i32.const 0))  ;; Dummy implementation for testing
    (export "realloc" (func $realloc))

    ;; Core implementations
    (func $multiply (param i32 i32) (result i32)
      local.get 0
      local.get 1
      i32.mul)
    ;; Simple function that doubles a number
    (func $number_to_double (param i32) (result i32)
      local.get 0
      i32.const 2
      i32.mul)
    (export "multiply" (func $multiply))
    (export "number-to-double" (func $number_to_double)))

  ;; Create the core module instance
  (core instance $instance (instantiate $impl))

  ;; Define result function types
  (type $multiply_ty (func (param "x" u32) (param "y" u32) (result u32)))
  (type $number_to_double_ty (func (param "x" u32) (result u32)))

  ;; Lift core functions to component functions
  (func $multiply_lifted (type $multiply_ty)
    (canon lift
      (core func $instance "multiply")
      (memory $instance "mem")
      (realloc (func $instance "realloc"))))

  ;; Simple numeric function that doubles a number
  (func $number_to_double_lifted (type $number_to_double_ty)
    (canon lift
      (core func $instance "number-to-double")
      (memory $instance "mem")
      (realloc (func $instance "realloc"))))

  ;; Define library instances with exports
  (instance $calc
    (export "multiply" (func $multiply_lifted)))

  (instance $format
    (export "number-to-double" (func $number_to_double_lifted)))

  ;; Export library interfaces
  (export "test:calc/lib" (instance $calc))
  (export "test:format/lib" (instance $format))
)"#;

/// Template with versioned library interfaces
///
/// Purpose:
/// - Tests the detection of versioned interfaces (with @x.y.z suffix)
/// - Demonstrates the issue with the current interface detection logic
///
/// Structure:
/// - Imports a math library with version ("test:math/lib@0.0.1")
/// - Exports a calc library with version ("test:calc/lib@0.0.1")
///
/// Key Features:
/// - Versioned interface names with @x.y.z suffix
///
/// Test Expectations:
/// - With current implementation: Should fail to detect imports and exports
/// - With fixed implementation: Should detect one import and one export
pub const VERSIONED_LIB_TEMPLATE: &str = r#"
(component
  ;; Import interface type and instance with version
  (type $add_ty (func (param "x" u32) (param "y" u32) (result u32)))
  (import "test:math/lib@0.0.1"
    (instance $math
      (export "add" (func (type $add_ty)))))

  ;; Core module implementation
  (core module $impl
    (memory (export "mem") 1)

    ;; Realloc function required for string handling
    (func $realloc (param i32 i32 i32 i32) (result i32)
      (i32.const 0))  ;; Dummy implementation for testing
    (export "realloc" (func $realloc))

    ;; Core implementation
    (func $multiply (param i32 i32) (result i32)
      local.get 0
      local.get 1
      i32.mul)
    (export "multiply" (func $multiply)))

  ;; Create core instance
  (core instance $instance (instantiate $impl))

  ;; Define multiply function type
  (type $multiply_ty (func (param "x" u32) (param "y" u32) (result u32)))

  ;; Lift core function to component function
  (func $multiply_lifted (type $multiply_ty)
    (canon lift
      (core func $instance "multiply")
      (memory $instance "mem")
      (realloc (func $instance "realloc"))))

  ;; Define calc library instance with exports
  (instance $calc
    (export "multiply" (func $multiply_lifted)))

  ;; Export the calc library instance with version
  (export "test:calc/lib@0.0.1" (instance $calc))
)"#;

/// Template with mixed versioned and non-versioned library interfaces
///
/// Purpose:
/// - Tests the detection of both versioned and non-versioned interfaces
/// - Ensures both types of interfaces can work together
///
/// Structure:
/// - Imports both versioned ("test:math/lib@0.0.1") and non-versioned ("test:string/lib") interfaces
/// - Exports both versioned ("test:calc/lib@0.0.1") and non-versioned ("test:format/lib") interfaces
///
/// Key Features:
/// - Mix of versioned and non-versioned interface names
///
/// Test Expectations:
/// - With current implementation: Should only detect non-versioned interfaces
/// - With fixed implementation: Should detect all interfaces
pub const MIXED_VERSIONED_LIB_TEMPLATE: &str = r#"
(component
  ;; Import interface types
  (type $add_ty (func (param "x" u32) (param "y" u32) (result u32)))
  (type $concat_ty (func (param "a" string) (param "b" string) (result string)))

  ;; Import versioned math library
  (import "test:math/lib@0.0.1"
    (instance $math
      (export "add" (func (type $add_ty)))))

  ;; Import non-versioned string library
  (import "test:string/lib"
    (instance $str
      (export "concat" (func (type $concat_ty)))))

  ;; Core module implementation
  (core module $impl
    (memory (export "mem") 1)

    ;; Realloc function required for string handling
    (func $realloc (param i32 i32 i32 i32) (result i32)
      (i32.const 0))  ;; Dummy implementation for testing
    (export "realloc" (func $realloc))

    ;; Core implementations
    (func $multiply (param i32 i32) (result i32)
      local.get 0
      local.get 1
      i32.mul)
    (export "multiply" (func $multiply))

    (func $format (param i32) (result i32)
      local.get 0)
    (export "format" (func $format)))

  ;; Create core instance
  (core instance $instance (instantiate $impl))

  ;; Define function types
  (type $multiply_ty (func (param "x" u32) (param "y" u32) (result u32)))
  (type $format_ty (func (param "x" u32) (result u32)))

  ;; Lift core functions to component functions
  (func $multiply_lifted (type $multiply_ty)
    (canon lift
      (core func $instance "multiply")
      (memory $instance "mem")
      (realloc (func $instance "realloc"))))

  (func $format_lifted (type $format_ty)
    (canon lift
      (core func $instance "format")
      (memory $instance "mem")
      (realloc (func $instance "realloc"))))

  ;; Define library instances with exports
  (instance $calc
    (export "multiply" (func $multiply_lifted)))

  (instance $format
    (export "format" (func $format_lifted)))

  ;; Export the library instances
  (export "test:calc/lib@0.0.1" (instance $calc))
  (export "test:format/lib" (instance $format))
)"#;
