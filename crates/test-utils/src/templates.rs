//! WebAssembly text format (WAT) templates for testing component interfaces

/// Empty component template with no imports or exports
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

    ;; For boolean return values, the canonical ABI expects a different signature
    (func $is_even_lifted (type $is_even_ty)
      (canon lift
        (core func $instance "is-even")
        (memory $instance "mem")
        (realloc (func $instance "realloc"))
        (boolean-to-i32)))

    (func $is_positive_lifted (type $is_positive_ty)
      (canon lift
        (core func $instance "is-positive")
        (memory $instance "mem")
        (realloc (func $instance "realloc"))
        (boolean-to-i32)))

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
    (func $log_impl (param i32 i32) (result i32)
      local.get 0)  ;; Just return the pointer, ignoring length for test purposes
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
pub const DUPLICATE_INTERFACE_TEMPLATE: &str = r#"
(component
  ;; Define function types
  (type $add_ty (func (param "x" u32) (param "y" u32) (result u32)))
  (type $subtract_ty (func (param "x" u32) (param "y" u32) (result u32)))

  ;; Import library interfaces with duplicate names
  (import "test:math/lib" (instance $math1
    (export "add" (func (type $add_ty)))))

  (import "test:math/lib" (instance $math2
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
    ;; For string return values, the canonical ABI expects a different signature
    ;; This is a simplified version for testing purposes
    (func $number_to_string (param i32) (result i32 i32)
      local.get 0  ;; Return pointer
      i32.const 1) ;; Return length
    (export "multiply" (func $multiply))
    (export "number-to-string" (func $number_to_string)))

  ;; Create the core module instance
  (core instance $instance (instantiate $impl))

  ;; Define result function types
  (type $multiply_ty (func (param "x" u32) (param "y" u32) (result u32)))
  (type $number_to_string_ty (func (param "x" u32) (result i32 i32)))

  ;; Lift core functions to component functions
  (func $multiply_lifted (type $multiply_ty)
    (canon lift
      (core func $instance "multiply")
      (memory $instance "mem")
      (realloc (func $instance "realloc"))))

  (func $number_to_string_lifted (type $number_to_string_ty)
    (canon lift
      (core func $instance "number-to-string")
      (memory $instance "mem")
      (realloc (func $instance "realloc"))))

  ;; Define library instances with exports
  (instance $calc
    (export "multiply" (func $multiply_lifted)))

  (instance $format
    (export "number-to-string" (func $number_to_string_lifted)))

  ;; Export library interfaces
  (export "test:calc/lib" (instance $calc))
  (export "test:format/lib" (instance $format))
)"#;
