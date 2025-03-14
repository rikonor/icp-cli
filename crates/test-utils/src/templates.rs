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

/// Template with nested instances
pub const NESTED_INSTANCES_TEMPLATE: &str = r#"
(component
  ;; Define function types
  (type $add_ty (func (param "x" u32) (param "y" u32) (result u32)))
  (type $log_ty (func (param "msg" string)))

  ;; Import library interface
  (import "test:math/lib" (instance $math
    (export "add" (func (type $add_ty)))))

  ;; Core module implementation
  (core module $impl
    (memory (export "mem") 1)

    ;; Realloc function required for string handling
    (func $realloc (param i32 i32 i32 i32) (result i32)
      (i32.const 0))  ;; Dummy implementation for testing
    (export "realloc" (func $realloc))

    ;; Core implementations
    (func $log_impl (param i32 i32) (result)
      drop
      drop)  ;; No return value needed for this test
    (export "log-impl" (func $log_impl))

    (func $multiply (param i32 i32) (result i32)
      local.get 0
      local.get 1
      i32.mul)
    (export "multiply" (func $multiply))
  )

  ;; Create the core module instance
  (core instance $instance (instantiate $impl))

  ;; Define result function types
  (type $multiply_ty (func (param "x" u32) (param "y" u32) (result u32)))

  ;; Lift core functions to component functions
  (func $log_lifted (type $log_ty)
    (canon lift
      (core func $instance "log-impl")
      (memory $instance "mem")
      (realloc (func $instance "realloc"))))

  (func $multiply_lifted (type $multiply_ty)
    (canon lift
      (core func $instance "multiply")
      (memory $instance "mem")
      (realloc (func $instance "realloc"))))

  ;; Define nested instances
  (instance $logger
    (export "log" (func $log_lifted)))

  (instance $calc
    (export "multiply" (func $multiply_lifted)))

  ;; Define a parent instance that contains the nested instances
  (instance $utils
    (export "logger" (instance $logger))
    (export "calc" (instance $calc)))

  ;; Export the parent instance
  (export "test:utils/lib" (instance $utils))
)"#;

/// Template with duplicate interface names
///
/// Purpose:
/// - Tests handling of components with duplicate interface names
/// - Verifies correct detection of interfaces with the same name but different functions
///
/// Structure:
/// - Imports two interfaces with similar names (test:math/lib/1 and test:math/lib/2)
/// - Exports two interfaces with the exact same name (test:calc/lib)
///
/// Key Features:
/// - Duplicate interface names
/// - Multiple imports and exports
///
/// Common Issues:
/// - Some WebAssembly implementations may reject duplicate import names
/// - Interface detection must handle duplicate names correctly
///
/// Test Expectations:
/// - Should detect two imports with different functions
/// - Should detect two exports with the same name but different functions
pub const DUPLICATE_INTERFACE_TEMPLATE: &str = r#"
(component
  ;; Define function types
  (type $add_ty (func (param "x" u32) (param "y" u32) (result u32)))
  (type $subtract_ty (func (param "x" u32) (param "y" u32) (result u32)))

  ;; Import library interfaces with similar names
  (import "test:math/lib/1" (instance $math1
    (export "add" (func (type $add_ty)))))

  (import "test:math/lib/2" (instance $math2
    (export "subtract" (func (type $subtract_ty)))))

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

    (func $divide (param i32 i32) (result i32)
      local.get 0
      local.get 1
      i32.div_s)
    (export "divide" (func $divide))
  )

  ;; Create the core module instance
  (core instance $instance (instantiate $impl))

  ;; Define result function types
  (type $multiply_ty (func (param "x" u32) (param "y" u32) (result u32)))
  (type $divide_ty (func (param "x" u32) (param "y" u32) (result u32)))

  ;; Lift core functions to component functions
  (func $multiply_lifted (type $multiply_ty)
    (canon lift
      (core func $instance "multiply")
      (memory $instance "mem")
      (realloc (func $instance "realloc"))))

  (func $divide_lifted (type $divide_ty)
    (canon lift
      (core func $instance "divide")
      (memory $instance "mem")
      (realloc (func $instance "realloc"))))

  ;; Define library instances with exports
  (instance $calc1
    (export "multiply" (func $multiply_lifted)))

  (instance $calc2
    (export "divide" (func $divide_lifted)))

  ;; Export library interfaces with duplicate names
  (export "test:calc/lib" (instance $calc1))
  (export "test:calc/lib" (instance $calc2))
)"#;

/// Template with missing memory (for error testing)
pub const MISSING_MEMORY_TEMPLATE: &str = r#"
(component
  ;; Core module implementation without memory export
  (core module $impl
    ;; Realloc function required for string handling
    (func $realloc (param i32 i32 i32 i32) (result i32)
      (i32.const 0))  ;; Dummy implementation for testing
    (export "realloc" (func $realloc))

    ;; Core implementation
    (func $add (param i32 i32) (result i32)
      local.get 0
      local.get 1
      i32.add)
    (export "add" (func $add))
  )

  ;; Create core instance
  (core instance $instance (instantiate $impl))

  ;; Define add function type
  (type $add_ty (func (param "x" u32) (param "y" u32) (result u32)))

  ;; Attempt to lift core function to component function
  ;; This will fail because memory is missing
  (func $add_lifted (type $add_ty)
    (canon lift
      (core func $instance "add")
      ;; Missing memory reference
      (realloc (func $instance "realloc"))))

  ;; Define math library instance with exports
  (instance $math
    (export "add" (func $add_lifted)))

  ;; Export the math library instance
  (export "test:math/lib" (instance $math))
)"#;

/// Template with missing realloc function (for error testing)
pub const MISSING_REALLOC_TEMPLATE: &str = r#"
(component
  ;; Core module implementation without realloc export
  (core module $impl
    (memory (export "mem") 1)

    ;; Core implementation
    (func $add (param i32 i32) (result i32)
      local.get 0
      local.get 1
      i32.add)
    (export "add" (func $add))
  )

  ;; Create core instance
  (core instance $instance (instantiate $impl))

  ;; Define add function type
  (type $add_ty (func (param "x" u32) (param "y" u32) (result u32)))

  ;; Attempt to lift core function to component function
  ;; This will fail because realloc is missing
  (func $add_lifted (type $add_ty)
    (canon lift
      (core func $instance "add")
      (memory $instance "mem")
      ;; Missing realloc reference
    ))

  ;; Define math library instance with exports
  (instance $math
    (export "add" (func $add_lifted)))

  ;; Export the math library instance
  (export "test:math/lib" (instance $math))
)"#;

/// Template with non-library (invalid) interfaces
///
/// Purpose:
/// - Tests handling of non-standard interface naming patterns
/// - Verifies detection of interfaces that don't follow the "/lib" naming convention
///
/// Structure:
/// - Imports an interface with a non-standard name (test:math/helper)
/// - Exports an interface with a non-standard name (test:calc/utils)
///
/// Key Features:
/// - Non-standard interface naming
/// - String parameter handling
///
/// Common Issues:
/// - Interface detection should work regardless of naming conventions
/// - Some implementations might have special handling for "/lib" interfaces
///
/// Test Expectations:
/// - Should detect one import: "test:math/helper" with function "log"
/// - Should detect one export: "test:calc/utils" with function "square"
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

    ;; Realloc function required for string handling
    (func $realloc (param i32 i32 i32 i32) (result i32)
      (i32.const 0))  ;; Dummy implementation for testing
    (export "realloc" (func $realloc))

    ;; Core implementation
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
    (canon lift
      (core func $instance "square")
      (memory $instance "mem")
      (realloc (func $instance "realloc"))))

  ;; Define utils instance with exports
  (instance $utils
    (export "square" (func $square_lifted)))

  ;; Export non-library interface
  (export "test:calc/utils" (instance $utils))
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
