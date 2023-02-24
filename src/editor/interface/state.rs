//! The editor interface is scheduled to be drawn periodically by the host DAW. Some state must be
//! kept to maintain a consistent appearance across frames. This module contains the
//! `InterfaceState` struct along with logic to update it in response to window events like clicks,
//! drags, etc. as well as from external state updates.

use vst_window::WindowEvent;

use super::{ image_consts::{ ORIG_KNOB_RADIUS, ORIG_KNOB_X, ORIG_KNOB_Y }, SCALE, SIZE_X, SIZE_Y };
use crate::{
    plugin_state::StateUpdate,
    SHAPE_ROT_X,
    SHAPE_ROT_Y,
    SHAPE_ROT_Z,
    SHAPE_MORPH,
    WAVE_TABLE_AMP,
};

/// All the possible ways a click+drag operation on the interface window might be interpreted.
enum DragBehavior {
    TurnAmplitudeKnob {
        click_y: isize,
        original_value: f32,
    },
}

/// Holds any state required to render and update the editor interface.
pub(in crate::editor) struct InterfaceState {
    /// Represents the position of the knob, from 0 to 1.
    pub amplitude_value: f32,
    pub rotation_x_value: f32,
    pub rotation_y_value: f32,
    pub rotation_z_value: f32,
    pub morph_value: f32,
    /// (X, Y) pixel coordinate of the cursor, from the top-left corner.
    /// Coordinates could be negative if the cursor is dragged outside of the window!
    cursor_pos: (isize, isize),
    drag_behavior: Option<DragBehavior>,
    note: Option<u8>,
}

const KNOB_CENTER_X: usize = ((ORIG_KNOB_X as f64) * SCALE) as usize;
const KNOB_CENTER_Y: usize = ((ORIG_KNOB_Y as f64) * SCALE) as usize;
const KNOB_RADIUS: usize = ((ORIG_KNOB_RADIUS as f64) * SCALE) as usize;

const KNOB_CHANGE_SPEED: f32 = 0.5;

impl InterfaceState {
    pub fn new() -> Self {
        Self {
            amplitude_value: Default::default(),
            rotation_x_value: Default::default(),
            rotation_y_value: Default::default(),
            rotation_z_value: Default::default(),
            morph_value: Default::default(),
            cursor_pos: Default::default(),
            drag_behavior: None,
            note: None,
        }
    }
    /// Update the editor state in response to an external message.
    pub fn react_to_control_event(&mut self, event: StateUpdate) {
        match event {
            StateUpdate::SetKnob(index, value) => {
                match index as usize {
                    WAVE_TABLE_AMP => {
                        self.amplitude_value = value;
                    }
                    SHAPE_ROT_X => {
                        self.rotation_x_value = value;
                    }
                    SHAPE_ROT_Y => {
                        self.rotation_y_value = value;
                    }
                    SHAPE_ROT_Z => {
                        self.rotation_z_value = value;
                    }
                    SHAPE_MORPH => {
                        self.morph_value = value;
                    }
                    _ => (),
                }
            }
            StateUpdate::NoteOn(n) => {
                self.note = Some(n);
            }
            StateUpdate::NoteOff(n) => {
                self.note = Some(n);
            }
        }
    }

    /// Update the editor state and remote state store as necessary in response to an interaction
    /// with the editor window.
    pub fn react_to_window_event<S: super::EditorRemoteState>(
        &mut self,
        event: WindowEvent,
        remote_state: &S
    ) {
        match event {
            WindowEvent::CursorMovement(x, y) => {
                self.cursor_pos = ((x * (SIZE_X as f32)) as isize, (y * (SIZE_Y as f32)) as isize);
                if
                    let Some(DragBehavior::TurnAmplitudeKnob { click_y, original_value, .. }) =
                        self.drag_behavior
                {
                    let diff_y = click_y - self.cursor_pos.1;
                    self.amplitude_value = (
                        original_value +
                        ((diff_y as f32) / (SIZE_Y as f32)) * KNOB_CHANGE_SPEED
                    )
                        .max(0.0)
                        .min(1.0);
                    remote_state.set_knob_control(0,self.amplitude_value);
                }
            }
            WindowEvent::MouseClick(button) => {
                let (x, y) = self.cursor_pos;
                if
                    (x - (KNOB_CENTER_X as isize)).pow(2) + (y - (KNOB_CENTER_Y as isize)).pow(2) <
                    (KNOB_RADIUS.pow(2) as isize)
                {
                    if button == vst_window::MouseButton::Left {
                        self.drag_behavior = Some(DragBehavior::TurnAmplitudeKnob {
                            click_y: y,
                            original_value: self.amplitude_value,
                        });
                    } else if button == vst_window::MouseButton::Right {
                        self.amplitude_value = 0.5;
                        remote_state.set_knob_control(0,self.amplitude_value);
                    }
                }
            }
            WindowEvent::MouseRelease(vst_window::MouseButton::Left) => {
                drop(self.drag_behavior.take());
            }
            _ => (),
        }
    }
}