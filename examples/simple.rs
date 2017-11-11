#[macro_use]
extern crate wlroots;

use std::time::Instant;

use wlroots::{Compositor, InputDevice, KeyEvent};
use wlroots::{InputManagerHandler, KeyboardHandler, OutputHandler, OutputManagerHandler};
use wlroots::types::{KeyboardHandle, OutputHandle};
use wlroots::types::server_decoration::ServerDecorationMode;
use wlroots::wlroots_sys::gl;
use wlroots::xkbcommon::xkb::keysyms::KEY_Escape;

struct Output {
    color: [f32; 3],
    dec: usize,
    last_frame: Instant
}

struct OutputManager;

struct InputManager;
struct ExKeyboardHandler;

impl KeyboardHandler for ExKeyboardHandler {
    fn on_key(&mut self, keyboard: &mut KeyboardHandle, key_event: &mut KeyEvent) {
        let keys = key_event.input_keys();

        wlr_log!(L_DEBUG,
                 "Got key event. Keys: {:?}. Modifiers: {}",
                 keys,
                 keyboard.get_modifiers());

        for key in keys {
            if key == KEY_Escape {
                wlroots::terminate()
            }
        }
    }
}

impl InputManagerHandler for InputManager {
    fn keyboard_added(&mut self, _: &mut InputDevice) -> Option<Box<KeyboardHandler>> {
        Some(Box::new(ExKeyboardHandler))
    }
}

impl OutputManagerHandler for OutputManager {
    fn output_added(&mut self, output: &mut OutputHandle) -> Option<Box<OutputHandler>> {
        output.choose_best_mode();
        Some(Box::new(Output {
                          color: [0.0, 0.0, 0.0],
                          dec: 0,
                          last_frame: Instant::now()
                      }))
    }
}

impl OutputHandler for Output {
    fn output_frame(&mut self, output: &mut OutputHandle) {
        let now = Instant::now();
        let delta = now.duration_since(self.last_frame);
        let seconds_delta = delta.as_secs();
        let nano_delta = delta.subsec_nanos() as u64;
        let ms = (seconds_delta * 1000) + nano_delta / 1000000;
        let inc = (self.dec + 1) % 3;
        self.color[inc as usize] += ms as f32 / 2000.0;
        self.color[self.dec as usize] -= ms as f32 / 2000.0;

        if self.color[self.dec as usize] < 0.0 {
            self.color[inc as usize] = 1.0;
            self.color[self.dec as usize] = 0.0;
            self.dec = inc;
        }
        self.last_frame = now;
        // NOTE gl functions will probably always be unsafe.
        output.make_current();
        unsafe {
            gl::ClearColor(self.color[0], self.color[1], self.color[2], 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        output.swap_buffers()
    }
}

fn main() {
    let input_manager = InputManager;
    let output_manager = OutputManager;
    let mut compositor = Compositor::new(Box::new(input_manager), Box::new(output_manager));

    if let Some(ref mut decoration_manager) = compositor.server_decoration_manager() {
        decoration_manager.set_default_mode(ServerDecorationMode::Client);
    }
    compositor.run();
}
