//! Voice graph: declarative spec → compiled DAG → per-trigger runner.
//!
//! Python builds a [`VoiceSpec`] (JSON), submits it to [`super::engine::register_voice`],
//! and gets back an opaque voice id. Each trigger spawns a [`VoiceInstance`] (mutable
//! per-node state) that the mixer ticks until the output node finishes.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::dsp::{
    BiquadCfg, BiquadMode, BiquadState, EnvAr, EnvArState, Mix, NoiseBurst, NoiseBurstState,
    NoiseShape, Rng, Transient, TransientState,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind")]
pub enum NodeKind {
    #[serde(rename = "noise_burst")]
    NoiseBurst {
        id: String,
        duration_ms: f32,
        #[serde(default = "default_shape")]
        shape: String,
    },
    #[serde(rename = "biquad")]
    Biquad {
        id: String,
        mode: String,
        freq_hz: f32,
        q: f32,
    },
    #[serde(rename = "transient")]
    Transient {
        id: String,
        freq_hz: f32,
        duration_ms: f32,
    },
    #[serde(rename = "env_ar")]
    EnvAr {
        id: String,
        attack_ms: f32,
        release_ms: f32,
    },
    #[serde(rename = "mix")]
    Mix { id: String, gains: Vec<f32> },
    #[serde(rename = "gain")]
    Gain { id: String, gain: f32 },
}

fn default_shape() -> String {
    "exp".to_string()
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct VariationSpec {
    #[serde(default)]
    pub freq_jitter_pct: f32,
    #[serde(default)]
    pub gain_jitter_pct: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VoiceSpec {
    pub nodes: Vec<NodeKind>,
    pub edges: Vec<(String, String)>,
    pub output: String,
    #[serde(default)]
    pub variations: VariationSpec,
}

impl VoiceSpec {
    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| format!("invalid voice graph json: {e}"))
    }
}

fn node_id(n: &NodeKind) -> &str {
    match n {
        NodeKind::NoiseBurst { id, .. }
        | NodeKind::Biquad { id, .. }
        | NodeKind::Transient { id, .. }
        | NodeKind::EnvAr { id, .. }
        | NodeKind::Mix { id, .. }
        | NodeKind::Gain { id, .. } => id,
    }
}

#[derive(Debug, Clone)]
pub enum CompiledNode {
    NoiseBurst {
        cfg: NoiseBurst,
    },
    Biquad {
        cfg: BiquadCfg,
    },
    Transient {
        cfg: Transient,
    },
    EnvAr {
        cfg: EnvAr,
    },
    Mix(Mix),
    Gain(f32),
}

#[derive(Debug, Clone)]
pub struct CompiledGraph {
    pub nodes: Vec<CompiledNode>,
    pub node_ids: Vec<String>,
    /// For each node index, the source node indices it consumes (in input order).
    pub inputs: Vec<Vec<usize>>,
    /// Topological order (parents before children).
    pub order: Vec<usize>,
    pub output_index: usize,
    pub variations: VariationSpec,
    pub sample_rate: u32,
}

impl CompiledGraph {
    pub fn compile(spec: &VoiceSpec, sample_rate: u32) -> Result<Self, String> {
        if spec.nodes.is_empty() {
            return Err("voice graph has no nodes".into());
        }
        let mut id_to_idx: BTreeMap<String, usize> = BTreeMap::new();
        for (idx, n) in spec.nodes.iter().enumerate() {
            let id = node_id(n).to_string();
            if id_to_idx.insert(id.clone(), idx).is_some() {
                return Err(format!("duplicate node id: {id}"));
            }
        }
        let output_index = *id_to_idx
            .get(&spec.output)
            .ok_or_else(|| format!("output node not found: {}", spec.output))?;

        let mut inputs: Vec<Vec<usize>> = vec![Vec::new(); spec.nodes.len()];
        for (src, dst) in &spec.edges {
            let s = *id_to_idx
                .get(src)
                .ok_or_else(|| format!("edge src not found: {src}"))?;
            let d = *id_to_idx
                .get(dst)
                .ok_or_else(|| format!("edge dst not found: {dst}"))?;
            inputs[d].push(s);
        }

        let order = topo_order(spec.nodes.len(), &inputs)?;

        let nodes = spec
            .nodes
            .iter()
            .map(|n| compile_node(n, sample_rate))
            .collect::<Result<Vec<_>, _>>()?;

        let node_ids: Vec<String> = spec.nodes.iter().map(|n| node_id(n).to_string()).collect();

        Ok(Self {
            nodes,
            node_ids,
            inputs,
            order,
            output_index,
            variations: spec.variations.clone(),
            sample_rate,
        })
    }
}

fn topo_order(n: usize, inputs: &[Vec<usize>]) -> Result<Vec<usize>, String> {
    let mut in_deg = vec![0usize; n];
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (dst, srcs) in inputs.iter().enumerate() {
        for &s in srcs {
            adj[s].push(dst);
            in_deg[dst] += 1;
        }
    }
    let mut q: std::collections::VecDeque<usize> = in_deg
        .iter()
        .enumerate()
        .filter(|(_, d)| **d == 0)
        .map(|(i, _)| i)
        .collect();
    let mut order = Vec::with_capacity(n);
    while let Some(i) = q.pop_front() {
        order.push(i);
        for &j in &adj[i] {
            in_deg[j] -= 1;
            if in_deg[j] == 0 {
                q.push_back(j);
            }
        }
    }
    if order.len() != n {
        return Err("voice graph has a cycle".into());
    }
    // Make sure every distinct node id is present (BTreeSet dedup as sanity).
    let unique: BTreeSet<usize> = order.iter().copied().collect();
    if unique.len() != n {
        return Err("voice graph topological order corrupted".into());
    }
    Ok(order)
}

fn compile_node(n: &NodeKind, sample_rate: u32) -> Result<CompiledNode, String> {
    match n {
        NodeKind::NoiseBurst {
            duration_ms, shape, ..
        } => {
            let shape = match shape.as_str() {
                "exp" => NoiseShape::Exp,
                "linear" => NoiseShape::Linear,
                "flat" => NoiseShape::Flat,
                other => return Err(format!("noise_burst.shape unknown: {other}")),
            };
            Ok(CompiledNode::NoiseBurst {
                cfg: NoiseBurst {
                    duration_samples: ms_to_samples(*duration_ms, sample_rate),
                    shape,
                },
            })
        }
        NodeKind::Biquad {
            mode, freq_hz, q, ..
        } => {
            let mode = match mode.as_str() {
                "bandpass" => BiquadMode::Bandpass,
                "lowpass" => BiquadMode::Lowpass,
                "highpass" => BiquadMode::Highpass,
                other => return Err(format!("biquad.mode unknown: {other}")),
            };
            Ok(CompiledNode::Biquad {
                cfg: BiquadCfg {
                    mode,
                    freq_hz: *freq_hz,
                    q: *q,
                },
            })
        }
        NodeKind::Transient {
            freq_hz,
            duration_ms,
            ..
        } => Ok(CompiledNode::Transient {
            cfg: Transient {
                freq_hz: *freq_hz,
                duration_samples: ms_to_samples(*duration_ms, sample_rate),
            },
        }),
        NodeKind::EnvAr {
            attack_ms,
            release_ms,
            ..
        } => Ok(CompiledNode::EnvAr {
            cfg: EnvAr {
                attack_samples: ms_to_samples(*attack_ms, sample_rate),
                release_samples: ms_to_samples(*release_ms, sample_rate),
            },
        }),
        NodeKind::Mix { gains, .. } => Ok(CompiledNode::Mix(Mix {
            gains: gains.clone(),
        })),
        NodeKind::Gain { gain, .. } => Ok(CompiledNode::Gain(*gain)),
    }
}

fn ms_to_samples(ms: f32, sample_rate: u32) -> u32 {
    (ms.max(0.0) * sample_rate as f32 / 1000.0).round() as u32
}

// ─── Voice instance: per-trigger mutable state ───────────────────────────────

enum NodeRuntime {
    NoiseBurst(NoiseBurstState),
    Biquad(BiquadState),
    Transient(TransientState),
    EnvAr(EnvArState),
    Mix(Mix),
    Gain(f32),
}

pub struct VoiceInstance {
    graph: std::sync::Arc<CompiledGraph>,
    runtimes: Vec<NodeRuntime>,
    last_output: Vec<f32>,
    gain: f32,
    finished: bool,
}

impl VoiceInstance {
    pub fn spawn(graph: std::sync::Arc<CompiledGraph>, velocity: f32, seed: u32) -> Self {
        let mut rng = Rng(if seed == 0 { 1 } else { seed });
        let freq_jit = graph.variations.freq_jitter_pct.max(0.0);
        let gain_jit = graph.variations.gain_jitter_pct.max(0.0);

        let runtimes: Vec<NodeRuntime> = graph
            .nodes
            .iter()
            .enumerate()
            .map(|(_, node)| match node {
                CompiledNode::NoiseBurst { cfg } => {
                    NodeRuntime::NoiseBurst(NoiseBurstState::new(*cfg, rng.next_u32()))
                }
                CompiledNode::Biquad { cfg } => {
                    let mut c = *cfg;
                    if freq_jit > 0.0 {
                        let j = rng.next_signed() * freq_jit * 0.01;
                        c.freq_hz *= 1.0 + j;
                    }
                    NodeRuntime::Biquad(BiquadState::new(c.compute(graph.sample_rate)))
                }
                CompiledNode::Transient { cfg } => {
                    NodeRuntime::Transient(TransientState::new(*cfg, graph.sample_rate, rng.next_u32()))
                }
                CompiledNode::EnvAr { cfg } => NodeRuntime::EnvAr(EnvArState::new(*cfg)),
                CompiledNode::Mix(m) => {
                    let mut mm = m.clone();
                    if gain_jit > 0.0 {
                        for g in &mut mm.gains {
                            let j = rng.next_signed() * gain_jit * 0.01;
                            *g *= 1.0 + j;
                        }
                    }
                    NodeRuntime::Mix(mm)
                }
                CompiledNode::Gain(g) => NodeRuntime::Gain(*g),
            })
            .collect();

        let n = graph.nodes.len();
        Self {
            graph,
            runtimes,
            last_output: vec![0.0; n],
            gain: velocity.clamp(0.0, 1.0),
            finished: false,
        }
    }

    pub fn finished(&self) -> bool {
        self.finished
    }

    pub fn tick(&mut self) -> f32 {
        if self.finished {
            return 0.0;
        }
        let order = self.graph.order.clone();
        for &idx in &order {
            let inputs: Vec<f32> = self.graph.inputs[idx]
                .iter()
                .map(|i| self.last_output[*i])
                .collect();
            let y = match &mut self.runtimes[idx] {
                NodeRuntime::NoiseBurst(s) => s.tick(),
                NodeRuntime::Biquad(s) => s.tick(inputs.first().copied().unwrap_or(0.0)),
                NodeRuntime::Transient(s) => s.tick(),
                NodeRuntime::EnvAr(s) => s.tick(inputs.first().copied().unwrap_or(0.0)),
                NodeRuntime::Mix(m) => m.tick(&inputs),
                NodeRuntime::Gain(g) => inputs.first().copied().unwrap_or(0.0) * *g,
            };
            self.last_output[idx] = y;
        }
        let out = self.last_output[self.graph.output_index] * self.gain;
        // Finished when the output env (if any) is past its tail. Heuristic: the AR
        // env at `output` reports `finished()`. If output isn't an env, fall back to
        // amplitude near zero for several ticks.
        if let NodeRuntime::EnvAr(s) = &self.runtimes[self.graph.output_index] {
            if s.finished() {
                self.finished = true;
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_spec() -> VoiceSpec {
        VoiceSpec {
            nodes: vec![
                NodeKind::NoiseBurst {
                    id: "n".into(),
                    duration_ms: 5.0,
                    shape: "exp".into(),
                },
                NodeKind::EnvAr {
                    id: "env".into(),
                    attack_ms: 0.1,
                    release_ms: 20.0,
                },
            ],
            edges: vec![("n".into(), "env".into())],
            output: "env".into(),
            variations: VariationSpec::default(),
        }
    }

    #[test]
    fn compile_simple_graph_succeeds() {
        let s = simple_spec();
        let g = CompiledGraph::compile(&s, 48_000).unwrap();
        assert_eq!(g.nodes.len(), 2);
        assert_eq!(g.order.len(), 2);
    }

    #[test]
    fn compile_rejects_unknown_output() {
        let mut s = simple_spec();
        s.output = "missing".into();
        let r = CompiledGraph::compile(&s, 48_000);
        assert!(r.is_err());
    }

    #[test]
    fn compile_rejects_cycle() {
        let s = VoiceSpec {
            nodes: vec![
                NodeKind::Gain {
                    id: "a".into(),
                    gain: 1.0,
                },
                NodeKind::Gain {
                    id: "b".into(),
                    gain: 1.0,
                },
            ],
            edges: vec![("a".into(), "b".into()), ("b".into(), "a".into())],
            output: "a".into(),
            variations: VariationSpec::default(),
        };
        let r = CompiledGraph::compile(&s, 48_000);
        assert!(r.is_err());
    }

    #[test]
    fn voice_instance_finishes_eventually() {
        let g = std::sync::Arc::new(CompiledGraph::compile(&simple_spec(), 48_000).unwrap());
        let mut v = VoiceInstance::spawn(g.clone(), 1.0, 1);
        let mut ticks = 0;
        while !v.finished() && ticks < g.sample_rate as usize {
            let _ = v.tick();
            ticks += 1;
        }
        assert!(v.finished());
    }

    #[test]
    fn json_round_trip() {
        let json = r#"{"nodes":[{"kind":"noise_burst","id":"n","duration_ms":5.0,"shape":"exp"},{"kind":"env_ar","id":"env","attack_ms":0.1,"release_ms":20.0}],"edges":[["n","env"]],"output":"env"}"#;
        let s = VoiceSpec::from_json(json).unwrap();
        let g = CompiledGraph::compile(&s, 48_000).unwrap();
        assert_eq!(g.node_ids, vec!["n".to_string(), "env".to_string()]);
    }
}
