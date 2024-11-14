let db;
let inputBuffer, outputBuffer;
onmessage = async (evt) => {

    console.log("received message", evt);
    if (inputBuffer === undefined) {
        inputBuffer = evt.data.input;
        outputBuffer = evt.data.output;
        console.log("db buffer loaded", evt.data);
    }
    while (true) {
        console.log("db polling..,db=", db);
        const inputArr = new Int32Array(inputBuffer);
        const outputArr = new Int32Array(outputBuffer);
        await Atomics.waitAsync(inputArr, 0, 0).value;
        // Clear status flag so we won't infinite loop
        const command = inputArr[0];
        inputArr[0] = 0;
        const reportResult = data => {
            console.log("Reporting result", data);
            const encoder = new TextEncoder();
            const encoded = encoder.encode(JSON.stringify(data));
            outputArr[1] = encoded.length;
            const u8Buf = new Uint8Array(outputBuffer);
            u8Buf.set(encoded, 8);
            outputArr[0] = 1;
            Atomics.notify(outputArr, 0);
        }
        const readInput = () => {
            const length = inputArr[1];
            const u8Buf = new Uint8Array(inputBuffer);
            const decoder = new TextDecoder();
            const copiedArr = new Uint8Array(length);
            copiedArr.set(u8Buf.subarray(8, 8 + length));
            return JSON.parse(decoder.decode(copiedArr));
        }
        console.log("db received command", command);
        if (command === 1) {
            // Initialize
            try {

                db = await new Promise((resolve, reject) => {
                    const req = indexedDB.open("test", 1);
                    req.onupgradeneeded = evt => {
                        console.log("Updating db");
                        const db = evt.target.result;
                        const objectStore = db.createObjectStore("test_table", {
                            keyPath: "id",
                            autoIncrement: true
                        });
                        objectStore.createIndex("email", "email", { unique: true });
                        console.log("Db updated", db);
                    };
                    req.onsuccess = evt => resolve(evt.target.result);
                    req.onerror = evt => reject(evt.target.error);
                });
                reportResult({ success: true });
            } catch (e) {
                reportResult(e);
            }

        } else if (command === 2) {
            const data = readInput();
            // Add record
            try {
                const result = await new Promise((resolve, reject) => {
                    const transaction = db.transaction(["test_table"], "readwrite");
                    const store = transaction.objectStore("test_table");
                    const newItem = { name: data.name, email: data.email };
                    const request = store.add(newItem);
                    request.onsuccess = () => {
                        console.log("Record added with ID:", request.result);
                        resolve({
                            success: true,
                            result: {
                                id: request.result
                            }
                        });
                    };

                    request.onerror = (evt) => {
                        reject({
                            success: false,
                            message: evt.target.error.message
                        });
                    };

                    transaction.onerror = (evt) => {
                        reject({
                            success: false,
                            message: evt.target.error.message
                        });
                    };
                });
                reportResult(result);
            } catch (e) {
                reportResult(e);
            }

        } else if (command === 3) {
            const data = readInput();
            // Get record
            try {
                const result = await new Promise((resolve, reject) => {
                    const transaction = db.transaction(["test_table"], "readwrite");
                    const store = transaction.objectStore("test_table");
                    const request = store.get(data.id);
                    request.onsuccess = () => {
                        resolve({
                            success: true,
                            result: request.result || null
                        });
                    };

                    request.onerror = (evt) => {
                        reject({
                            success: false,
                            message: evt.target.error.message
                        });
                    };

                    transaction.onerror = (evt) => {
                        reject({
                            success: false,
                            message: evt.target.error.message
                        });
                    };
                })
                reportResult(result);

            } catch (e) {
                reportResult(e);
            }
        } else if (command === 4) {
            // exit
            reportResult({ success: true });
            return;
        }
    }

};
