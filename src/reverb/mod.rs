use nih_plug::nih_log;

use self::filters::{AllPass, Comb};

mod filters;

const MIX_MATRIX: [[i8; 2]; 2] = [[1, 1], [-1, -1]];

const DELAY_LENGTHS_ALLPASS: [usize; 4] = [556, 441, 341, 225];
const DELAY_LENGTHS_COMB: [usize; 8] = [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617];

fn adjust_length(length: usize, sr: usize) -> usize {
    (length as f64 * sr as f64 / 44100.0) as usize
}

pub struct Reverb {
    wet: f32,
    dry: f32,
    room_size: f32,
    dampening: f32,
    comb_filters: [Comb; DELAY_LENGTHS_COMB.len()],
    allpass_filters: [AllPass; DELAY_LENGTHS_ALLPASS.len()],
}

impl Reverb {
    pub fn new() -> Self {
        Self {
            wet: 0.0,
            dry: 0.0,
            room_size: 0.0,
            dampening: 0.0,
            comb_filters: DELAY_LENGTHS_COMB.map(|x| filters::Comb::new(x)),
            allpass_filters: DELAY_LENGTHS_ALLPASS.map(|x| filters::AllPass::new(x)),
        }
    }

    fn update(&mut self) {
        self.comb_filters.iter_mut().for_each(|comb| {
            comb.set_feedback(self.room_size as f64);
            comb.set_dampening(self.dampening as f64)
        })
    }

    pub fn set_room_size(&mut self, room_size: f32) {
        self.room_size = room_size;
        self.update();
    }

    pub fn set_dampening(&mut self, dampening: f32) {
        self.dampening = dampening;
        self.update();
    }

    pub fn set_wet(&mut self, wet: f32) {
        self.wet = wet;
    }

    pub fn set_dry(&mut self, dry: f32) {
        self.wet = dry;
    }

    pub fn process(&mut self, samples: (f32, f32)) -> (f32, f32) {
        let sample = (samples.0 + samples.1) / 2.0;
        let allpass_combined_out = self
            .allpass_filters
            .iter_mut()
            .fold(sample, |acc: f32, f: &mut AllPass| {
                f.tick(acc as f64) as f32
            }) as f64;

        let comb_outs: Vec<f64> = self
            .comb_filters
            .iter_mut()
            .map(|f| f.tick(allpass_combined_out as f64))
            .collect();

        let mut outs: [f64; 2] = [0.0; 2];
        for row_index in 0..MIX_MATRIX.len() {
            let row = MIX_MATRIX[row_index];
            for j in 0..row.len() {
                outs[row_index] = comb_outs[j] * row[j] as f64;
            }
        }
        let mixed_l = (samples.0 * self.dry + outs[0] as f32 * self.wet) / 2.0;
        let mixed_r = (samples.1 * self.dry + outs[1] as f32 * self.wet) / 2.0;
        (mixed_l, mixed_r)
    }
}
