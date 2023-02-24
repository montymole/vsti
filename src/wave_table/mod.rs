
/**
 * table consists of n frames with stereo channels
 *
 */

pub const WAVE_SIZE: usize = 256;
pub const FRAME_COUNT: usize = 256;

#[derive(Debug, Clone)]
struct StereoWave {
    l: Vec<f32>,
    r: Vec<f32>,
}

impl StereoWave {
    pub fn default() -> StereoWave {
        let mut l = Vec::with_capacity(WAVE_SIZE);
        let mut r = Vec::with_capacity(WAVE_SIZE);
        let mut t: f32 = -1.0;
        let s: f32 = 4.0 / 256.0;

        for n in 0..WAVE_SIZE {
            l.push(crate::wave_math::generate_one_cycle_sin(t));
            r.push(crate::wave_math::generate_one_cycle_square(t));
            t += s;
        }

        StereoWave { l, r }
    }

    pub fn sin_to_square(morph: f32) -> StereoWave {
        let mut l = Vec::with_capacity(WAVE_SIZE);
        let mut r = Vec::with_capacity(WAVE_SIZE);
        let mut t: f32 = -1.0;
        let s: f32 = 4.0 / 256.0;

        for n in 0..WAVE_SIZE {
            l.push(
                crate::wave_math::generate_one_cycle_sin(t) * morph +
                    crate::wave_math::generate_one_cycle_square(t) * (1.0 - morph)
            );
            r.push(
                crate::wave_math::generate_one_cycle_sin(t) * morph +
                    crate::wave_math::generate_one_cycle_square(t) * (1.0 - morph)
            );
            t += s;
        }

        StereoWave { l, r }
    }
}

#[derive(Debug, Clone)]
pub struct Wavetable {
    frame_count: usize,
    frames: Vec<StereoWave>,
}

impl Wavetable {
    pub fn default() -> Wavetable {
        let mut frames: Vec<StereoWave> = Vec::with_capacity(FRAME_COUNT);
        let mut m: f32 = 0.0;
        let s: f32 = 1.0 / 256.0;
        for f in 0..FRAME_COUNT {
            frames.push(StereoWave::sin_to_square(m));
            m += s;
        }

        Wavetable {
            frame_count: FRAME_COUNT,
            frames,
        }
    }

    pub fn get_wave(&mut self, time: f32, base_freq: f32, frame: f32, amp: f32) -> (f32, f32) {
        let current_sample = ((time * base_freq * 16.0) as usize) % WAVE_SIZE;
        let mut current_frame: usize = 0;

        if frame == 1.0 {
            current_frame = FRAME_COUNT - 1;
        } else if frame > 0.0 {
            current_frame = ((256.0 * frame) as usize) % FRAME_COUNT;
        }

        (
            self.frames[current_frame].l[current_sample] * amp,
            self.frames[current_frame].r[current_sample] * amp,
        )
    }
}