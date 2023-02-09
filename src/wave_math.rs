use std::f32::consts::PI;

pub const TAU: f32 = PI * 2.0;


pub fn parameter_to_u8(value:f32, steps:i32) -> u8 {
    (value * steps as f32).round() as u8
}
/// Convert the midi note's pitch into the equivalent frequency.
///
/// This function assumes A4 is 440hz.
pub fn midi_pitch_to_freq(pitch: u8) -> f32 {
    const A4_PITCH: i8 = 69;
    const A4_FREQ: f32 = 440.0;
    // Midi notes can be 0-127
    (f32::from((pitch as i8) - A4_PITCH) / 12.0).exp2() * A4_FREQ
}


pub fn generate_one_cycle_sin(t:f32) -> f32 {
    (t*TAU).sin()
}

pub fn generate_one_cycle_square(t:f32) -> f32 {
    if t > 0.0 { 1.0 } else { -1.0 }
}

pub fn generate_sine_wave(time: f32, base_freq: f32, amp: f32) -> f32 {
    (time * TAU * base_freq).sin() * amp
}

pub fn _generate_pulse_treshold(value: f32, tresh: f32) -> f32 {
    if value > tresh { 1.0 } else if value < tresh { -1.0 } else { 0.0 }
}

pub fn generate_square_wave(time: f32, base_freq: f32, amp: f32) -> f32 {
    let period: f32 = 1.0 / base_freq;
    let t: f32 = time % period;
    if t < period / 2.0 {
        amp
    } else {
        -amp
    }
}

pub fn generate_pulse_wave(time: f32, base_freq: f32, pulse_width: f32, amp: f32) -> f32 {
    let period = 1.0 / base_freq;
    let t = time % period;
    if t < pulse_width * period {
        amp
    } else {
        -amp
    }
}

pub fn generate_triangle_wave(time: f32, base_freq: f32, amp: f32) -> f32 {
    let period = 1.0 / base_freq;
    let t = time % period;
    if t < period / 2.0 {
        (t / (period / 2.0)) * amp
    } else {
        ((period - t) / (period / 2.0)) * amp
    }
}

pub fn generate_sawtooth_wave(time: f32, base_freq: f32, sawtooth_width: f32, amp: f32) -> f32 {
    let period: f32 = 1.0 / base_freq;
    let t = time % period;
    if t < sawtooth_width * period {
        0.0
    } else {
        ((2.0 * t) / period - 1.0) * amp
    }
}

pub fn phase_shifted_time(time: f32, base_freq: f32, fraction: f32) -> f32 {
    time + fraction / base_freq
}

pub fn scale_to_range(value: f32, range: f32, max_amp_abs: f32) -> f32 {
    value * (range / max_amp_abs)
}

pub fn generate_white_noise(amp: f32) -> f32 {
    (rand::random::<f32>() - 0.5) * amp
}

pub fn lfo(shape:u8, time: f32, base_freq: f32, amp: f32) -> f32 {
    match shape {
        0 => generate_sine_wave(time, base_freq, amp),
        1 => generate_square_wave(time, base_freq, amp),
        2 => generate_triangle_wave(time, base_freq, amp),
        3 => generate_sawtooth_wave(time, base_freq, 0.0, amp),
        _ => 0.0
    }
}

