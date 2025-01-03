This is a temporary README for the purpose of explaining how one can run the
example.

# Project structure
`/wasm` is where handwritten JS binding code resides.
`/example` is example that can run in web with missing FMOD Emscripten module.
`/resources` is dir for binary assets, these are included with include_bytes!().
`/src/main.rs` is the binary example that can run in both WASM and native.
`/src/wasmfmod.rs` is where functions that call WASM and all that stuff resides.

# How to run the example
*NOTE:* I only tried it on Windows as that's what I have access to (my PC).
*NOTE:* I also used FMOD Studio's example for the purposes of testing this.

- Put FMOD assets you want in `/resources`.
- Update `/src/main.rs` with assets added.
- Get FMOD's dev libraries as described as here:
  https://github.com/lebedec/libfmod?tab=readme-ov-file#fmod-development-libraries
- Also get FMOD Engine's HTML5 bindings. These will be copied from
  `api/core/lib/upstream/wasm` and `api/studio/lib/upstream/wasm` into
  `/example/third_party_deps`. Download link for this can be found here:
  https://www.fmod.com/download#fmodengine
- Build repo with with wasm32-unknown-unknown target:
  `cargo build --target wasm32-unknown-unknown`
- Install wasm-bindgen-cl:
  https://rustwasm.github.io/wasm-bindgen/reference/cli.html
- Run wasm-bindgen like so:
  `wasm-bindgen target\wasm32-unknown-unknown\debug\fmod-test-bed.wasm --target no-modules --out-dir example/deps`
- Copy `wasm\wasmfmod.js` to `example\deps\wasmfmod.js`. This isn't a generated
  file, but it isn't possible to serve for testing purposes that can acccess
  server FS from `../`.
- Serve the folder example with some quick serve thing like basic-http-server.
  In case of basic-http-server, it is simple as `basic-http-server example`.
- Load the link in the browser. It requires a click interaction to start.
  I think this is related to FMOD, idk if it can be disabled or not. (or)
  if it is worth the hassle.


# Batch script I used as an example to recompile & execute WASM stuff
```batch
cargo build --target wasm32-unknown-unknown
wasm-bindgen target\wasm32-unknown-unknown\debug\fmod-test-bed.wasm --target no-modules --out-dir example/deps
copy target\wasm32-unknown-unknown\debug\fmod-test-bed.wasm example\deps\fmod-test-bed.wasm
copy wasm\wasmfmod.js example\deps\wasmfmod.js
basic-http-server example
```
