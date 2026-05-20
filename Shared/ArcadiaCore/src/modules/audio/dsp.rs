//! Generic DSP primitives. Each node has a stateless params struct and a stateful
//! runner that produces one sample per `tick`. No node knows about specific use
//! cases; the [`graph`](super::graph) layer wires them.

use std::f32::consts::PI;

const TWO_PI: f32 = 2.0 * PI;

/// Tiny deterministic xorshift PRNG used for noise + voice-trigger variation. We
/// don't want `rand` in this hot path — this is two `xor`s and a shift.
#[derive(Clone, Copy, Debug)]
pub struct Rng(pub u32);

impl Rng {
    pub fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        if x == 0 {
            x = 0x9E37_79B9;
        }
        self.0 = x;
        x
    }
    /// Uniform `[-1, 1)` float.
    pub fn next_signed(&mut self) -> f32 {
        let u = (self.next_u32() >> 8) as f32;
        (u / (1u32 << 24) as f32) * 2.0 - 1.0
    }
}

// ─── Noise burst ─────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug)]
pub enum NoiseShape {
    Exp,
    Linear,
    Flat,
}

#[derive(Clone, Copy, Debug)]
pub struct NoiseBurst {
    pub duration_samples: u32,
    pub shape: NoiseShape,
}

pub struct NoiseBurstState {
    cfg: NoiseBurst,
    elapsed: u32,
    rng: Rng,
}

impl NoiseBurstState {
    pub fn new(cfg: NoiseBurst, seed: u32) -> Self {
        let s = if seed == 0 { 0xDEAD_BEEF } else { seed };
        Self {
            cfg,
            elapsed: 0,
            rng: Rng(s),
        }
    }
    pub fn tick(&mut self) -> f32 {
        if self.elapsed >= self.cfg.duration_samples {
            return 0.0;
        }
        let t = self.elapsed as f32 / self.cfg.duration_samples as f32;
        let env = match self.cfg.shape {
            NoiseShape::Exp => (-4.0 * t).exp(),
            NoiseShape::Linear => 1.0 - t,
            NoiseShape::Flat => 1.0,
        };
        self.elapsed += 1;
        self.rng.next_signed() * env
    }
}

// ─── Biquad (bandpass / lowpass / highpass) ──────────────────────────────────

#[derive(Clone, Copy, Debug)]
pub enum BiquadMode {
    Bandpass,
    Lowpass,
    Highpass,
}

#[derive(Clone, Copy, Debug)]
pub struct BiquadCfg {
    pub mode: BiquadMode,
    pub freq_hz: f32,
    pub q: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct BiquadCoeffs {
    pub b0: f32,
    pub b1: f32,
    pub b2: f32,
    pub a1: f32,
    pub a2: f32,
}

impl BiquadCfg {
    pub fn compute(&self, sample_rate: u32) -> BiquadCoeffs {
        let f = self.freq_hz.clamp(20.0, sample_rate as f32 * 0.45);
        let q = self.q.max(0.001);
        let w0 = TWO_PI * f / sample_rate as f32;
        let cos_w = w0.cos();
        let sin_w = w0.sin();
        let alpha = sin_w / (2.0 * q);
        let a0 = 1.0 + alpha;
        match self.mode {
            BiquadMode::Bandpass => BiquadCoeffs {
                b0: alpha / a0,
                b1: 0.0,
                b2: -alpha / a0,
                a1: -2.0 * cos_w / a0,
                a2: (1.0 - alpha) / a0,
            },
            BiquadMode::Lowpass => {
                let one_minus_cos = 1.0 - cos_w;
                BiquadCoeffs {
                    b0: (one_minus_cos / 2.0) / a0,
                    b1: one_minus_cos / a0,
                    b2: (one_minus_cos / 2.0) / a0,
                    a1: -2.0 * cos_w / a0,
                    a2: (1.0 - alpha) / a0,
                }
            }
            BiquadMode::Highpass => {
                let one_plus_cos = 1.0 + cos_w;
                BiquadCoeffs {
                    b0: (one_plus_cos / 2.0) / a0,
                    b1: -one_plus_cos / a0,
                    b2: (one_plus_cos / 2.0) / a0,
                    a1: -2.0 * cos_w / a0,
                    a2: (1.0 - alpha) / a0,
                }
            }
        }
    }
}

#[derive(Default)]
pub struct BiquadState {
    pub coeffs: BiquadCoeffs,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl BiquadState {
    pub fn new(coeffs: BiquadCoeffs) -> Self {
        Self {
            coeffs,
            ..Default::default()
        }
    }
    pub fn tick(&mut self, x: f32) -> f32 {
        let c = self.coeffs;
        let y = c.b0 * x + c.b1 * self.x1 + c.b2 * self.x2 - c.a1 * self.y1 - c.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

impl Default for BiquadCoeffs {
    fn default() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
        }
    }
}

// ─── Transient click ─────────────────────────────────────────────────────────
//
// Short bright pulse: high-passed noise modulated by a fast decay. Used for the
// contact-click portion of a keypress recipe.

#[derive(Clone, Copy, Debug)]
pub struct Transient {
    pub freq_hz: f32,
    pub duration_samples: u32,
}

pub struct TransientState {
    cfg: Transient,
    elapsed: u32,
    rng: Rng,
    hp: BiquadState,
}

impl TransientState {
    pub fn new(cfg: Transient, sample_rate: u32, seed: u32) -> Self {
        let coeffs = BiquadCfg {
            mode: BiquadMode::Highpass,
            freq_hz: cfg.freq_hz,
            q: 0.7071,
        }
        .compute(sample_rate);
        let s = if seed == 0 { 0xCAFE_F00D } else { seed.wrapping_mul(2654435761) };
        Self {
            cfg,
            elapsed: 0,
            rng: Rng(s),
            hp: BiquadState::new(coeffs),
        }
    }
    pub fn tick(&mut self) -> f32 {
        if self.elapsed >= self.cfg.duration_samples {
            return 0.0;
        }
        let t = self.elapsed as f32 / self.cfg.duration_samples as f32;
        let env = (-8.0 * t).exp();
        self.elapsed += 1;
        self.hp.tick(self.rng.next_signed() * env)
    }
}

// ─── AR envelope ─────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug)]
pub struct EnvAr {
    pub attack_samples: u32,
    pub release_samples: u32,
}

pub struct EnvArState {
    cfg: EnvAr,
    elapsed: u32,
}

impl EnvArState {
    pub fn new(cfg: EnvAr) -> Self {
        Self { cfg, elapsed: 0 }
    }
    pub fn finished(&self) -> bool {
        self.elapsed >= self.cfg.attack_samples + self.cfg.release_samples
    }
    pub fn level(&self) -> f32 {
        let a = self.cfg.attack_samples;
        let r = self.cfg.release_samples;
        if self.elapsed < a {
            if a == 0 {
                1.0
            } else {
                self.elapsed as f32 / a as f32
            }
        } else if self.elapsed < a + r {
            if r == 0 {
                0.0
            } else {
                1.0 - (self.elapsed - a) as f32 / r as f32
            }
        } else {
            0.0
        }
    }
    pub fn tick(&mut self, x: f32) -> f32 {
        let y = x * self.level();
        self.elapsed = self.elapsed.saturating_add(1);
        y
    }
}

// ─── Mix ─────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct Mix {
    pub gains: Vec<f32>,
}

impl Mix {
    pub fn tick(&self, inputs: &[f32]) -> f32 {
        let n = inputs.len().min(self.gains.len());
        let mut sum = 0.0;
        for i in 0..n {
            sum += inputs[i] * self.gains[i];
        }
        sum
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noise_burst_decays_to_zero_after_duration() {
        let mut n = NoiseBurstState::new(
            NoiseBurst {
                duration_samples: 100,
                shape: NoiseShape::Exp,
            },
            1,
        );
        for _ in 0..100 {
            let _ = n.tick();
        }
        let after = n.tick();
        assert_eq!(after, 0.0);
    }

    #[test]
    fn biquad_lowpass_passes_dc() {
        let coeffs = BiquadCfg {
            mode: BiquadMode::Lowpass,
            freq_hz: 1000.0,
            q: 0.7071,
        }
        .compute(48_000);
        let mut b = BiquadState::new(coeffs);
        let mut last = 0.0;
        for _ in 0..2000 {
            last = b.tick(1.0);
        }
        assert!((last - 1.0).abs() < 0.05);
    }

    #[test]
    fn env_ar_finishes_at_attack_plus_release() {
        let mut e = EnvArState::new(EnvAr {
            attack_samples: 4,
            release_samples: 8,
        });
        for _ in 0..12 {
            assert!(!e.finished());
            let _ = e.tick(1.0);
        }
        assert!(e.finished());
    }

    #[test]
    fn rng_is_deterministic_for_seed() {
        let mut a = Rng(42);
        let mut b = Rng(42);
        for _ in 0..32 {
            assert_eq!(a.next_u32(), b.next_u32());
        }
    }
}
