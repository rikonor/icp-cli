# Task Handoff - HANDOFF-005

## Current State

The Core Interface Types Migration task is at 97% completion. We've identified the specific issues with the WebAssembly Component Model templates that are causing test failures. Two tests are currently failing:

1. `test_duplicate_interface_detection` - Error: "import name `test:math/lib/1` is not a valid extern name"
2. `test_nested_instances_detection` - Error: "Invalid input WebAssembly code at offset 160: type mismatch: expected a type but nothing on stack"

These issues are related to the canonical ABI requirements for the WebAssembly Component Model.

## Completed Work

- Analyzed the code and identified the specific issues with the WebAssembly Component Model templates
- Reviewed the canonical ABI documentation to understand the requirements
- Ran tests to identify which specific tests are failing and the nature of the errors
- Developed a detailed plan for fixing the issues

## Technical Details

### Canonical ABI Requirements

The WebAssembly Component Model's canonical ABI has specific requirements for different types:

1. **String Parameters**: When a component function takes a string parameter, the corresponding core function must take two i32 parameters:

   - A pointer to the string data in memory
   - The length of the string

2. **String Returns**: When a component function returns a string, the corresponding core function must return two i32 values:

   - A pointer to the string data in memory
   - The length of the string

3. **Boolean Returns**: When a component function returns a boolean, special handling is needed with the `boolean-to-i32` option in the canon lift operation.

### Identified Issues

1. **DUPLICATE_INTERFACE_TEMPLATE**: The error "import name `test:math/lib/1` is not a valid extern name" suggests there might be an issue with how duplicate interface names are handled. The `/1` suffix might not be a valid part of an interface name.

2. **NESTED_INSTANCES_TEMPLATE**: The error "type mismatch: expected a type but nothing on stack" suggests a function signature mismatch. Based on the handoff document HANDOFF-004.md, this is likely related to the `log_impl` function, which might not be correctly handling string parameters or returns.

3. **Other Potential Issues**: Based on HANDOFF-004.md, there might also be issues with:
   - The `number_to_double` function in `MULTI_LIB_TEMPLATE`
   - Boolean conversion in `MANY_INTERFACES_TEMPLATE`

## Challenges

- **Parameter Type Mismatch**: The canonical ABI expects two parameters for string arguments, but some core functions might only accept one.
- **Return Type Mismatch**: The canonical ABI expects two return values for string returns, but some core functions might only return one.
- **Boolean Conversion**: Boolean return values require special handling with the `boolean-to-i32` option.

## Next Steps

1. **Fix DUPLICATE_INTERFACE_TEMPLATE**:

   - Review the interface naming convention in the template
   - Update the interface names to be valid extern names (remove the `/1` suffix or handle it differently)

2. **Fix NESTED_INSTANCES_TEMPLATE**:

   - Check the `log_impl` function signature
   - Ensure it correctly handles string parameters (two i32 parameters)
   - If it returns a string, ensure it returns two i32 values (pointer and length)

3. **Review Other Templates**:

   - Check the `number_to_double` function in `MULTI_LIB_TEMPLATE`
   - Verify boolean conversion in `MANY_INTERFACES_TEMPLATE`

4. **Run Tests and Verify Fixes**:

   - Run `cargo test --package test-utils --test iface_detection -- --nocapture` to verify the fixes
   - Address any remaining issues

5. **Update Documentation**:

   - Add comments to the templates explaining the canonical ABI requirements
   - Update any relevant documentation

6. **Uncomment Error Handling Tests**:
   - Once the main tests are passing, uncomment and fix the error handling tests in `iface_detection.rs`

## Notes

- The canonical ABI documentation in `docs/explainer/explainer-canonical-abi.md` provides detailed information about the requirements.
- The type system documentation in `docs/explainer/explainer-type-system.md` provides information about how different types are handled.
- The error message from HANDOFF-004.md ("lowered result types `[I32]` do not match result types `[I32 I32]` of core function 2") suggests that a function is returning a single I32 value when the canonical ABI expects two I32 values, which is typical for string-returning functions.
- The fixes should be minimal and targeted to maintain compatibility with the existing codebase.
