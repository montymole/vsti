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
            state_record: Mutex::new(vec![0.1; NUM_PARAMETERS as usize]),
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
            PULSE_WIDTH_MOD_FREQ | PHASE_SHIFT_MOD_FREQ | PITCH_MOD_FREQ | FILTER_CUTOFF =>
                "Hz".to_string(),

            AMP_SUSTAIN_LEVEL | FILTER_SUSTAIN_LEVEL | FILTER_DRIVE => "%".to_string(),

            AMP_ATTACK | AMP_DECAY | AMP_RELEASE | FILTER_ATTACK | FILTER_DECAY | FILTER_RELEASE =>
                "s".to_string(),

            _ => "".to_string(),
        }
    }

    fn get_parameter_text(&self, index: i32) -> String {
        match index as usize {
            NOISE_COLOR => {
                let value = self.state_record.lock().unwrap()[index as usize].round() as u8;
                match value {
                    0 => "white".to_string(),
                    1 => "pink".to_string(),
                    _ => "Invalid value".to_string(),
                }
            }

            PHASE_SHIFT_MOD_SHAPE | PITCH_MOD_SHAPE | FILTER_CUTOFF_MOD_SHAPE => {
                let value = (self.state_record.lock().unwrap()[index as usize] * 4.0).round() as u8;
                match value {
                    0 => "sine".to_string(),
                    1 => "square".to_string(),
                    2 => "triangle".to_string(),
                    3 => "saw".to_string(),
                    4 => "sample and hold".to_string(),
                    _ => "Invalid value".to_string(),
                }
            }

            FILTER_ATTACK | AMP_ATTACK =>
                format!(
                    "{:.2}",
                    f32::abs(
                        self.state_record.lock().unwrap()[index as usize] * MAX_ENV_ATTACK_TIME
                    )
                ),

            FILTER_DECAY | AMP_DECAY =>
                format!(
                    "{:.2}",
                    f32::abs(self.state_record.lock().unwrap()[index as usize] * MAX_ENV_DECAY_TIME)
                ),

            FILTER_SUSTAIN_LEVEL | AMP_SUSTAIN_LEVEL =>
                format!("{:.2}", f32::abs(self.state_record.lock().unwrap()[index as usize])),

            FILTER_RELEASE | AMP_RELEASE =>
                format!(
                    "{:.2}",
                    f32::abs(
                        self.state_record.lock().unwrap()[index as usize] * MAX_ENV_RELEASE_TIME
                    )
                ),

            _ => format!("{:.1}", self.state_record.lock().unwrap()[index as usize] * 100.0),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        (
            match index as usize {
                NOISE_AMP => "Noise",
                NOISE_COLOR => "Noise Color",

                SINE_AMP => "Sine",
                SINE_OCTAVE => "SineOctave",

                PULSE_AMP => "Pulse",
                PULSE_WIDTH => "Pulse width",
                PULSE_WIDTH_MOD_AMP => "Pulse width modulation amplitude",
                PULSE_WIDTH_MOD_FREQ => "Pulse width modulation frequency",

                SAWTOOTH_AMP => "Sawtooth",
                SAWTOOTH_SHAPE => "Sawtooth width",

                PHASE_SHIFT_AMOUNT => "Channel phase shift amount",
                PHASE_SHIFT_MOD_FREQ => "Phase shift modulation frequency",
                PHASE_SHIFT_MOD_SHAPE => "Phase shift modulation wave form",

                PITCH_MOD_SHAPE => "Pitch modulation waveform",
                PITCH_MOD_AMP => "Pitch modulation amplitude",
                PITCH_MOD_FREQ => "Pitch modulation frequency",

                AMP_ATTACK => "Attack",
                AMP_DECAY => "Decay",
                AMP_SUSTAIN_LEVEL => "Sustain",
                AMP_RELEASE => "Release",

                FILTER_ATTACK => "Filter Attack",
                FILTER_DECAY => "Filter Decay",
                FILTER_SUSTAIN_LEVEL => "Fiter Sustain",
                FILTER_RELEASE => "Filter Release",

                FILTER_CUTOFF => "Cutoff",
                FILTER_RESONANCE => "Resonance",
                FILTER_POLES => "Poles",
                FILTER_DRIVE => "Drive",

                FILTER_CUTOFF_MOD_SHAPE => "Cutoff modulation waveform",
                FILTER_CUTOFF_MOD_AMP => "Cutoff modulation amplitude",
                FILTER_CUTOFF_MOD_FREQ => "Cutoff modulation frequency",

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
    fn set_amplitude_control(&self, value: f32) {
        self.state_record.lock().unwrap()[0] = value;

        self.to_dsp.lock().unwrap().send(StateUpdate::SetKnob(0, value)).unwrap();

        self.host.automate(0, value);
    }

    fn set_event_subscription(&self, enabled: bool) {
        self.editor_is_open.store(enabled, Ordering::Relaxed);
    }
}