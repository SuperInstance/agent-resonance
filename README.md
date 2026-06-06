# agent-resonance

**Resonance between agents — constructive and destructive interference for AI fleets.**

When agents operate in a fleet, they develop natural frequencies — how often they produce output, make decisions, or cycle through tasks. When two agents share a frequency and align their phase, their output amplifies (constructive interference). When they share a frequency but oppose in phase, they cancel (destructive interference). `agent-resonance` models, detects, and tunes this phenomenon.

## Core Concepts

### ResonanceFrequency

Each agent has:
- **hz** — operating frequency (how rapidly it cycles)
- **phase** — where in the cycle it starts [0, 2π)
- **amplitude** — output strength [0.0, 1.0]

Signal model: `A * sin(2πft + φ)`

### ResonancePair

Two agents considered together. Compute:
- Frequency distance (0.0 = identical)
- Phase difference (0 = aligned, π = opposed)
- Whether they're constructive or destructive
- Combined signal at any time point

### ConstructiveInterference

When agents align, output amplifies. Measured by:
- **amplification** factor (≥ 1.0 for constructive)
- **combined_hz** (average of the pair)

### DestructiveInterference

When agents oppose, output cancels. Measured by:
- **cancellation** factor (0.0 = total silence, 1.0 = no cancellation)

### ResonanceSpectrum

Fleet-level analysis:
- All agents and their harmonics
- Find all constructive/destructive pairs
- Dominant frequency detection
- Fleet coherence score (0.0 = chaos, 1.0 = perfect alignment)

### TuningAdvisor

Suggests parameter adjustments to bring agents into resonance:
- Target frequency alignment
- Phase correction
- Per-agent tuning suggestions with reasons
- Optimal fleet frequency (amplitude-weighted average)

## Usage

```rust
use agent_resonance::*;

// Define a fleet
let agents = vec![
    ResonanceFrequency::new("planner", 2.0, 0.0, 1.0),
    ResonanceFrequency::new("executor", 2.1, 0.1, 0.8),
    ResonanceFrequency::new("reviewer", 5.0, 1.0, 0.5),
];

// Analyze the spectrum
let spectrum = ResonanceSpectrum::new(agents);

// Find constructive resonance
let constructive = spectrum.find_constructive(0.3, 0.5);
for ci in &constructive {
    println!("{} + {} amplify by {:.2}x",
        ci.pair.agent_a.agent_id,
        ci.pair.agent_b.agent_id,
        ci.amplification);
}

// Find destructive interference
let destructive = spectrum.find_destructive(0.3);
for di in &destructive {
    println!("{} + {} cancel by {:.0}%",
        di.pair.agent_a.agent_id,
        di.pair.agent_b.agent_id,
        di.cancellation * 100.0);
}

// Get tuning advice
let optimal = TuningAdvisor::find_optimal_frequency(&spectrum.agents);
let advisor = TuningAdvisor::new(optimal);
for suggestion in advisor.advise(&spectrum) {
    println!("{}: {} → {} Hz ({})",
        suggestion.agent_id,
        suggestion.current_hz,
        suggestion.suggested_hz,
        suggestion.reason);
}

// Fleet coherence
println!("Fleet coherence: {:.0}%", spectrum.coherence() * 100.0);
```

## The Physics

This models agent fleets as oscillating systems. Each agent's activity pattern is a wave. When waves overlap:

- **Same frequency, same phase** → amplitude doubles (constructive)
- **Same frequency, opposite phase** → amplitude cancels (destructive)
- **Different frequencies** → beat patterns, partial interference

This isn't metaphor — it's a useful abstraction. Agents that "resonate" collaborate more smoothly. Agents that "cancel" create deadlock or redundancy. Tuning frequency and phase means adjusting cycle times and coordination offsets.

## Fleet Coherence

The `coherence()` metric tells you how aligned your fleet is:
- **1.0** — all agents on similar frequencies (tight fleet)
- **0.5** — mixed (some aligned, some not)
- **0.0** — total chaos (every agent at a different frequency)

## Testing

16 tests covering frequency creation, phase wrapping, amplitude clamping, signal sampling, distance metrics, constructive/destructive detection, spectrum analysis, resonance pair detection, tuning suggestions, optimal frequency, fleet coherence, edge cases (empty/single-agent spectrums), and signal superposition.

```bash
cargo test
```

## License

MIT
