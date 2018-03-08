use std::collections::{HashMap, HashSet};
use std::mem;
use piston::input::{Button, ButtonState, Input, Motion};

use {Clock, Span};

/// message type Handler sends to User App
#[derive(Clone, Debug)]
pub struct InputMessage {
    pub buttons: Vec<ButtonMessage>,
    pub focused: bool,
    pub mouse_xy: Option<(f64, f64)>,
    pub mouse_scroll: (f64, f64),
}

#[derive(Clone, Debug)]
pub struct ButtonMessage {
    pub button: Button,
    pub handle: ButtonHandle,
}

impl ButtonMessage {
    fn press(b: Button, c: Clock) -> ButtonMessage {
        ButtonMessage {
            button: b,
            handle: ButtonHandle::Press(c),
        }
    }
    fn release(b: Button, s: Span) -> ButtonMessage {
        ButtonMessage {
            button: b,
            handle: ButtonHandle::Release(s),
        }
    }
}

/// state of button
/// ButtonState is used in Piston, so we have to use an alternative name
#[derive(Clone, Copy, Debug)]
pub enum ButtonHandle {
    Press(Clock),
    Release(Span),
}

#[derive(Debug, Default)]
pub struct InputHandler {
    /// global settings (not reset by update)
    /// window is active or not
    focused: bool,
    /// list of presed buttons with first press time
    pressed_buttons: HashMap<Button, Clock>,
    /// Coordinate of mouse
    mouse_xy: Option<(f64, f64)>,

    /// settings reset by update
    /// list of released buttons
    released_buttons: Vec<(Button, Span)>,
    mouse_scroll: (f64, f64),
}

impl InputHandler {
    fn reset_by_upd(&mut self) {
        self.mouse_scroll = (0.0, 0.0);
    }
    fn press_button(&mut self, b: Button, clock: Clock) {
        self.pressed_buttons.entry(b).or_insert(clock);
    }
    fn release_button(&mut self, b: Button, current: Clock) {
        let span = if let Some(pressed) = self.pressed_buttons.remove(&b) {
            Span::new(pressed, current)
        } else {
            warn!("not pressed button released! {:?}", b);
            Span::new(current - 3, current)
        };
        self.released_buttons.push((b, span));
    }
    pub fn get_message(&mut self) -> InputMessage {
        let mut v = vec![];
        mem::swap(&mut self.released_buttons, &mut v);
        let buttons = self.pressed_buttons
            .iter()
            .map(|(&b, &c)| ButtonMessage::press(b, c))
            .chain(v.into_iter().map(|(b, s)| ButtonMessage::release(b, s)));
        InputMessage {
            buttons: buttons.collect(),
            focused: self.focused,
            mouse_xy: self.mouse_xy,
            mouse_scroll: self.mouse_scroll,
        }
    }
    pub fn handle(&mut self, input: Input, clock: Clock) {
        match input {
            Input::Button(args) => {
                let b = args.button;
                match args.state {
                    ButtonState::Press => {
                        self.press_button(b, clock);
                    }
                    ButtonState::Release => {
                        self.release_button(b, clock);
                    }
                }
            }
            Input::Move(motion) => {
                if !self.focused {
                    return;
                }
                match motion {
                    Motion::MouseCursor(x, y) => {
                        self.mouse_xy = Some((x, y));
                    }
                    Motion::MouseScroll(x, y) => {
                        self.mouse_scroll.0 += x;
                        self.mouse_scroll.1 += y;
                    }
                    // TODO: joystick support
                    Motion::ControllerAxis(_args) => {}
                    // TODO: touch event support
                    Motion::Touch(_args) => {}
                    _ => {}
                }
            }
            _ => {}
        }
    }
}
