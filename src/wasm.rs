use crate::repl;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    colored::control::set_override(true);
}

#[wasm_bindgen]
#[derive(Debug, Copy, Clone)]
pub enum ResponseKind {
    Message,
    Clear,
    Reset,
}

#[wasm_bindgen]
pub struct Response {
    message: String,
    #[wasm_bindgen(readonly)]
    pub kind: ResponseKind,
}

#[wasm_bindgen]
impl Response {
    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        self.message.clone()
    }
}

#[wasm_bindgen]
pub struct Repl {
    inner: repl::Repl,
}

#[wasm_bindgen]
impl Repl {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: repl::Repl::new(),
        }
    }

    pub fn run(&mut self, input: &str) -> Response {
        match self.inner.run(input) {
            repl::Response::Empty => Response {
                message: "".into(),
                kind: ResponseKind::Message,
            },
            repl::Response::Message(msg) => Response {
                message: msg.into(),
                kind: ResponseKind::Message,
            },
            repl::Response::ClearScreen => Response {
                message: "".into(),
                kind: ResponseKind::Clear,
            },
            repl::Response::Quit => {
                self.inner = repl::Repl::new();

                Response {
                    message: "".into(),
                    kind: ResponseKind::Reset,
                }
            }
        }
    }

    #[wasm_bindgen(getter)]
    pub fn completion_candidates(&self) -> Vec<JsValue> {
        self.inner
            .completion_candidates()
            .map(JsValue::from_str)
            .collect()
    }
}
