# Simple proof-of-concept of wrapped sync API for IndexedDB

## How to use
- Install wasm-pack: `cargo install wasm-pack`
- Build wasm binaries: `cd wasmlib && wasm-pack build --target web`
- Use a web server to host directory `wasmlib`, such as nginx. The following is an example of nginx configuration

```
    server {
        listen 5600;
        server_name localhost;
        root /root/wasm-webworker-poc/wasmlib;
        location / {
            index index.html;
            add_header Cross-Origin-Embedder-Policy "require-corp" always;
            add_header Cross-Origin-Opener-Policy "same-origin" always;
        }
    }
```

- Note that the two headers are necessary for SharedArrayBuffer.

- Access `http://127.0.0.1:5600` and try it.



