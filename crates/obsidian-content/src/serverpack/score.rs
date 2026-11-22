use super::model::DetectedSide;

const DECISIVE: f32 = 0.85;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Lean {
	Client,
	Server,
	Both,
}

impl Lean {
	fn slot(self) -> usize {
		match self {
			Self::Client => 0,
			Self::Server => 1,
			Self::Both => 2,
		}
	}
}

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
	pub confidence: f32,
	pub evidence: Vec<String>,
}

pub(crate) fn decide(signals: &[Signal]) -> Verdict {
	let mut logits = [0.0_f32; 3];
	for signal in signals {
		logits[signal.lean.slot()] += signal.weight;
	}
	let peak = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max);
	let weights = logits.map(|logit| (logit - peak).exp());
	let total: f32 = weights.iter().sum();
	let (best, confidence) = weights
		.iter()
		.enumerate()
		.map(|(slot, weight)| (slot, weight / total))
		.fold((0, f32::NEG_INFINITY), |best, candidate| {
			if candidate.1 > best.1 { candidate } else { best }
		});
	let side = match (confidence >= DECISIVE, best) {
		(false, _) => DetectedSide::Unsure,
		(true, 0) => DetectedSide::Client,
		(true, 1) => DetectedSide::Server,
		(true, _) => DetectedSide::Both,
	};
	Verdict {
		side,
		confidence,
		evidence: signals.iter().map(|signal| signal.evidence.clone()).collect(),
	}
}

#[cfg(test)]
mod tests {
	use super::{Lean, Signal, decide};
	use crate::serverpack::model::DetectedSide;

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
}
