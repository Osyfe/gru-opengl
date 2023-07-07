rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
cargo install wasm-opt
cargo install basic-http-server

no srgb conversion
Browsers on mobile devices emit click and cursor rather than touch events, and the latter often does not work as expected.
Browsers do not like audio to be initialized before the user has triggered some input event and may block it completely otherwise.