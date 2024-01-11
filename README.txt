A small Rust framework for building games that run both on desktop and the web with the same code. It integrates windowing + inputs (via "winit"), graphics (via "glow" which maps to OpenGL 2.0 or WebGL 1, respectively), audio (via "rodio"), resource loading and ui (both self written). It aims to be easy to set up while being cross-platform and to run on older devices as well (therefore the old OpenGL versions).

For compiling and testing web build you need to install:

rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
cargo install wasm-opt
cargo install basic-http-server

Notes:

no srgb conversion anywhere (due to lack of support for that in WebGL 1)
Browsers on mobile devices emit click and cursor rather than touch events, and the latter often does not work as expected.
Browsers do not like audio to be initialized before the user has triggered some input event and may block it completely otherwise (this is handled by this library).
