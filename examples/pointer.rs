#[macro_use]
extern crate wlroots;

use wlroots::{CompositorBuilder, CompositorHandle, Cursor, CursorHandle, CursorHandler,
              InputManagerHandler, KeyboardHandle, KeyboardHandler, OutputBuilder,
              OutputBuilderResult, OutputHandle, OutputHandler, OutputLayout, OutputLayoutHandle,
              OutputLayoutHandler, OutputManagerHandler, PointerHandle, PointerHandler,
              XCursorTheme};
use wlroots::key_events::KeyEvent;
use wlroots::pointer_events::{AxisEvent, ButtonEvent, MotionEvent};
use wlroots::utils::{init_logging, L_DEBUG};
use wlroots::wlroots_sys::gl;
use wlroots::wlroots_sys::wlr_button_state::WLR_BUTTON_RELEASED;
use wlroots::xkbcommon::xkb::keysyms::KEY_Escape;

struct State {
    color: [f32; 4],
    default_color: [f32; 4],
    xcursor_theme: XCursorTheme,
    cursor: CursorHandle,
    layout: OutputLayoutHandle
}

impl State {
    fn new(xcursor_theme: XCursorTheme, layout: OutputLayoutHandle, cursor: CursorHandle) -> Self {
        State { color: [0.25, 0.25, 0.25, 1.0],
                default_color: [0.25, 0.25, 0.25, 1.0],
                xcursor_theme,
                cursor,
                layout }
    }
}

compositor_data!(State);

struct ExCursor;

struct OutputManager;

struct ExOutput;

struct InputManager;

struct ExPointer;

struct ExKeyboardHandler;

struct OutputLayoutEx;

impl CursorHandler for ExCursor {}

impl OutputLayoutHandler for OutputLayoutEx {}

impl OutputManagerHandler for OutputManager {
    fn output_added<'output>(&mut self,
                             compositor: CompositorHandle,
                             builder: OutputBuilder<'output>)
                             -> Option<OutputBuilderResult<'output>> {
        let mut result = builder.build_best_mode(ExOutput);
        with_handles!([(compositor: {compositor})] => {
            let state: &mut State = compositor.into();
            let layout = &mut state.layout;
            let cursor = &mut state.cursor;
            let xcursor = state.xcursor_theme
                .get_cursor("left_ptr".into())
                .expect("Could not load left_ptr cursor");
            let image = &xcursor.images()[0];
            // TODO use output config if present instead of auto
            with_handles!([(layout: {layout}),
                          (cursor: {cursor}),
                          (output: {&mut result.output})] => {
                layout.add_auto(output);
                cursor.attach_output_layout(layout);
                cursor.set_cursor_image(image);
                let (x, y) = cursor.coords();
                // https://en.wikipedia.org/wiki/Mouse_warping
                cursor.warp(None, x, y);
            }).unwrap();
            Some(result)
        }).unwrap()
    }
}

impl KeyboardHandler for ExKeyboardHandler {
    fn on_key(&mut self, compositor: CompositorHandle, _: KeyboardHandle, key_event: &KeyEvent) {
        with_handles!([(compositor: {compositor})] => {
            for key in key_event.pressed_keys() {
                if key == KEY_Escape {
                    compositor.terminate()
                }
            }
        }).unwrap();
    }
}

impl PointerHandler for ExPointer {
    fn on_motion(&mut self, compositor: CompositorHandle, _: PointerHandle, event: &MotionEvent) {
        with_handles!([(compositor: {compositor})] => {
            let state: &mut State = compositor.into();
            let (delta_x, delta_y) = event.delta();
            state.cursor
                .run(|cursor| cursor.move_to(None, delta_x, delta_y))
                .unwrap();
        }).unwrap();
    }

    fn on_button(&mut self, compositor: CompositorHandle, _: PointerHandle, event: &ButtonEvent) {
        with_handles!([(compositor: {compositor})] => {
            let state: &mut State = compositor.into();
            if event.state() == WLR_BUTTON_RELEASED {
                state.color = state.default_color;
            } else {
                state.color = [0.25, 0.25, 0.25, 1.0];
                state.color[event.button() as usize % 3] = 1.0;
            }
        }).unwrap();
    }

    fn on_axis(&mut self, compositor: CompositorHandle, _: PointerHandle, event: &AxisEvent) {
        with_handles!([(compositor: {compositor})] => {
            let state: &mut State = compositor.into();
            for color_byte in &mut state.default_color[..3] {
                *color_byte += if event.delta() > 0.0 { -0.05 } else { 0.05 };
                if *color_byte > 1.0 {
                    *color_byte = 1.0
                }
                if *color_byte < 0.0 {
                    *color_byte = 0.0
                }
            }
            state.color = state.default_color.clone()
        }).unwrap();
    }
}

impl OutputHandler for ExOutput {
    fn on_frame(&mut self, compositor: CompositorHandle, output: OutputHandle) {
        with_handles!([(compositor: {compositor}), (output: {output})] => {
            let state: &mut State = compositor.into();
            // NOTE gl functions will probably always be unsafe.
            unsafe {
                output.make_current();
                gl::ClearColor(state.color[0], state.color[1], state.color[2], 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
                output.swap_buffers(None, None);
            }
        }).unwrap();
    }
}

impl InputManagerHandler for InputManager {
    fn pointer_added(&mut self,
                     compositor: CompositorHandle,
                     pointer: PointerHandle)
                     -> Option<Box<PointerHandler>> {
        with_handles!([(compositor: {compositor}), (pointer: {pointer})] => {
            let state: &mut State = compositor.into();
            state.cursor
                .run(|cursor| cursor.attach_input_device(pointer.input_device()))
                .unwrap();
        }).unwrap();
        Some(Box::new(ExPointer))
    }

    fn keyboard_added(&mut self,
                      _: CompositorHandle,
                      _: KeyboardHandle)
                      -> Option<Box<KeyboardHandler>> {
        Some(Box::new(ExKeyboardHandler))
    }
}

fn main() {
    init_logging(L_DEBUG, None);
    let cursor = Cursor::create(Box::new(ExCursor));
    let xcursor_theme = XCursorTheme::load_theme(None, 16).expect("Could not load theme");
    let layout = OutputLayout::create(Box::new(OutputLayoutEx));

    let compositor = CompositorBuilder::new().input_manager(Box::new(InputManager))
                                             .output_manager(Box::new(OutputManager))
                                             .build_auto(State::new(xcursor_theme, layout, cursor));
    compositor.run();
}
