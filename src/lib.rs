//! # agent-resonance
//!
//! When agents operate in a fleet, they develop natural operating frequencies.
//! Two agents that resonate at the same frequency amplify each other's output
//! (constructive interference). When they're out of phase, they cancel
//! (destructive interference). This crate models that phenomenon.

use std::collections::HashMap;

/// An agent's natural operating frequency in Hz.
#[derive(Debug, Clone, PartialEq)]
pub struct ResonanceFrequency {
    pub agent_id: String,
    /// Primary frequency in Hz (e.g. how often the agent produces output).
    pub hz: f64,
    /// Phase offset in radians [0, 2π).
    pub phase: f64,
    /// Amplitude [0.0, 1.0].
    pub amplitude: f64,
}

impl ResonanceFrequency {
    pub fn new(agent_id: impl Into<String>, hz: f64, phase: f64, amplitude: f64) -> Self {
        Self {
            agent_id: agent_id.into(),
            hz,
            phase: phase % (2.0 * std::f64::consts::PI),
            amplitude: amplitude.clamp(0.0, 1.0),
        }
    }

    /// Compute the signal value at time `t` (seconds): A * sin(2π*f*t + φ).
    pub fn sample(&self, t: f64) -> f64 {
        self.amplitude * (2.0 * std::f64::consts::PI * self.hz * t + self.phase).sin()
    }

    /// How close two frequencies are (0.0 = identical, 1.0 = maximally different).
    pub fn frequency_distance(&self, other: &ResonanceFrequency) -> f64 {
        let diff = (self.hz - other.hz).abs();
        let max_hz = self.hz.max(other.hz).max(1.0);
        (diff / max_hz).min(1.0)
    }

    /// Phase difference in radians [0, π].
    pub fn phase_difference(&self, other: &ResonanceFrequency) -> f64 {
        let diff = (self.phase - other.phase).abs() % (2.0 * std::f64::consts::PI);
        diff.min(2.0 * std::f64::consts::PI - diff)
    }
}

/// Two agents forming a resonance pair.
#[derive(Debug, Clone)]
pub struct ResonancePair {
    pub agent_a: ResonanceFrequency,
    pub agent_b: ResonanceFrequency,
}

impl ResonancePair {
    pub fn new(a: ResonanceFrequency, b: ResonanceFrequency) -> Self {
        Self { agent_a: a, agent_b: b }
    }

    /// Whether this pair is in constructive resonance (close frequency, small phase diff).
    pub fn is_constructive(&self, freq_tolerance: f64, phase_tolerance: f64) -> bool {
        let freq_close = self.agent_a.frequency_distance(&self.agent_b) < freq_tolerance;
        let phase_aligned = self.agent_a.phase_difference(&self.agent_b) < phase_tolerance;
        freq_close && phase_aligned
    }

    /// Whether this pair is in destructive resonance (close frequency, large phase diff).
    pub fn is_destructive(&self, freq_tolerance: f64) -> bool {
        let freq_close = self.agent_a.frequency_distance(&self.agent_b) < freq_tolerance;
        let phase_near_pi = self.agent_a.phase_difference(&self.agent_b)
            > std::f64::consts::PI - 0.5;
        freq_close && phase_near_pi
    }

    /// Combined amplitude at time t (superposition).
    pub fn combined_signal(&self, t: f64) -> f64 {
        self.agent_a.sample(t) + self.agent_b.sample(t)
    }
}

/// Result of constructive interference — amplified output.
#[derive(Debug, Clone)]
pub struct ConstructiveInterference {
    pub pair: ResonancePair,
    /// The amplification factor (≥ 1.0 for constructive).
    pub amplification: f64,
    /// The effective combined frequency.
    pub combined_hz: f64,
}

impl ConstructiveInterference {
    pub fn from_pair(pair: ResonancePair) -> Self {
        let a = &pair.agent_a;
        let b = &pair.agent_b;
        let combined_hz = (a.hz + b.hz) / 2.0;

        // Sample over one period to find peak amplitude
        let period = if combined_hz > 0.0 { 1.0 / combined_hz } else { 1.0 };
        let steps = 100;
        let mut max_signal = 0.0f64;
        let mut sum_signal = 0.0f64;
        for i in 0..steps {
            let t = period * (i as f64) / (steps as f64);
            let s = pair.combined_signal(t).abs();
            max_signal = max_signal.max(s);
            sum_signal += s;
        }
        let avg_signal = sum_signal / steps as f64;
        let individual_avg = (a.amplitude + b.amplitude) / 2.0;
        let amplification = if individual_avg > 0.0 {
            avg_signal / individual_avg
        } else {
            1.0
        };

        Self {
            pair,
            amplification: amplification.max(1.0),
            combined_hz,
        }
    }
}

/// Result of destructive interference — cancelled output.
#[derive(Debug, Clone)]
pub struct DestructiveInterference {
    pub pair: ResonancePair,
    /// The cancellation factor (0.0 = complete cancellation, 1.0 = no cancellation).
    pub cancellation: f64,
}

impl DestructiveInterference {
    pub fn from_pair(pair: ResonancePair) -> Self {
        let period = if pair.agent_a.hz > 0.0 {
            1.0 / pair.agent_a.hz
        } else {
            1.0
        };
        let steps = 100;
        let mut sum_abs = 0.0f64;
        for i in 0..steps {
            let t = period * (i as f64) / (steps as f64);
            sum_abs += pair.combined_signal(t).abs();
        }
        let avg = sum_abs / steps as f64;
        let max_possible = pair.agent_a.amplitude + pair.agent_b.amplitude;
        let cancellation = if max_possible > 0.0 {
            avg / max_possible
        } else {
            1.0
        };

        Self {
            pair,
            cancellation: cancellation.min(1.0),
        }
    }
}

/// A harmonic in the spectrum.
#[derive(Debug, Clone)]
pub struct Harmonic {
    pub frequency_hz: f64,
    pub amplitude: f64,
    pub agent_ids: Vec<String>,
}

/// Frequency analysis of a fleet of agents.
#[derive(Debug, Clone)]
pub struct ResonanceSpectrum {
    pub harmonics: Vec<Harmonic>,
    pub agents: Vec<ResonanceFrequency>,
}

impl ResonanceSpectrum {
    pub fn new(agents: Vec<ResonanceFrequency>) -> Self {
        let harmonics = agents.iter()
            .map(|a| Harmonic {
                frequency_hz: a.hz,
                amplitude: a.amplitude,
                agent_ids: vec![a.agent_id.clone()],
            })
            .collect();

        Self { harmonics, agents }
    }

    /// Find all resonance pairs in the fleet.
    pub fn resonance_pairs(&self) -> Vec<ResonancePair> {
        let mut pairs = Vec::new();
        for i in 0..self.agents.len() {
            for j in (i + 1)..self.agents.len() {
                pairs.push(ResonancePair::new(
                    self.agents[i].clone(),
                    self.agents[j].clone(),
                ));
            }
        }
        pairs
    }

    /// Detect constructive resonance pairs.
    pub fn find_constructive(&self, freq_tolerance: f64, phase_tolerance: f64) -> Vec<ConstructiveInterference> {
        self.resonance_pairs()
            .into_iter()
            .filter(|p| p.is_constructive(freq_tolerance, phase_tolerance))
            .map(ConstructiveInterference::from_pair)
            .collect()
    }

    /// Detect destructive resonance pairs.
    pub fn find_destructive(&self, freq_tolerance: f64) -> Vec<DestructiveInterference> {
        self.resonance_pairs()
            .into_iter()
            .filter(|p| p.is_destructive(freq_tolerance))
            .map(DestructiveInterference::from_pair)
            .collect()
    }

    /// Dominant frequency in the fleet.
    pub fn dominant_frequency(&self) -> Option<f64> {
        self.harmonics.iter()
            .max_by(|a, b| a.amplitude.partial_cmp(&b.amplitude).unwrap())
            .map(|h| h.frequency_hz)
    }

    /// Fleet coherence: how aligned the fleet is (1.0 = perfect alignment, 0.0 = chaos).
    pub fn coherence(&self) -> f64 {
        if self.agents.len() < 2 {
            return 1.0;
        }
        let pairs = self.resonance_pairs();
        let total = pairs.len() as f64;
        let aligned = pairs.iter()
            .filter(|p| p.agent_a.frequency_distance(&p.agent_b) < 0.3)
            .count() as f64;
        aligned / total
    }
}

/// A tuning suggestion for an agent.
#[derive(Debug, Clone)]
pub struct TuningSuggestion {
    pub agent_id: String,
    pub current_hz: f64,
    pub suggested_hz: f64,
    pub current_phase: f64,
    pub suggested_phase: f64,
    pub reason: String,
}

/// Advises how to tune agent parameters for better fleet resonance.
pub struct TuningAdvisor {
    pub target_frequency: f64,
    pub freq_tolerance: f64,
    pub phase_tolerance: f64,
}

impl TuningAdvisor {
    pub fn new(target_frequency: f64) -> Self {
        Self {
            target_frequency,
            freq_tolerance: 0.2,
            phase_tolerance: 0.5,
        }
    }

    /// Suggest tuning adjustments for agents to resonate better with the target frequency.
    pub fn advise(&self, spectrum: &ResonanceSpectrum) -> Vec<TuningSuggestion> {
        spectrum.agents.iter().map(|agent| {
            let freq_dist = ((agent.hz - self.target_frequency).abs())
                / self.target_frequency.max(1.0);
            let needs_freq = freq_dist > self.freq_tolerance;

            // Find the most common phase among well-tuned agents
            let reference_phase = 0.0;
            let needs_phase = agent.phase_difference(&ResonanceFrequency::new(
                "_ref", self.target_frequency, reference_phase, 1.0,
            )) > self.phase_tolerance;

            let suggested_hz = if needs_freq { self.target_frequency } else { agent.hz };
            let suggested_phase = if needs_phase { reference_phase } else { agent.phase };

            let reason = match (needs_freq, needs_phase) {
                (true, true) => "Agent is out of tune and out of phase with fleet".into(),
                (true, false) => "Agent frequency needs adjustment to match fleet".into(),
                (false, true) => "Agent phase needs alignment with fleet".into(),
                (false, false) => "Agent is well-tuned".into(),
            };

            TuningSuggestion {
                agent_id: agent.agent_id.clone(),
                current_hz: agent.hz,
                suggested_hz,
                current_phase: agent.phase,
                suggested_phase,
                reason,
            }
        }).collect()
    }

    /// Find the optimal target frequency for a fleet (weighted average).
    pub fn find_optimal_frequency(agents: &[ResonanceFrequency]) -> f64 {
        if agents.is_empty() {
            return 0.0;
        }
        let total_amp: f64 = agents.iter().map(|a| a.amplitude).sum();
        if total_amp == 0.0 {
            return agents.iter().map(|a| a.hz).sum::<f64>() / agents.len() as f64;
        }
        agents.iter()
            .map(|a| a.hz * a.amplitude)
            .sum::<f64>()
            / total_amp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_agent(id: &str, hz: f64, phase: f64, amp: f64) -> ResonanceFrequency {
        ResonanceFrequency::new(id, hz, phase, amp)
    }

    #[test]
    fn test_frequency_creation() {
        let f = make_agent("a", 2.0, std::f64::consts::PI, 0.8);
        assert_eq!(f.agent_id, "a");
        assert!((f.hz - 2.0).abs() < f64::EPSILON);
        assert!((f.amplitude - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn test_phase_wrapping() {
        let f = make_agent("a", 1.0, 3.0 * std::f64::consts::PI, 0.5);
        assert!(f.phase < 2.0 * std::f64::consts::PI);
    }

    #[test]
    fn test_amplitude_clamping() {
        let f = make_agent("a", 1.0, 0.0, 5.0);
        assert!((f.amplitude - 1.0).abs() < f64::EPSILON);
        let f2 = make_agent("a", 1.0, 0.0, -1.0);
        assert!((f2.amplitude - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_sample() {
        let f = make_agent("a", 1.0, 0.0, 1.0);
        let s = f.sample(0.0);
        assert!((s - 0.0).abs() < 1e-10); // sin(0) = 0

        let s_quarter = f.sample(0.25); // sin(π/2) = 1
        assert!((s_quarter - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_frequency_distance() {
        let a = make_agent("a", 2.0, 0.0, 1.0);
        let b = make_agent("b", 2.0, 0.0, 1.0);
        assert!((a.frequency_distance(&b) - 0.0).abs() < f64::EPSILON);

        let c = make_agent("c", 4.0, 0.0, 1.0);
        assert!(a.frequency_distance(&c) > 0.0);
    }

    #[test]
    fn test_phase_difference() {
        let a = make_agent("a", 1.0, 0.0, 1.0);
        let b = make_agent("b", 1.0, std::f64::consts::PI, 1.0);
        let diff = a.phase_difference(&b);
        assert!((diff - std::f64::consts::PI).abs() < 0.01);
    }

    #[test]
    fn test_constructive_interference() {
        let a = make_agent("a", 2.0, 0.0, 1.0);
        let b = make_agent("b", 2.0, 0.0, 1.0);
        let pair = ResonancePair::new(a, b);

        assert!(pair.is_constructive(0.3, 0.5));

        let ci = ConstructiveInterference::from_pair(pair);
        assert!(ci.amplification >= 1.0); // Constructive should not reduce
        // Two perfectly in-phase signals should amplify: peak should be 2x individual peak
        assert!((ci.combined_hz - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_destructive_interference() {
        let a = make_agent("a", 2.0, 0.0, 1.0);
        let b = make_agent("b", 2.0, std::f64::consts::PI, 1.0);
        let pair = ResonancePair::new(a, b);

        assert!(pair.is_destructive(0.3));

        let di = DestructiveInterference::from_pair(pair);
        assert!(di.cancellation < 0.2, "Expected strong cancellation, got {}", di.cancellation);
    }

    #[test]
    fn test_spectrum_analysis() {
        let agents = vec![
            make_agent("a", 2.0, 0.0, 1.0),
            make_agent("b", 2.1, 0.1, 0.8),
            make_agent("c", 5.0, 1.0, 0.5),
        ];
        let spectrum = ResonanceSpectrum::new(agents);

        let constructive = spectrum.find_constructive(0.3, 0.5);
        assert!(!constructive.is_empty(), "a and b should resonate constructively");

        let dominant = spectrum.dominant_frequency().unwrap();
        assert!((dominant - 2.0).abs() < 0.1 || (dominant - 2.1).abs() < 0.1);
    }

    #[test]
    fn test_resonance_detection() {
        let agents = vec![
            make_agent("a", 1.0, 0.0, 1.0),
            make_agent("b", 1.0, 3.14, 1.0), // anti-phase
            make_agent("c", 1.0, 0.1, 0.9),  // near a
        ];
        let spectrum = ResonanceSpectrum::new(agents);

        let constructive = spectrum.find_constructive(0.1, 0.5);
        let destructive = spectrum.find_destructive(0.1);

        // a-c should be constructive, a-b should be destructive
        assert!(constructive.iter().any(|c|
            c.pair.agent_a.agent_id == "a" && c.pair.agent_b.agent_id == "c"
            || c.pair.agent_a.agent_id == "c" && c.pair.agent_b.agent_id == "a"
        ));
        assert!(destructive.iter().any(|d|
            d.pair.agent_a.agent_id == "a" && d.pair.agent_b.agent_id == "b"
            || d.pair.agent_a.agent_id == "b" && d.pair.agent_b.agent_id == "a"
        ));
    }

    #[test]
    fn test_tuning_suggestions() {
        let agents = vec![
            make_agent("a", 2.0, 0.0, 1.0),
            make_agent("b", 5.0, 1.5, 0.8), // way off
        ];
        let spectrum = ResonanceSpectrum::new(agents);
        let advisor = TuningAdvisor::new(2.0);

        let suggestions = advisor.advise(&spectrum);
        assert_eq!(suggestions.len(), 2);

        let b_suggestion = suggestions.iter().find(|s| s.agent_id == "b").unwrap();
        assert!((b_suggestion.suggested_hz - 2.0).abs() < f64::EPSILON);
        assert!(b_suggestion.reason.contains("out of tune") || b_suggestion.reason.contains("needs adjustment"));
    }

    #[test]
    fn test_optimal_frequency() {
        let agents = vec![
            make_agent("a", 2.0, 0.0, 1.0),
            make_agent("b", 2.0, 0.0, 1.0),
            make_agent("c", 4.0, 0.0, 0.5),
        ];
        let optimal = TuningAdvisor::find_optimal_frequency(&agents);
        // Weighted: (2*1 + 2*1 + 4*0.5) / (1+1+0.5) = 6/2.5 = 2.4
        assert!((optimal - 2.4).abs() < 0.01);
    }

    #[test]
    fn test_fleet_coherence() {
        let coherent = ResonanceSpectrum::new(vec![
            make_agent("a", 2.0, 0.0, 1.0),
            make_agent("b", 2.1, 0.0, 1.0),
        ]);
        assert!(coherent.coherence() > 0.5);

        let incoherent = ResonanceSpectrum::new(vec![
            make_agent("a", 1.0, 0.0, 1.0),
            make_agent("b", 100.0, 0.0, 1.0),
        ]);
        assert!(incoherent.coherence() < 0.5);
    }

    #[test]
    fn test_empty_spectrum() {
        let spectrum = ResonanceSpectrum::new(vec![]);
        assert!(spectrum.dominant_frequency().is_none());
        assert!(spectrum.resonance_pairs().is_empty());
        assert!((spectrum.coherence() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_single_agent_spectrum() {
        let spectrum = ResonanceSpectrum::new(vec![
            make_agent("a", 2.0, 0.0, 1.0),
        ]);
        assert!(spectrum.resonance_pairs().is_empty());
        assert!((spectrum.coherence() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_combined_signal_superposition() {
        let a = make_agent("a", 1.0, 0.0, 1.0);
        let b = make_agent("b", 1.0, 0.0, 1.0);
        let pair = ResonancePair::new(a, b);

        let signal = pair.combined_signal(0.25);
        // Both sin(π/2) = 1.0, combined = 2.0
        assert!((signal - 2.0).abs() < 1e-10);
    }
}
