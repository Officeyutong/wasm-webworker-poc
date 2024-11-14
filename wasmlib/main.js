
// importScripts("./pkg/wasmlib.js");
import * as wasmlib from "./pkg/wasmlib.js";
await wasmlib.default("./pkg/wasmlib_bg.wasm");
console.log("main wrapper initialized");
let first = true;
onmessage = msg => {
    console.log("main wrapper received", msg);
    if (first) {
        first = false;
        wasmlib.set_shared_array(msg.data.input, msg.data.output);
        console.log("initialized");
        return;
    }
    const result = wasmlib.dispatch_command(JSON.stringify(msg.data));
    self.postMessage(JSON.parse(result));
}
