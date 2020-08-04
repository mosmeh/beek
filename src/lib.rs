pub mod interpreter;
pub mod language;
pub mod repl;

#[cfg(target_arch = "wasm32")]
mod wasm;
