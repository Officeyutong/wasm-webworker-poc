use std::cell::RefCell;

use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use serde_json::json;
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsValue};
use web_sys::js_sys::{
    Atomics, Function, Int32Array, SharedArrayBuffer, Uint32Array, Uint8Array, Uint8ClampedArray,
};
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    #[wasm_bindgen(js_namespace = console,js_name="log")]
    fn logj(s: &JsValue);
    #[wasm_bindgen(js_name = "setTimeout")]
    fn set_timeout(f: &Function, timeout: usize);

}

#[derive(Serialize, Deserialize)]
#[serde(tag = "command")]
pub enum Command {
    AddRecord { name: String, email: String },
    GetRecord { id: i32 },
    Stop,
    Init,
}
#[derive(Serialize, Deserialize, Debug)]

pub enum CommandResult {
    AddRecord { id: i32 },
    GetRecord { name: String, email: String },
}

thread_local! {
    static INPUT_BUFFER:RefCell<Option<SharedArrayBuffer>> =const { RefCell::new( None) };
    static OUTPUT_BUFFER:RefCell<Option<SharedArrayBuffer>> =const { RefCell::new( None) };
}

#[wasm_bindgen]
pub fn set_shared_array(input: JsValue, output: JsValue) {
    log(&format!("Setting shared array: {:?}, {:?}", input, output));
    INPUT_BUFFER.with(|v| {
        *v.borrow_mut() = Some(input.dyn_into().unwrap());
    });
    OUTPUT_BUFFER.with(|v| {
        *v.borrow_mut() = Some(output.dyn_into().unwrap());
    });
}

pub fn call_db_worker(
    command: usize,
    payload: serde_json::Value,
) -> anyhow::Result<serde_json::Value> {
    log("Calling db worker");
    let encoded = serde_json::to_vec(&payload).with_context(|| anyhow!("Failed to encode"))?;
    // Set up output
    OUTPUT_BUFFER.with(|v| {
        let u32arr = Uint32Array::new(&v.borrow().clone().unwrap());
        u32arr.set_index(0, 0);
    });
    log("Output set");
    // Set up input
    INPUT_BUFFER.with(|v| {
        let binding = v.borrow();
        let buf = binding.as_ref().unwrap();
        let i32arr = Int32Array::new(buf);
        let u8arr = Uint8ClampedArray::new(buf);
        i32arr.set_index(1, encoded.len() as i32);
        for (i, v) in encoded.into_iter().enumerate() {
            u8arr.set_index((8 + i) as u32, v);
        }
        i32arr.set_index(0, command as i32);
        Atomics::notify(&i32arr, 0).unwrap();
    });
    log("Input set, waiting result");
    // Wait for result
    let result = OUTPUT_BUFFER.with(|v| {
        let buf = v.borrow().clone().unwrap();
        let i32arr = Int32Array::new(&buf);
        Atomics::wait(&i32arr, 0, 0).unwrap();
        let u8arr = Uint8Array::new(&buf);

        let len = i32arr.get_index(1) as usize;
        let mut buf = vec![0u8; 0];
        for i in 0..len {
            buf.push(u8arr.get_index((8 + i) as u32));
        }
        serde_json::from_slice::<serde_json::Value>(&buf)
    })?;
    Ok(result)
}

#[wasm_bindgen]
pub fn dispatch_command(command_str: String) -> Result<String, String> {
    let parsed_command = serde_json::from_str::<Command>(&command_str)
        .map_err(|e| format!("Failed to parse command: {}", e))?;
    log(&format!("raw command: {}", command_str));
    let result = match parsed_command {
        Command::Init => call_db_worker(1, json!({})),
        Command::AddRecord { name, email } => call_db_worker(
            2,
            json!({
                "name":name,
                "email":email,
            }),
        ),
        Command::GetRecord { id } => call_db_worker(
            3,
            json!({
                "id":id
            }),
        ),
        Command::Stop => call_db_worker(4, json!({})),
    }
    .map_err(|e| format!("Failed to call db worker: {}", e))?;

    log(&format!("result={:?}", result));
    Ok(serde_json::to_string(&result).unwrap())
}

#[wasm_bindgen(start)]
fn startup() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}
