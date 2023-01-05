//! The plugin's digital signal processing is fully implemented within this module.
//!
//! All updates to input parameters are received through message passing to avoid thread locking
//! during audio processing. In particular, note that parameter smoothing is considered within the
//! scope of audio processing rather than state management. This module uses the `SmoothedRange`
//! struct to ensure that parameters are consistently and efficiently interpolated while minimizing
//! the number of messages passed.

use crate::{ plugin_state::StateUpdate, * };
use std::{ sync::mpsc::Receiver };
use crate::{ wave_math::* };
use vst::{ buffer::AudioBuffer };

#[derive(Debug, Clone, PartialEq)]
enum VoiceState {
    Off,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Debug, Clone)]
struct Voice {
    state: VoiceState,
    note: u8,
    amplitude: f32,
}

impl Voice {
    fn default() -> Voice {
        Voice { state: VoiceState::Off, note: 0, amplitude: 0.0 }
    }
}
/// Handles all audio processing algorithms for the plugin.
pub(super) struct PluginDsp {
    sample_rate: f32,
    time: f32,
    voices: Vec<Voice>, //state, note, duration, amplitude
    parameter: Vec<f32>,
    messages_from_params: Receiver<StateUpdate>
}

impl PluginDsp {
    pub fn new(incoming_messages: Receiver<StateUpdate>) -> Self {
        Self {
            time: 0.0,
            sample_rate: 44100.0,
            voices: vec![Voice::default(); NUM_VOICES as usize],
            parameter: vec![0.0; NUM_PARAMETERS as usize],
            messages_from_params: incoming_messages
        }
    }

    pub fn set_sample_rate(&mut self, rate: f32) {
        self.sample_rate = rate;
    }

    pub fn note_on(&mut self, note: u8) {
        // find if same note is already playing
        for i in 1..self.voices.len() {
            if self.voices[i].note == note && self.voices[i].state != VoiceState::Off {
                self.voices[i].state = VoiceState::Attack; // return to attack phase
                return;
            }
        }
        // find free note slot
        for i in 1..self.voices.len() {
            if self.voices[i].state == VoiceState::Off {
                self.voices[i].state = VoiceState::Attack; // goto attack phase
                self.voices[i].note = note;
                self.voices[i].amplitude = 0.0; // amplitude to 0
                break;
            }
        }
    }

    pub fn note_off(&mut self, note: u8) {
        for i in 1..self.voices.len() {
            if self.voices[i].note == note && self.voices[i].state != VoiceState::Off {
                self.voices[i].state = VoiceState::Release;
                break;
            }
        }
    }

    fn adsr_for_voice(&mut self, i: usize) -> f32 {
        let time_per_sample = 1.0 / self.sample_rate;

        match self.voices[i].state {
            VoiceState::Off => 0.0, //do nothing
            VoiceState::Attack => {
                // grow volume slope /
                if self.voices[i].amplitude < 1.0 {
                    let attack_time = self.parameter[AMP_ATTACK as usize] * MAX_ENV_ATTACK_TIME;
                    let slope_up: f32 = time_per_sample / attack_time;
                    self.voices[i].amplitude += slope_up;
                } else {
                    // attack complete, set state to decay
                    self.voices[i].state = VoiceState::Decay;
                    self.voices[i].amplitude = 1.0;
                }
                self.voices[i].amplitude
            }
            VoiceState::Decay => {
                let sustain_level: f32 = self.parameter[AMP_SUSTAIN_LEVEL as usize];
                if self.voices[i].amplitude > sustain_level {
                    // reduce volume slope \
                    let decay_time: f32 = self.parameter[AMP_DECAY as usize] * MAX_ENV_DECAY_TIME;
                    let slope_down: f32 = time_per_sample / decay_time;
                    self.voices[i].amplitude -= slope_down;
                } else {
                    // decay done
                    self.voices[i].state = VoiceState::Sustain;
                }
                self.voices[i].amplitude
            }
            VoiceState::Sustain => {
                let sustain_level: f32 = self.parameter[AMP_SUSTAIN_LEVEL as usize];
                self.voices[i].amplitude = sustain_level;
                self.voices[i].amplitude
            }
            VoiceState::Release => {
                if self.voices[i].amplitude > 0.0 {
                    let release_time = self.parameter[AMP_RELEASE as usize] * MAX_ENV_RELEASE_TIME;
                    let slope_down: f32 = time_per_sample / release_time;
                    self.voices[i].amplitude -= slope_down;
                } else {
                    // voice done
                    self.voices[i].amplitude = 0.0;
                    self.voices[i].state = VoiceState::Off;
                }
                self.voices[i].amplitude
            }
        }
    }

    pub fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        // First, get any new changes to parameter ranges.
        while let Ok(message) = self.messages_from_params.try_recv() {
            match message {
                StateUpdate::SetKnob(index, value) => {
                    self.parameter[index as usize] = value;
                }
                StateUpdate::NoteOn(n) => self.note_on(n),
                StateUpdate::NoteOff(n) => self.note_off(n),
            }
        }

        let samples = buffer.samples();
        let (_, mut outputs) = buffer.split();

        let time_per_sample = 1.0 / self.sample_rate;

        for sample_idx in 0..samples {
            // get modulation controls
            let noise_amp: f32 = self.parameter[NOISE_AMP];
            let noise_color = self.parameter[NOISE_COLOR].round() as u8;

            let sine_amp: f32 = self.parameter[SINE_AMP];

            let pulse_width = self.parameter[PULSE_WIDTH];
            let pulse_width_mod: f32 = self.parameter[PULSE_WIDTH_MOD_AMP];
            let pulse_width_mod_freq: f32 = self.parameter[PULSE_WIDTH_MOD_FREQ];
            let pulse_amp: f32 = self.parameter[PULSE_AMP];

            let sawtooth_width: f32 = self.parameter[SAWTOOTH_SHAPE];
            let sawtooth_amp: f32 = self.parameter[SAWTOOTH_AMP];

            let pitch_mod: f32 = self.parameter[PITCH_MOD_AMP];
            let pitch_mod_freq: f32 = self.parameter[PITCH_MOD_FREQ];

            let stereo_width: f32 = self.parameter[PHASE_SHIFT_AMOUNT];
            let stereo_mod: f32 = self.parameter[PHASE_SHIFT_MOD_FREQ];

            let phase_modulator: f32 = (self.time * stereo_mod * 100.0).sin() + 1.0;
            let pulse_width_modulator: f32 =
                (self.time * pulse_width_mod_freq).sin() * pulse_width_mod;
            let pitch_modulator: f32 = (self.time * pitch_mod_freq * 100.0).sin() * pitch_mod;

            // mutated between channels
            let mut channel_phase_shift_amount: f32 = 0.0;

            let noise = if noise_amp <= 0.0 {
                0.0
            } else {
                match noise_color {
                    1 => generate_pink_noise(noise_amp),
                    _ => generate_white_noise(noise_amp),
                }
            };

            for output_idx in 0..outputs.len() {
                let mut signal = 0.0;
                let mut max_signal = 1.0;
                for i in 1..self.voices.len() {
                    if self.voices[i].state != VoiceState::Off {
                        signal += noise;

                        let base_freq =
                            midi_pitch_to_freq(self.voices[i].note) * 2.0 + pitch_modulator;
                        let time = phase_shifted_time(
                            self.time,
                            base_freq,
                            channel_phase_shift_amount * phase_modulator
                        );

                        signal += generate_sine_wave(time, base_freq, sine_amp);

                        signal += generate_pulse_wave(
                            time,
                            base_freq,
                            pulse_width + pulse_width_modulator,
                            pulse_amp
                        );

                        signal += generate_sawtooth_wave(
                            time,
                            base_freq,
                            sawtooth_width,
                            sawtooth_amp
                        );

                        signal *= self.adsr_for_voice(i);

                        max_signal += 1.0; // each active voise adds range
                    }
                }
                let buff = outputs.get_mut(output_idx);
                buff[sample_idx] = scale_to_range(signal, 1.0, max_signal);
                channel_phase_shift_amount += stereo_width; // introduce timeshift between channels
            }
            self.time += time_per_sample;
        }
    }
}