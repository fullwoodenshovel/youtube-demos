// THIS FILE IS AI GENERATED

// All WASM-specific JS interop lives here.
// The #![cfg(...)] at the top means this entire file is excluded from
// every non-WASM build, so nothing here touches Linux/Windows/Mac.
#![cfg(target_arch = "wasm32")]

use std::sync::Mutex;

// ── Shared state ─────────────────────────────────────────────────────────────

// Input submitted by JS, waiting to be consumed by the main loop.
static PENDING_INPUT: Mutex<Option<String>> = Mutex::new(None);

// Output produced by Rust, waiting to be read by JS.
// We use a fixed-size buffer in .bss so the pointer is always stable.
// 64 KB is far more than any expression result will ever need.
const OUT_BUF_SIZE: usize = 65536;
static OUTPUT: Mutex<StaticString> = Mutex::new(StaticString::new());

// True while graphics() is running (canvas should be visible).
static IN_SHOW_MODE: Mutex<bool> = Mutex::new(false);

// ── Internal helpers (called from parse.rs and main.rs) ──────────────────────

/// Drain any pending input. Returns None when no input is waiting.
pub fn take_input() -> Option<String> {
    PENDING_INPUT.lock().unwrap().take()
}

/// Append a line to the output buffer for JS to read.
pub fn push_output(s: String) {
    let mut out = OUTPUT.lock().unwrap();
    out.push_str(&s);
    out.push('\n');
}

/// Tell JS whether the canvas should be showing.
pub fn set_show_mode(val: bool) {
    *IN_SHOW_MODE.lock().unwrap() = val;
}

// ── Static string helper ──────────────────────────────────────────────────────
// We need a pointer into static memory (not heap) so that web_get_output_ptr
// returns a stable address JS can read from wasm_memory.buffer.

struct StaticString {
    buf: [u8; OUT_BUF_SIZE],
    len: usize,
}

impl StaticString {
    const fn new() -> Self {
        Self { buf: [0u8; OUT_BUF_SIZE], len: 0 }
    }
    fn push_str(&mut self, s: &str) {
        let bytes = s.as_bytes();
        let end = (self.len + bytes.len()).min(OUT_BUF_SIZE);
        self.buf[self.len..end].copy_from_slice(&bytes[..end - self.len]);
        self.len = end;
    }
    fn push(&mut self, c: char) {
        if self.len < OUT_BUF_SIZE {
            self.buf[self.len] = c as u8;
            self.len += 1;
        }
    }
    fn clear(&mut self) {
        self.len = 0;
    }
    fn as_ptr(&self) -> *const u8 {
        self.buf.as_ptr()
    }
    fn len(&self) -> usize {
        self.len
    }
}

// ── Exported functions (called from JavaScript) ───────────────────────────────

/// JS writes the input string into a shared buffer, then calls this with its length.
/// Use web_input_buf_ptr() to get the address to write to.
#[unsafe(no_mangle)]
pub extern "C" fn web_input_buf_ptr() -> *mut u8 {
    // INPUT_BUF is a static array so this pointer is always valid.
    INPUT_BUF.lock().unwrap().as_mut_ptr()
}

/// Signal that the user has submitted a command. `len` is the byte length written
/// into the buffer returned by web_input_buf_ptr().
#[unsafe(no_mangle)]
pub extern "C" fn web_submit_input(len: usize) {
    let buf = INPUT_BUF.lock().unwrap();
    if let Ok(s) = std::str::from_utf8(&buf[..len.min(INPUT_BUF_SIZE)]) {
        *PENDING_INPUT.lock().unwrap() = Some(s.to_string());
    }
}

/// Returns 1 if the visualisation canvas should be visible, 0 otherwise.
#[unsafe(no_mangle)]
pub extern "C" fn web_is_show_mode() -> i32 {
    if *IN_SHOW_MODE.lock().unwrap() { 1 } else { 0 }
}

/// Returns a pointer to the start of the output buffer in WASM memory.
/// Read `web_get_output_len()` bytes from this address, then call web_clear_output().
#[unsafe(no_mangle)]
pub extern "C" fn web_get_output_ptr() -> *const u8 {
    OUTPUT.lock().unwrap().as_ptr()
}

/// Returns the number of bytes currently in the output buffer.
#[unsafe(no_mangle)]
pub extern "C" fn web_get_output_len() -> usize {
    OUTPUT.lock().unwrap().len()
}

/// Clears the output buffer. Call this after JS has finished reading.
#[unsafe(no_mangle)]
pub extern "C" fn web_clear_output() {
    OUTPUT.lock().unwrap().clear();
}

// ── Input scratch buffer ──────────────────────────────────────────────────────
// 4 KB is plenty for any expression.
const INPUT_BUF_SIZE: usize = 4096;
static INPUT_BUF: Mutex<[u8; INPUT_BUF_SIZE]> = Mutex::new([0u8; INPUT_BUF_SIZE]);
