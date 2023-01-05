//! ampli-Fe is a minimal yet complete VST2 plugin designed to demonstrate usage of the
//! `vst_window` crate.
//!
//! It features a fully-customized editor UI with an interactive knob and corresponding numerical
//! value readout.
//!
//! ampli-Fe's code is well-documented - feel free to use it as a starting point for your next VST2
//! plugin in Rust.

use std::sync::{ mpsc::channel, Arc };

use vst::{
    event::Event,
    api::{ Supported, Events },
    buffer::AudioBuffer,
    editor::Editor,
    plugin::{ CanDo, HostCallback, Info, Plugin, PluginParameters, Category },
};

mod dsp;
use dsp::PluginDsp;

mod editor;
use editor::PluginEditor;

mod plugin_state;
use plugin_state::PluginState;

mod wave_math;

pub const NUM_VOICES: i32 = 12;
pub const NUM_PARAMETERS: i32 = 31;

// parameter indexes

// noise
pub const NOISE_AMP: usize = 0;
pub const NOISE_COLOR: usize = 1;

// sine waveform
pub const SINE_AMP: usize = 2;
pub const SINE_OCTAVE: usize = 3;

// pwm waveform
pub const PULSE_AMP: usize = 4;
pub const PULSE_WIDTH: usize = 5;
pub const PULSE_WIDTH_MOD_AMP: usize = 6;
pub const PULSE_WIDTH_MOD_FREQ: usize = 7;

// sawtooth waveform
pub const SAWTOOTH_AMP: usize = 8;
pub const SAWTOOTH_SHAPE: usize = 9;

// stereo modulators
pub const PHASE_SHIFT_MOD_SHAPE: usize = 10;
pub const PHASE_SHIFT_AMOUNT: usize = 11;
pub const PHASE_SHIFT_MOD_FREQ: usize = 12;

// pitch modulators
pub const PITCH_MOD_SHAPE: usize = 13;
pub const PITCH_MOD_AMP: usize = 14;
pub const PITCH_MOD_FREQ: usize = 15;

// amp envelope
pub const AMP_ATTACK: usize =16;
pub const AMP_DECAY: usize =17;
pub const AMP_SUSTAIN_LEVEL: usize =18;
pub const AMP_RELEASE: usize =19;

// filter envelope
pub const FILTER_ATTACK: usize = 20;
pub const FILTER_DECAY: usize = 21;
pub const FILTER_SUSTAIN_LEVEL: usize = 22;
pub const FILTER_RELEASE: usize = 23;
// filter modifiers
pub const FILTER_CUTOFF: usize =24;
pub const FILTER_RESONANCE:usize = 25;
pub const FILTER_POLES:usize = 26;
pub const FILTER_DRIVE:usize = 27;

pub const FILTER_CUTOFF_MOD_SHAPE: usize = 28;
pub const FILTER_CUTOFF_MOD_AMP: usize = 29;
pub const FILTER_CUTOFF_MOD_FREQ: usize = 30;

// values
pub const MIN_ENV_ATTACK_TIME: f32 = 0.001; // prevent pop
pub const MAX_ENV_ATTACK_TIME: f32 = 1.0;
pub const MAX_ENV_DECAY_TIME: f32 = 2.0;
pub const MAX_ENV_RELEASE_TIME: f32 = 1.0;

/// Top level wrapper that exposes a full `vst::Plugin` implementation.
struct MachineElf {
    /// The `PluginDsp` handles all of the plugin's audio processing, and is only accessed from the
    /// audio processing thread.
    dsp: PluginDsp,

    /// The `PluginEditor` implements the plugin's custom editor interface. It's temporarily stored
    /// here until being moved to the UI thread by the first `get_editor` method call.
    editor_placeholder: Option<PluginEditor>,

    /// The `PluginState` holds the long-term state of the plugin and distributes raw parameter
    /// updates as they occur to other parts of the plugin. It is shared on both the audio
    /// processing thread and the UI thread, and updated using thread-safe interior mutability.
    state_handle: Arc<PluginState>,
}

impl MachineElf {
    /// Initializes the VST plugin, along with an optional `HostCallback` handle.
    fn new_maybe_host(maybe_host: Option<HostCallback>) -> Self {
        let host = maybe_host.unwrap_or_default();

        let (to_editor, editor_recv) = channel();
        let (to_dsp, dsp_recv) = channel();

        let state_handle = Arc::new(PluginState::new(host, to_dsp, to_editor));

        let editor_placeholder = Some(PluginEditor::new(Arc::clone(&state_handle), editor_recv));

        let dsp = PluginDsp::new(dsp_recv);

        log::debug!("Initialized plugin");

        Self {
            dsp,
            state_handle,
            editor_placeholder,
        }
    }

    /// Process an incoming midi event.
    ///
    /// The midi data is split up like so:
    ///
    /// `data[0]`: Contains the status and the channel. Source: [source]
    /// `data[1]`: Contains the supplemental data for the message - so, if this was a NoteOn then
    ///            this would contain the note.
    /// `data[2]`: Further supplemental data. Would be velocity in the case of a NoteOn message.
    ///
    /// [source]: http://www.midimountain.com/midi/midi_status.htm
    fn process_midi_event(&mut self, data: [u8; 3]) {
        match data[0] {
            128 => self.note_off(data[1]),
            144 => self.note_on(data[1]),
            _ => (),
        }
    }

    fn note_on(&mut self, note: u8) {
        self.dsp.note_on(note);
    }

    fn note_off(&mut self, note: u8) {
        self.dsp.note_off(note);
    }
}

/// `vst::plugin_main` requires a `Default` implementation.
impl Default for MachineElf {
    fn default() -> Self {
        Self::new_maybe_host(None)
    }
}

fn init_logger() {
    #[cfg(target_os = "windows")]
    {
        let level = env_logger::builder().build().filter();

        if let Some(level) = level.to_level() {
            windebug_logger::init_with_level(level).unwrap_or(());
        }
    }

    env_logger::try_init().unwrap_or(());
}

/// Main `vst` plugin implementation.
impl Plugin for MachineElf {
    fn new(host: HostCallback) -> Self {
        init_logger();

        Self::new_maybe_host(Some(host))
    }

    fn set_sample_rate(&mut self, rate: f32) {
        self.dsp.set_sample_rate(rate);
    }

    fn get_info(&self) -> Info {
        /// Use a hash of a string describing this plugin to avoid unique ID conflicts.
        const UNIQUE_ID_SEED: &str = "machineElf VST2 Plugin";
        static UNIQUE_ID: once_cell::sync::Lazy<i32> = once_cell::sync::Lazy::new(|| {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{ Hash, Hasher };

            let mut s = DefaultHasher::new();
            UNIQUE_ID_SEED.hash(&mut s);
            s.finish() as i32
        });

        Info {
            name: "MachineElf".to_string(),
            vendor: "JLFO".to_string(),
            unique_id: *UNIQUE_ID,
            category: Category::Synth,
            inputs: 0,
            outputs: 2,
            parameters: NUM_PARAMETERS,
            initial_delay: 0,
            preset_chunks: false,
            ..Info::default()
        }
    }

    #[allow(unused_variables)]
    #[allow(clippy::single_match)]
    fn process_events(&mut self, events: &Events) {
        for event in events.events() {
            match event {
                Event::Midi(ev) => self.process_midi_event(ev.data),
                // More events can be handled here.
                _ => (),
            }
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        self.dsp.process(buffer);
    }

    fn can_do(&self, _can_do: CanDo) -> Supported {
        Supported::Maybe
    }

    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.state_handle) as Arc<dyn PluginParameters>
    }

    /*
    fn get_editor(&mut self) -> Option<Box<dyn Editor>> {
        self.editor_placeholder.take().map(|editor| Box::new(editor) as Box<dyn Editor>)
    }
    */
}

vst::plugin_main!(MachineElf);