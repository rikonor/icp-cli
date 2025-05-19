use anyhow::{anyhow, Context, Error};
use icp_component_invoke::Val as IcpVal;
use wasmtime::{component::Val as WasmVal, StoreContextMut};

use crate::State;

/// Executes a resolved component function with the given parameters.
///
/// This function handles the asynchronous call to the WebAssembly function,
/// processes its return value, and serializes the result into a list of U8 WasmVals.
pub async fn execute_component_function(
    mut store: &mut StoreContextMut<'_, State>,
    func_to_call: &wasmtime::component::Func,
    nested_params_bytes: Vec<u8>, // Changed from &[WasmVal]
) -> Result<Vec<WasmVal>, Error> {
    // Deserialize params from bytes
    let nested_icp_vals = match serde_json::from_slice::<Vec<IcpVal>>(&nested_params_bytes) {
        Ok(params) => params,
        Err(err) => {
            return Err(anyhow!(
                "failed to deserialize nested params from json: {}",
                err
            ));
        }
    };

    let actual_nested_params_wasm = nested_icp_vals
        .into_iter()
        .map(WasmVal::from)
        .collect::<Vec<WasmVal>>();

    // This initial vec for results needs to match the expected arity and types
    // of the actual WebAssembly function being called.
    // The original code used `vec![WasmVal::Bool(false)]` assuming a single result.
    // If functions can have multiple results or different types, this needs to be more dynamic
    // or the function signature metadata should be used to create this.
    // For now, sticking to the original pattern.
    let mut actual_nested_results_wasm = vec![WasmVal::Bool(false)];

    func_to_call
        .call_async(
            &mut store,
            &actual_nested_params_wasm, // Use the processed params
            &mut actual_nested_results_wasm,
        )
        .await
        .with_context(|| format!("error during Wasm function call_async",))?;

    func_to_call
        .post_return_async(&mut store)
        .await
        .with_context(|| format!("error during Wasm function post_return_async",))?;

    // Convert the WasmVal results from the call into IcpVal
    let results_as_icp_val = actual_nested_results_wasm
        .into_iter()
        .map(IcpVal::from)
        .collect::<Vec<IcpVal>>();

    // Serialize the IcpVal results into a byte vector (JSON representation)
    let results_as_bytes = serde_json::to_vec(&results_as_icp_val)
        .with_context(|| format!("failed to serialize nested results to json",))?;

    // Convert the byte vector into a list of U8 WasmVals
    let results_as_wasm_u8_list = results_as_bytes
        .into_iter()
        .map(WasmVal::U8)
        .collect::<Vec<WasmVal>>();

    Ok(results_as_wasm_u8_list)
}

/// Attempts to convert a WasmVal, expected to be a List of U8, into a Vec<u8>.
pub(crate) fn try_wasm_list_u8_to_vec_u8(raw_val: &WasmVal) -> Result<Vec<u8>, Error> {
    match raw_val {
        // Correct
        WasmVal::List(vs) => vs
            .iter()
            .map(|item| match item {
                WasmVal::U8(val_u8) => Ok(*val_u8),
                _ => Err(anyhow!("List contains a non-U8 type: {:?}", item)),
            })
            .collect::<Result<Vec<u8>, Error>>() // Collect into a Result<Vec<u8>, anyhow::Error>
            .map_err(|e| anyhow!("Failed to convert WasmVal List to Vec<u8>: {}", e)),

        // Wrong
        _ => Err(anyhow!(
            "Input WasmVal is not a List (expected List of U8): {:?}",
            raw_val
        )),
    }
}
