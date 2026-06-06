//! Two agents at similar frequencies start resonating.
//! Shows constructive interference building up amplitude, and destructive interference canceling out.

use agent_resonance::*;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║        RESONANCE DEMO — When Agents Amplify Each Other     ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Two agents at the same frequency, same phase — perfect resonance
    let agent_a = ResonanceFrequency::new("alpha", 2.0, 0.0, 0.7);
    let agent_b = ResonanceFrequency::new("beta",  2.0, 0.0, 0.7);

    println!("━━━ Constructive Resonance: alpha + beta ━━━");
    println!("  alpha: hz={}, phase={:.2}, amplitude={:.2}", agent_a.hz, agent_a.phase, agent_a.amplitude);
    println!("  beta:  hz={}, phase={:.2}, amplitude={:.2}", agent_b.hz, agent_b.phase, agent_b.amplitude);

    let pair = ResonancePair::new(agent_a.clone(), agent_b.clone());
    println!("  Constructive: {}", pair.is_constructive(0.3, 0.5));
    println!("  Destructive:  {}", pair.is_destructive(0.3));

    // Show the resonance building up over time
    println!("\n  Combined signal over time (amplitude builds to 2x individual):");
    println!("  {:>6} {:>8} {:>8} {:>8}", "Time", "Alpha", "Beta", "Combined");
    println!("  {}", "─".repeat(36));
    for step in 0..20 {
        let t = step as f64 * 0.05;
        let sa = agent_a.sample(t);
        let sb = agent_b.sample(t);
        let combined = pair.combined_signal(t);
        let bar_a = signal_bar(sa, 0.7);
        let bar_b = signal_bar(sb, 0.7);
        let bar_c = signal_bar(combined, 1.4);
        println!("  {:>6.2} {:>+8.3} {:>+8.3} {:>+8.3}", t, sa, sb, combined);
        println!("        {} {} {}", bar_a, bar_b, bar_c);
    }

    let ci = ConstructiveInterference::from_pair(pair);
    println!("\n  Amplification factor: {:.2}x", ci.amplification);
    println!("  Combined frequency:   {:.2} Hz", ci.combined_hz);

    // Destructive interference — same frequency, opposite phase
    println!("\n\n━━━ Destructive Resonance: alpha + anti-beta ━━━");
    let agent_c = ResonanceFrequency::new("anti-beta", 2.0, std::f64::consts::PI, 0.7);
    println!("  anti-beta: hz={}, phase={:.2} (π), amplitude={:.2}",
        agent_c.hz, agent_c.phase, agent_c.amplitude);

    let pair_d = ResonancePair::new(agent_a.clone(), agent_c.clone());
    println!("  Constructive: {}", pair_d.is_constructive(0.3, 0.5));
    println!("  Destructive:  {}", pair_d.is_destructive(0.3));

    println!("\n  Signals cancel each other out:");
    println!("  {:>6} {:>8} {:>8} {:>8}", "Time", "Alpha", "Anti-B", "Combined");
    println!("  {}", "─".repeat(36));
    for step in 0..20 {
        let t = step as f64 * 0.05;
        let sa = agent_a.sample(t);
        let sc = agent_c.sample(t);
        let combined = pair_d.combined_signal(t);
        println!("  {:>6.2} {:>+8.3} {:>+8.3} {:>+8.3}", t, sa, sc, combined);
    }

    let di = DestructiveInterference::from_pair(pair_d);
    println!("\n  Cancellation factor: {:.3} (0.0 = total silence, 1.0 = no cancellation)", di.cancellation);

    // Fleet analysis
    println!("\n\n━━━ Fleet Resonance Analysis ━━━");
    let fleet = vec![
        ResonanceFrequency::new("alpha",  2.0, 0.0,            0.9),
        ResonanceFrequency::new("beta",   2.1, 0.1,            0.8),
        ResonanceFrequency::new("gamma",  2.0, 3.14,           0.7),  // anti-phase
        ResonanceFrequency::new("delta",  5.0, 1.0,            0.5),  // different freq
        ResonanceFrequency::new("epsilon",2.05,0.05,            0.6),  // matches alpha/beta
    ];

    let spectrum = ResonanceSpectrum::new(fleet);
    println!("  Fleet size: {} agents", spectrum.agents.len());
    println!("  Dominant frequency: {:.2} Hz", spectrum.dominant_frequency().unwrap_or(0.0));
    println!("  Fleet coherence:    {:.2} (1.0 = perfect alignment)", spectrum.coherence());

    let constructive = spectrum.find_constructive(0.3, 0.5);
    let destructive = spectrum.find_destructive(0.3);

    println!("\n  Constructive pairs ({}):", constructive.len());
    for ci in &constructive {
        println!("    {} + {} → {:.2}x amplification @ {:.2} Hz",
            ci.pair.agent_a.agent_id, ci.pair.agent_b.agent_id,
            ci.amplification, ci.combined_hz);
    }

    println!("\n  Destructive pairs ({}):", destructive.len());
    for di in &destructive {
        println!("    {} + {} → {:.1}% signal remaining",
            di.pair.agent_a.agent_id, di.pair.agent_b.agent_id,
            di.cancellation * 100.0);
    }

    // Tuning advice
    println!("\n  ━━━ Tuning Advisor ━━━");
    let optimal = TuningAdvisor::find_optimal_frequency(&spectrum.agents);
    println!("  Optimal fleet frequency: {:.2} Hz", optimal);

    let advisor = TuningAdvisor::new(optimal);
    for suggestion in advisor.advise(&spectrum) {
        let status = if (suggestion.current_hz - suggestion.suggested_hz).abs() < 0.01 &&
                      (suggestion.current_phase - suggestion.suggested_phase).abs() < 0.01 {
            "✅"
        } else {
            "🔧"
        };
        println!("  {} {:<10} hz: {:.2}→{:.2}  phase: {:.2}→{:.2}  ({})",
            status, suggestion.agent_id,
            suggestion.current_hz, suggestion.suggested_hz,
            suggestion.current_phase, suggestion.suggested_phase,
            suggestion.reason);
    }
}

fn signal_bar(value: f64, max: f64) -> String {
    let width = 20;
    let normalized = (value / max).clamp(-1.0, 1.0);
    let center = width / 2;
    let offset = (normalized * center as f64).round() as isize;
    let mut bar = vec![' '; width];
    bar[center] = '│';
    let pos = (center as isize + offset) as usize;
    if pos < width && pos != center {
        bar[pos] = if offset > 0 { '▸' } else { '◂' };
    }
    bar.into_iter().collect()
}
