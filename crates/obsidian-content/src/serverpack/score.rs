use super::model::{DetectedSide, Lean};

const DECISIVE: f32 = 0.85;
const TIE_MARGIN: f32 = 0.02;
const LEANS: [Lean; 3] = [Lean::Client, Lean::Server, Lean::Both];

#[derive(Debug, Clone)]
pub(crate) struct Signal {
	pub lean: Lean,
	pub weight: f32,
	pub evidence: String,
}

impl Signal {
	pub(crate) fn new(lean: Lean, weight: f32, evidence: impl Into<String>) -> Self {
		Self {
			lean,
			weight,
			evidence: evidence.into(),
		}
	}
}

#[derive(Debug, Clone)]
pub(crate) struct Verdict {
	pub side: DetectedSide,
	pub leaning: Option<Lean>,
	pub confidence: f32,
	pub evidence: Vec<String>,
}

pub(crate) fn decide(signals: &[Signal]) -> Verdict {
	let mut logits = [0.0_f32; 3];
	for signal in signals {
		logits[signal.lean as usize] += signal.weight;
	}
	let peak = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max);
	let weights = logits.map(|logit| (logit - peak).exp());
	let total: f32 = weights.iter().sum();
	let mut ranked: Vec<(Lean, f32)> = LEANS.into_iter().zip(weights.map(|weight| weight / total)).collect();
	ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
	let (best, confidence) = ranked[0];
	let leaning = (confidence - ranked[1].1 > TIE_MARGIN).then_some(best);
	let side = match leaning {
		Some(lean) if confidence >= DECISIVE => DetectedSide::from(lean),
		_ => DetectedSide::Unsure,
	};
	Verdict {
		side,
		leaning,
		confidence,
		evidence: signals.iter().map(|signal| signal.evidence.clone()).collect(),
	}
}

#[cfg(test)]
mod tests {
	use super::{Signal, decide};
	use crate::serverpack::model::{DetectedSide, Lean};

	fn one(lean: Lean, weight: f32) -> Vec<Signal> {
		vec![Signal::new(lean, weight, "test")]
	}

	#[test]
	fn strong_fabric_environment_is_decisive() {
		assert_eq!(decide(&one(Lean::Client, 5.0)).side, DetectedSide::Client);
		assert_eq!(decide(&one(Lean::Server, 5.0)).side, DetectedSide::Server);
	}

	#[test]
	fn wildcard_environment_reads_as_both() {
		assert_eq!(decide(&one(Lean::Both, 3.0)).side, DetectedSide::Both);
	}

	#[test]
	fn weak_or_missing_evidence_is_unsure() {
		assert_eq!(decide(&one(Lean::Client, 1.0)).side, DetectedSide::Unsure);
		assert_eq!(decide(&one(Lean::Both, 1.0)).side, DetectedSide::Unsure);
		assert_eq!(decide(&[]).side, DetectedSide::Unsure);
	}

	#[test]
	fn conflicting_evidence_is_unsure() {
		let signals = vec![Signal::new(Lean::Client, 5.0, "a"), Signal::new(Lean::Server, 5.0, "b")];
		let verdict = decide(&signals);
		assert_eq!(verdict.side, DetectedSide::Unsure);
		assert_eq!(verdict.evidence, vec!["a".to_owned(), "b".to_owned()]);
	}

	#[test]
	fn corroborated_evidence_becomes_decisive() {
		let signals = vec![Signal::new(Lean::Client, 4.0, "dependency"), Signal::new(Lean::Client, 1.0, "classes")];
		assert_eq!(decide(&signals).side, DetectedSide::Client);
	}

	#[test]
	fn unsure_mods_still_lean_towards_their_strongest_side() {
		let verdict = decide(&one(Lean::Client, 1.0));
		assert_eq!(verdict.side, DetectedSide::Unsure);
		assert_eq!(verdict.leaning, Some(Lean::Client));
		assert!(verdict.confidence > 0.5 && verdict.confidence < 0.85);
	}

	#[test]
	fn ties_and_missing_evidence_have_no_lean() {
		let tie = vec![Signal::new(Lean::Client, 5.0, "a"), Signal::new(Lean::Server, 5.0, "b")];
		assert_eq!(decide(&tie).leaning, None);
		assert_eq!(decide(&[]).leaning, None);
	}
}
