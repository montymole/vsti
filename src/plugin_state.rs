//! Parameters are kept as the single "source of truth" for the long-term state of the plugin. As
//! used by the VST API, the parameter bank is accessible by both the audio processing thread and
//! the UI thread, and updated using thread-safe interior mutability. However, to avoid costly
//! synchronization overhead, and to reduce recalculation of derived parameters, the audio
//! processing and UI threads subscribe to parameter updates through cross-thread message passing.
//!
//! This plugin's long-term state only consists of a single floating-point value (the value of the
//! amplitude knob), but it should be simple to extend this scheme to work with multiple knobs,
//! toggles, node locations, waveforms, user-defined labels, and so on.

use std::sync::{ atomic::{ AtomicBool, Ordering }, mpsc::Sender, Mutex };

use vst::{ host::Host, plugin::{ HostCallback, PluginParameters } };

use crate::*;

/// Describes a discrete operation that can update this plugin's long-term state.
#[derive(Clone)]
pub enum StateUpdate {
    SetKnob(i32, f32),
    NoteOn(u8),
    NoteOff(u8),
}

pub struct PluginState {
    host: HostCallback,
    to_dsp: Mutex<Sender<StateUpdate>>,
    to_editor: Mutex<Sender<StateUpdate>>,
    editor_is_open: AtomicBool,
    state_record: Mutex<Vec<f32>>,
}

/// VST-accessible long-term plugin state storage. This is accessed through the audio processing
/// thread and the UI thread, so all fields are protected by thread-safe interior mutable
/// constructs.
impl PluginState {
    pub fn new(
        host: HostCallback,
        to_dsp: Sender<StateUpdate>,
        to_editor: Sender<StateUpdate>
    ) -> Self {
        Self {
            host,
            to_dsp: Mutex::new(to_dsp),
            to_editor: Mutex::new(to_editor),
            editor_is_open: AtomicBool::new(false),
            state_record: Mutex::new(vec![1.0, 0.5, 0.5, 0.5, 0.5, 0.1, 0.1, 0.8, 0.2]),
        }
    }
}

/// The DAW directly accesses the plugin state through the VST API to get reports on knob states.
impl PluginParameters for PluginState {
    fn set_parameter(&self, index: i32, value: f32) {
        let state_update = StateUpdate::SetKnob(index, value);
        if self.editor_is_open.load(Ordering::Relaxed) {
            self.to_editor.lock().unwrap().send(state_update.clone()).unwrap();
        }
        self.to_dsp.lock().unwrap().send(state_update).unwrap();
        self.state_record.lock().unwrap()[index as usize] = value;
    }

    fn get_parameter(&self, index: i32) -> f32 {
        self.state_record.lock().unwrap()[index as usize]
    }

    fn get_parameter_label(&self, index: i32) -> String {
        match index as usize {
            _ => "".to_string(),
        }
    }

    fn get_parameter_text(&self, index: i32) -> String {
        match index as usize {
            _ => format!("{:.1}", self.state_record.lock().unwrap()[index as usize] * 100.0),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        (
            match index as usize {
                WAVE_TABLE_AMP => "Wave table",
                SHAPE_ROT_X => "Rotate X",
                SHAPE_ROT_Y => "Rotate Y",
                SHAPE_ROT_Z => "Rotate Z",
                SHAPE_MORPH => "Shape morph",

                AMP_ATTACK => "Attack",
                AMP_DECAY => "Decay",
                AMP_SUSTAIN_LEVEL => "Sustain",
                AMP_RELEASE => "Release",

                _ => "Unknown",
            }
        ).to_string()
    }

    fn string_to_parameter(&self, index: i32, text: String) -> bool {
        dbg!("Set string to parameter for {}, {}", index, &text);
        match index {
            0 =>
                match text.parse::<f32>() {
                    Ok(value) if value <= 1.0 && value >= 0.0 => {
                        self.set_parameter(index, value);
                        true
                    }
                    _ => false,
                }
            _ => unreachable!(),
        }
    }
}

/// The editor interface also directly accesses the plugin state through its own API.
impl crate::editor::EditorRemoteState for PluginState {
    fn set_knob_control(&self, index: i32, value: f32) {
        self.state_record.lock().unwrap()[index as usize] = value;
        self.to_dsp.lock().unwrap().send(StateUpdate::SetKnob(index, value)).unwrap();
        self.host.automate(index, value);
    }

    fn set_event_subscription(&self, enabled: bool) {
        self.editor_is_open.store(enabled, Ordering::Relaxed);
    }
}