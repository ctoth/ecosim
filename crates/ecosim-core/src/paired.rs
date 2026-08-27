//! Exact, domain-owned sentences over two audited ecosystem runs.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

use num_rational::BigRational;
use num_traits::Zero;

/// Exact metadata and terminal observations for one audited run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExactRunRecord {
    run_id: String,
    topology: String,
    parameters: String,
    observation_rule: String,
    paired_schedule: String,
    horizon: usize,
    elapsed: BigRational,
    terminal: BTreeMap<String, BigRational>,
}

impl ExactRunRecord {
    /// Constructs a run record whose fingerprints identify the common paired experiment.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        run_id: impl Into<String>,
        topology: impl Into<String>,
        parameters: impl Into<String>,
        observation_rule: impl Into<String>,
        paired_schedule: impl Into<String>,
        horizon: usize,
        elapsed: BigRational,
        terminal: BTreeMap<String, BigRational>,
    ) -> Result<Self, ExactPairError> {
        let run_id = nonblank("run id", run_id.into())?;
        let topology = nonblank("topology fingerprint", topology.into())?;
        let parameters = nonblank("parameter fingerprint", parameters.into())?;
        let observation_rule = nonblank("observation rule", observation_rule.into())?;
        let paired_schedule = nonblank("paired schedule", paired_schedule.into())?;
        if horizon == 0 {
            return Err(ExactPairError::ZeroHorizon);
        }
        if elapsed <= BigRational::zero() {
            return Err(ExactPairError::NonpositiveElapsed(elapsed));
        }
        if terminal.is_empty() {
            return Err(ExactPairError::EmptyTerminalState);
        }
        for axis in terminal.keys() {
            if axis.trim().is_empty() {
                return Err(ExactPairError::Blank("terminal axis"));
            }
        }
        Ok(Self {
            run_id,
            topology,
            parameters,
            observation_rule,
            paired_schedule,
            horizon,
            elapsed,
            terminal,
        })
    }

    /// Stable run identifier retained by evidence.
    pub fn run_id(&self) -> &str {
        &self.run_id
    }

    /// Exact terminal observation for one named stock axis.
    pub fn terminal(&self, axis: &str) -> Option<&BigRational> {
        self.terminal.get(axis)
    }

    /// Canonical terminal axis order.
    pub fn axes(&self) -> impl ExactSizeIterator<Item = &str> {
        self.terminal.keys().map(String::as_str)
    }

    fn renamed(&self, axes: &BTreeMap<String, String>) -> Result<Self, ExactPairError> {
        validate_renaming(self.terminal.keys(), axes)?;
        let terminal = self
            .terminal
            .iter()
            .map(|(axis, value)| (axes[axis].clone(), value.clone()))
            .collect();
        Self::new(
            self.run_id.clone(),
            self.topology.clone(),
            self.parameters.clone(),
            self.observation_rule.clone(),
            self.paired_schedule.clone(),
            self.horizon,
            self.elapsed.clone(),
            terminal,
        )
    }
}

/// Two exact runs known to belong to the same frozen paired experiment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExactPairedModel {
    baseline: ExactRunRecord,
    perturbed: ExactRunRecord,
}

impl ExactPairedModel {
    /// Validates common topology, parameters, observations, schedule, and horizon.
    pub fn new(
        baseline: ExactRunRecord,
        perturbed: ExactRunRecord,
    ) -> Result<Self, ExactPairError> {
        if baseline.run_id == perturbed.run_id {
            return Err(ExactPairError::DuplicateRunId(baseline.run_id));
        }
        ensure_equal("topology", &baseline.topology, &perturbed.topology)?;
        ensure_equal("parameters", &baseline.parameters, &perturbed.parameters)?;
        ensure_equal(
            "observation rule",
            &baseline.observation_rule,
            &perturbed.observation_rule,
        )?;
        ensure_equal(
            "paired schedule",
            &baseline.paired_schedule,
            &perturbed.paired_schedule,
        )?;
        if baseline.horizon != perturbed.horizon {
            return Err(ExactPairError::Mismatch("horizon"));
        }
        if baseline.elapsed != perturbed.elapsed {
            return Err(ExactPairError::Mismatch("elapsed"));
        }
        if baseline.terminal.keys().ne(perturbed.terminal.keys()) {
            return Err(ExactPairError::Mismatch("terminal axes"));
        }
        Ok(Self {
            baseline,
            perturbed,
        })
    }

    /// Baseline arm.
    pub fn baseline(&self) -> &ExactRunRecord {
        &self.baseline
    }

    /// Perturbed arm.
    pub fn perturbed(&self) -> &ExactRunRecord {
        &self.perturbed
    }

    /// Applies one total bijective axis renaming to both arms.
    pub fn rename_axes(&self, axes: &BTreeMap<String, String>) -> Result<Self, ExactPairError> {
        Self::new(self.baseline.renamed(axes)?, self.perturbed.renamed(axes)?)
    }

    /// Evaluates a stable ordered sentence suite and retains the first violation.
    pub fn evaluate(
        &self,
        sentences: &[PairedComparisonSentence],
    ) -> Result<PairedEvidence, ExactPairError> {
        if sentences.is_empty() {
            return Err(ExactPairError::EmptySentenceSuite);
        }
        let mut names = BTreeSet::new();
        let mut verdicts = Vec::with_capacity(sentences.len());
        for sentence in sentences {
            if !names.insert(sentence.name.clone()) {
                return Err(ExactPairError::DuplicateSentence(sentence.name.clone()));
            }
            let verdict = sentence.evaluate(self)?;
            let failed = matches!(verdict, PairedComparisonVerdict::Violated(_));
            verdicts.push(verdict);
            if failed {
                break;
            }
        }
        Ok(PairedEvidence { verdicts })
    }
}

/// Exact comparison relation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComparisonRelation {
    /// Observed perturbed-minus-baseline delta must be strictly greater.
    GreaterThan,
    /// Observed perturbed-minus-baseline delta must be strictly less.
    LessThan,
}

/// One named terminal-response sentence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PairedComparisonSentence {
    name: String,
    axis: String,
    relation: ComparisonRelation,
    threshold: BigRational,
}

impl PairedComparisonSentence {
    /// Constructs a sentence over a named terminal stock.
    pub fn new(
        name: impl Into<String>,
        axis: impl Into<String>,
        relation: ComparisonRelation,
        threshold: BigRational,
    ) -> Result<Self, ExactPairError> {
        Ok(Self {
            name: nonblank("sentence name", name.into())?,
            axis: nonblank("sentence axis", axis.into())?,
            relation,
            threshold,
        })
    }

    /// Stable sentence name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Stock axis compared at the terminal observation.
    pub fn axis(&self) -> &str {
        &self.axis
    }

    /// Applies the same conservative axis renaming used for a paired model.
    pub fn rename(&self, axes: &BTreeMap<String, String>) -> Result<Self, ExactPairError> {
        let renamed = axes
            .get(&self.axis)
            .ok_or_else(|| ExactPairError::UnknownAxis(self.axis.clone()))?;
        Self::new(
            self.name.clone(),
            renamed.clone(),
            self.relation,
            self.threshold.clone(),
        )
    }

    /// Evaluates the exact signed terminal delta.
    pub fn evaluate(
        &self,
        model: &ExactPairedModel,
    ) -> Result<PairedComparisonVerdict, ExactPairError> {
        let baseline = model
            .baseline
            .terminal(&self.axis)
            .ok_or_else(|| ExactPairError::UnknownAxis(self.axis.clone()))?;
        let perturbed = model
            .perturbed
            .terminal(&self.axis)
            .ok_or_else(|| ExactPairError::UnknownAxis(self.axis.clone()))?;
        let delta = perturbed - baseline;
        let satisfied = match self.relation {
            ComparisonRelation::GreaterThan => delta > self.threshold,
            ComparisonRelation::LessThan => delta < self.threshold,
        };
        if satisfied {
            Ok(PairedComparisonVerdict::Satisfied(
                PairedComparisonWitness {
                    sentence: self.name.clone(),
                    baseline_run: model.baseline.run_id.clone(),
                    perturbed_run: model.perturbed.run_id.clone(),
                    axis: self.axis.clone(),
                    delta,
                    relation: self.relation,
                    threshold: self.threshold.clone(),
                },
            ))
        } else {
            Ok(PairedComparisonVerdict::Violated(
                PairedComparisonViolation {
                    sentence: self.name.clone(),
                    baseline_run: model.baseline.run_id.clone(),
                    perturbed_run: model.perturbed.run_id.clone(),
                    axis: self.axis.clone(),
                    delta,
                    relation: self.relation,
                    threshold: self.threshold.clone(),
                },
            ))
        }
    }
}

/// Positive evidence for one exact comparison.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PairedComparisonWitness {
    /// Sentence name.
    pub sentence: String,
    /// Baseline run identity.
    pub baseline_run: String,
    /// Perturbed run identity.
    pub perturbed_run: String,
    /// Compared stock axis.
    pub axis: String,
    /// Exact perturbed-minus-baseline delta.
    pub delta: BigRational,
    /// Required relation.
    pub relation: ComparisonRelation,
    /// Exact threshold.
    pub threshold: BigRational,
}

/// First failed exact comparison.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PairedComparisonViolation {
    /// Sentence name.
    pub sentence: String,
    /// Baseline run identity.
    pub baseline_run: String,
    /// Perturbed run identity.
    pub perturbed_run: String,
    /// Compared stock axis.
    pub axis: String,
    /// Exact perturbed-minus-baseline delta.
    pub delta: BigRational,
    /// Required relation.
    pub relation: ComparisonRelation,
    /// Exact threshold.
    pub threshold: BigRational,
}

/// Typed result of one paired sentence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PairedComparisonVerdict {
    /// The named comparison holds.
    Satisfied(PairedComparisonWitness),
    /// The named comparison does not hold.
    Violated(PairedComparisonViolation),
}

/// Ordered paired-run evidence, truncated after its first violation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PairedEvidence {
    verdicts: Vec<PairedComparisonVerdict>,
}

impl PairedEvidence {
    /// Evaluated verdicts in input order through the first failure.
    pub fn verdicts(&self) -> &[PairedComparisonVerdict] {
        &self.verdicts
    }

    /// Whether every requested comparison was satisfied.
    pub fn is_satisfied(&self) -> bool {
        self.verdicts
            .iter()
            .all(|verdict| matches!(verdict, PairedComparisonVerdict::Satisfied(_)))
    }
}

/// Structural paired-model or sentence error.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ExactPairError {
    /// A required textual identifier was blank.
    Blank(&'static str),
    /// A run has no accepted transition.
    ZeroHorizon,
    /// Step duration is zero or negative.
    NonpositiveElapsed(BigRational),
    /// A terminal observation is missing every axis.
    EmptyTerminalState,
    /// Both arms use one run identity.
    DuplicateRunId(String),
    /// Paired metadata differs.
    Mismatch(&'static str),
    /// A sentence references an axis outside the model.
    UnknownAxis(String),
    /// An axis renaming is not total and bijective.
    InvalidRenaming,
    /// A sentence suite contains no comparisons.
    EmptySentenceSuite,
    /// A sentence name occurs more than once.
    DuplicateSentence(String),
}

impl fmt::Display for ExactPairError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Blank(field) => write!(formatter, "{field} must be nonblank"),
            Self::ZeroHorizon => formatter.write_str("paired runs require at least one transition"),
            Self::NonpositiveElapsed(value) => {
                write!(formatter, "elapsed time must be positive, got {value}")
            }
            Self::EmptyTerminalState => formatter.write_str("terminal state must be nonempty"),
            Self::DuplicateRunId(id) => write!(formatter, "paired run id occurs twice: {id}"),
            Self::Mismatch(field) => write!(formatter, "paired runs disagree on {field}"),
            Self::UnknownAxis(axis) => write!(formatter, "unknown paired stock axis: {axis}"),
            Self::InvalidRenaming => {
                formatter.write_str("axis renaming must be total and bijective")
            }
            Self::EmptySentenceSuite => formatter.write_str("comparison suite must be nonempty"),
            Self::DuplicateSentence(name) => {
                write!(formatter, "duplicate paired sentence: {name}")
            }
        }
    }
}

impl Error for ExactPairError {}

fn nonblank(field: &'static str, value: String) -> Result<String, ExactPairError> {
    if value.trim().is_empty() {
        Err(ExactPairError::Blank(field))
    } else {
        Ok(value)
    }
}

fn ensure_equal<T: Eq>(field: &'static str, left: &T, right: &T) -> Result<(), ExactPairError> {
    if left == right {
        Ok(())
    } else {
        Err(ExactPairError::Mismatch(field))
    }
}

fn validate_renaming<'a>(
    source: impl Iterator<Item = &'a String>,
    renaming: &BTreeMap<String, String>,
) -> Result<(), ExactPairError> {
    let source = source.cloned().collect::<BTreeSet<_>>();
    let targets = renaming.values().cloned().collect::<BTreeSet<_>>();
    if source.len() != renaming.len()
        || source.iter().any(|axis| !renaming.contains_key(axis))
        || targets.len() != renaming.len()
        || targets.iter().any(|axis| axis.trim().is_empty())
    {
        Err(ExactPairError::InvalidRenaming)
    } else {
        Ok(())
    }
}
