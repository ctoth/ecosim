#![forbid(unsafe_code)]

//! Conserved exact and dense foundations for ecosystem simulation.

mod food_web;
mod kinetics;
mod multikind_fixture;
mod paired;
pub mod paper_models;
mod trophic_network;

pub use food_web::{
    DenseFoodWeb, DenseFoodWebParameters, DenseFoodWebStep, FoodWeb, FoodWebError,
    FoodWebParameters, FoodWebStep,
};
pub use kinetics::{
    AllocationSentence, AllocationVerdict, AllocationViolation, AllocationWitness, KineticError,
    KineticLaw, KineticSentence, KineticVerdict, KineticViolation, KineticWitness,
};
pub use multikind_fixture::{SyntheticMultikindFixture, synthetic_multikind_fixture};
pub use paired::{
    ComparisonRelation, ExactPairError, ExactPairedModel, ExactRunRecord, PairedComparisonSentence,
    PairedComparisonVerdict, PairedComparisonViolation, PairedComparisonWitness, PairedEvidence,
    PairedResponseMetric,
};
pub use trophic_network::{
    CompleteTrophicLawEvidence, ConsumerSpec, DenseTrophicNetwork, DenseTrophicNetworkPlan,
    DenseTrophicNetworkStep, ExactTrophicLawSuiteEvidence, ExactTrophicNetwork,
    ExactTrophicNetworkEvidence, ExactTrophicNetworkPlan, ExactTrophicNetworkStep,
    ExactTrophicTransition, FeedingSpec, ProducerSpec, TROPHIC_DENSE_BALANCE_ABSOLUTE_TOLERANCE,
    TROPHIC_DENSE_BALANCE_RELATIVE_TOLERANCE, TracerTransportChannel, TracerTransportSentence,
    TracerTransportVerdict, TracerTransportViolation, TracerTransportWitness, TrophicBoundary,
    TrophicFlow, TrophicLaw, TrophicLawEvidence, TrophicLawVerdict, TrophicNetworkError,
    TrophicNetworkSpec, TrophicSentenceFamily, check_tracer_transport,
    trophic_dense_balance_tolerance,
};

use std::error::Error as StdError;
use std::fmt;

use conservation_core::{AxisId, BalanceLaw, Grade, GradedLaw, KindId, Provenance};
use conservation_linear::{NullspaceSource, TransitionMatrix, derive_left_nullspace};
use conservation_trace::TraceState;
use institution::Institution;
use institution_conservation::{ConservationInstitution, ConservationSignature, TraceModel};
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, Signed, Zero};

/// One of the two internal energy stores.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Compartment {
    /// The left-hand store.
    Left,
    /// The right-hand store.
    Right,
}

impl Compartment {
    /// Parses the stable external compartment name.
    pub fn parse(value: &str) -> Result<Self, WorldError> {
        match value {
            "left" => Ok(Self::Left),
            "right" => Ok(Self::Right),
            _ => Err(WorldError::UnknownCompartment(value.to_owned())),
        }
    }
}

impl fmt::Display for Compartment {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Left => formatter.write_str("left"),
            Self::Right => formatter.write_str("right"),
        }
    }
}

/// Exact accounting evidence for one run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BalanceReport {
    initial_stock: BigRational,
    inputs: BigRational,
    outputs: BigRational,
    final_stock: BigRational,
    residual: BigRational,
}

impl BalanceReport {
    /// Total stock before the first event.
    pub fn initial_stock(&self) -> &BigRational {
        &self.initial_stock
    }

    /// Cumulative energy crossing into the system boundary.
    pub fn inputs(&self) -> &BigRational {
        &self.inputs
    }

    /// Cumulative energy crossing out of the system boundary.
    pub fn outputs(&self) -> &BigRational {
        &self.outputs
    }

    /// Total stock after the most recent event.
    pub fn final_stock(&self) -> &BigRational {
        &self.final_stock
    }

    /// `initial + inputs - outputs - final` in exact arithmetic.
    pub fn residual(&self) -> &BigRational {
        &self.residual
    }

    /// Whether the exact residual is zero.
    pub fn is_balanced(&self) -> bool {
        self.residual.is_zero()
    }
}

/// A structurally valid two-compartment world and its complete trace.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct World {
    left: BigRational,
    right: BigRational,
    initial_stock: BigRational,
    inputs: BigRational,
    outputs: BigRational,
    trace: Vec<TraceState>,
}

/// An invalid event or malformed foundation result.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum WorldError {
    /// An initial stock or event amount was negative.
    NegativeAmount,
    /// A transfer named the same source and target.
    SameCompartment,
    /// A withdrawal exceeded the selected store.
    InsufficientEnergy {
        compartment: Compartment,
        available: Box<BigRational>,
        requested: Box<BigRational>,
    },
    /// A public compartment name was not recognized.
    UnknownCompartment(String),
    /// The transition system did not have exactly one independent energy law.
    UnexpectedLawCount(usize),
    /// A foundation rejected internally constructed data.
    Foundation(String),
}

impl fmt::Display for WorldError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NegativeAmount => formatter.write_str("energy amounts must be nonnegative"),
            Self::SameCompartment => formatter.write_str("transfer source and target must differ"),
            Self::InsufficientEnergy {
                compartment,
                available,
                requested,
            } => write!(
                formatter,
                "{compartment} contains {available}, but the event requested {requested}"
            ),
            Self::UnknownCompartment(value) => {
                write!(formatter, "unknown compartment {value:?}")
            }
            Self::UnexpectedLawCount(count) => {
                write!(formatter, "expected one energy law, derived {count}")
            }
            Self::Foundation(message) => formatter.write_str(message),
        }
    }
}

impl StdError for WorldError {}

impl World {
    /// Creates a world with exact, nonnegative initial stocks.
    pub fn new(left: BigRational, right: BigRational) -> Result<Self, WorldError> {
        ensure_nonnegative(&left)?;
        ensure_nonnegative(&right)?;
        let initial_stock = &left + &right;
        let mut world = Self {
            left,
            right,
            initial_stock,
            inputs: BigRational::zero(),
            outputs: BigRational::zero(),
            trace: Vec::new(),
        };
        world.trace.push(world.trace_state()?);
        Ok(world)
    }

    /// Returns the selected exact stock.
    pub fn stock(&self, compartment: Compartment) -> &BigRational {
        match compartment {
            Compartment::Left => &self.left,
            Compartment::Right => &self.right,
        }
    }

    /// Returns cumulative net external flow (`inputs - outputs`).
    pub fn net_external(&self) -> BigRational {
        &self.inputs - &self.outputs
    }

    /// Returns the number of states, including the initial state.
    pub fn history_len(&self) -> usize {
        self.trace.len()
    }

    /// Moves exact energy between internal stores.
    pub fn transfer(
        &mut self,
        source: Compartment,
        target: Compartment,
        amount: BigRational,
    ) -> Result<(), WorldError> {
        ensure_nonnegative(&amount)?;
        if source == target {
            return Err(WorldError::SameCompartment);
        }
        self.ensure_available(source, &amount)?;
        match (source, target) {
            (Compartment::Left, Compartment::Right) => {
                self.left -= &amount;
                self.right += amount;
            }
            (Compartment::Right, Compartment::Left) => {
                self.right -= &amount;
                self.left += amount;
            }
            _ => return Err(WorldError::SameCompartment),
        }
        self.append_state()
    }

    /// Adds exact energy through the external boundary.
    pub fn input(&mut self, target: Compartment, amount: BigRational) -> Result<(), WorldError> {
        ensure_nonnegative(&amount)?;
        match target {
            Compartment::Left => self.left += &amount,
            Compartment::Right => self.right += &amount,
        }
        self.inputs += amount;
        self.append_state()
    }

    /// Removes exact energy through the external boundary.
    pub fn output(&mut self, source: Compartment, amount: BigRational) -> Result<(), WorldError> {
        ensure_nonnegative(&amount)?;
        self.ensure_available(source, &amount)?;
        match source {
            Compartment::Left => self.left -= &amount,
            Compartment::Right => self.right -= &amount,
        }
        self.outputs += amount;
        self.append_state()
    }

    /// Produces the ordinary exact input/output ledger view.
    pub fn report(&self) -> BalanceReport {
        let final_stock = &self.left + &self.right;
        let residual = &self.initial_stock + &self.inputs - &self.outputs - &final_stock;
        BalanceReport {
            initial_stock: self.initial_stock.clone(),
            inputs: self.inputs.clone(),
            outputs: self.outputs.clone(),
            final_stock,
            residual,
        }
    }

    /// Checks the complete trace as a model of the derived conservation law.
    pub fn satisfies_energy_law(&self) -> Result<bool, WorldError> {
        let signature = energy_signature()?;
        let model = TraceModel::new(signature.clone(), self.trace.clone())
            .map_err(|error| WorldError::Foundation(error.to_string()))?;
        let sentence = energy_sentence()?;
        ConservationInstitution
            .satisfies(&signature, &model, &sentence)
            .map_err(|error| WorldError::Foundation(error.to_string()))
    }

    /// Checks that both internal stock axes stayed nonnegative over the complete trace.
    pub fn satisfies_nonnegative_stock_sentences(&self) -> Result<bool, WorldError> {
        let signature = energy_signature()?;
        let model = TraceModel::new(signature.clone(), self.trace.clone())
            .map_err(|error| WorldError::Foundation(error.to_string()))?;
        for compartment in [Compartment::Left, Compartment::Right] {
            let sentence = nonnegative_stock_sentence(compartment)?;
            if !ConservationInstitution
                .satisfies(&signature, &model, &sentence)
                .map_err(|error| WorldError::Foundation(error.to_string()))?
            {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn ensure_available(
        &self,
        compartment: Compartment,
        amount: &BigRational,
    ) -> Result<(), WorldError> {
        let available = self.stock(compartment);
        if amount > available {
            return Err(WorldError::InsufficientEnergy {
                compartment,
                available: Box::new(available.clone()),
                requested: Box::new(amount.clone()),
            });
        }
        Ok(())
    }

    fn append_state(&mut self) -> Result<(), WorldError> {
        self.trace.push(self.trace_state()?);
        Ok(())
    }

    fn trace_state(&self) -> Result<TraceState, WorldError> {
        TraceState::new([
            (axis("left")?, self.left.clone()),
            (axis("right")?, self.right.clone()),
            (axis("net_external")?, self.net_external()),
        ])
        .map_err(|error| WorldError::Foundation(error.to_string()))
    }
}

/// Derives the unique exact energy law from all supported transition types.
pub fn energy_law() -> Result<BalanceLaw, WorldError> {
    let one = BigRational::one();
    let zero = BigRational::zero();
    let minus_one = -one.clone();
    let matrix = TransitionMatrix::new(
        [axis("left")?, axis("right")?, axis("net_external")?],
        vec![
            vec![
                minus_one.clone(),
                one.clone(),
                one.clone(),
                zero.clone(),
                minus_one.clone(),
                zero.clone(),
            ],
            vec![
                one.clone(),
                minus_one.clone(),
                zero.clone(),
                one.clone(),
                zero.clone(),
                minus_one.clone(),
            ],
            vec![
                zero.clone(),
                zero,
                one.clone(),
                one.clone(),
                minus_one.clone(),
                minus_one,
            ],
        ],
    )
    .map_err(|error| WorldError::Foundation(error.to_string()))?;
    let mut laws = derive_left_nullspace(&matrix, energy_kind()?, NullspaceSource::Stoichiometric)
        .map_err(|error| WorldError::Foundation(error.to_string()))?;
    if laws.len() != 1 {
        return Err(WorldError::UnexpectedLawCount(laws.len()));
    }
    laws.pop().ok_or(WorldError::UnexpectedLawCount(0))
}

/// Reads the derived energy form as an invariant institutional sentence.
pub fn energy_sentence() -> Result<GradedLaw, WorldError> {
    Ok(GradedLaw::new(energy_law()?, Grade::Invariant))
}

/// Declares that one internal stock axis must remain nonnegative along a trace.
pub fn nonnegative_stock_sentence(compartment: Compartment) -> Result<GradedLaw, WorldError> {
    let form = BalanceLaw::new(
        energy_kind()?,
        [(axis(&compartment.to_string())?, BigRational::one())],
        Provenance::Declared,
    )
    .map_err(|error| WorldError::Foundation(error.to_string()))?;
    Ok(GradedLaw::new(form, Grade::Nonnegative))
}

fn energy_signature() -> Result<ConservationSignature, WorldError> {
    let kind = energy_kind()?;
    ConservationSignature::new([
        (axis("left")?, kind.clone()),
        (axis("right")?, kind.clone()),
        (axis("net_external")?, kind),
    ])
    .map_err(|error| WorldError::Foundation(error.to_string()))
}

fn axis(value: &str) -> Result<AxisId, WorldError> {
    AxisId::new(value).map_err(|error| WorldError::Foundation(error.to_string()))
}

fn energy_kind() -> Result<KindId, WorldError> {
    KindId::new("energy").map_err(|error| WorldError::Foundation(error.to_string()))
}

fn ensure_nonnegative(value: &BigRational) -> Result<(), WorldError> {
    if value.is_negative() {
        Err(WorldError::NegativeAmount)
    } else {
        Ok(())
    }
}

/// Constructs an exact integer-valued energy amount.
pub fn integer_amount(value: BigInt) -> BigRational {
    BigRational::from_integer(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn integer(value: i64) -> BigRational {
        integer_amount(BigInt::from(value))
    }

    #[test]
    fn transition_matrix_derives_boundary_aware_total_energy() {
        let law = energy_law().expect("the fixed transition matrix is valid");
        assert_eq!(law.coefficient(&axis("left").unwrap()), &integer(1));
        assert_eq!(law.coefficient(&axis("right").unwrap()), &integer(1));
        assert_eq!(
            law.coefficient(&axis("net_external").unwrap()),
            &integer(-1)
        );
        let sentence = energy_sentence().expect("the derived law has a valid grade");
        assert_eq!(sentence.form(), &law);
        assert_eq!(sentence.grade(), Grade::Invariant);
    }

    #[test]
    fn nonnegative_sentence_exposes_a_balanced_but_impossible_stock() {
        let signature = energy_signature().unwrap();
        let model = TraceModel::new(
            signature.clone(),
            vec![
                TraceState::new([
                    (axis("left").unwrap(), integer(1)),
                    (axis("right").unwrap(), integer(1)),
                    (axis("net_external").unwrap(), integer(0)),
                ])
                .unwrap(),
                TraceState::new([
                    (axis("left").unwrap(), integer(-1)),
                    (axis("right").unwrap(), integer(3)),
                    (axis("net_external").unwrap(), integer(0)),
                ])
                .unwrap(),
            ],
        )
        .unwrap();

        assert!(
            ConservationInstitution
                .satisfies(&signature, &model, &energy_sentence().unwrap())
                .unwrap()
        );
        assert!(
            !ConservationInstitution
                .satisfies(
                    &signature,
                    &model,
                    &nonnegative_stock_sentence(Compartment::Left).unwrap(),
                )
                .unwrap()
        );
        assert!(
            ConservationInstitution
                .satisfies(
                    &signature,
                    &model,
                    &nonnegative_stock_sentence(Compartment::Right).unwrap(),
                )
                .unwrap()
        );
    }

    #[test]
    fn one_run_has_matching_ledger_and_institution_evidence() {
        let mut world = World::new(integer(10), integer(5)).unwrap();
        world.input(Compartment::Left, integer(7)).unwrap();
        world
            .transfer(Compartment::Left, Compartment::Right, integer(4))
            .unwrap();
        world.output(Compartment::Right, integer(3)).unwrap();

        let report = world.report();
        assert_eq!(report.initial_stock(), &integer(15));
        assert_eq!(report.inputs(), &integer(7));
        assert_eq!(report.outputs(), &integer(3));
        assert_eq!(report.final_stock(), &integer(19));
        assert!(report.is_balanced());
        assert!(world.satisfies_energy_law().unwrap());
        assert!(world.satisfies_nonnegative_stock_sentences().unwrap());
    }

    #[test]
    fn rejected_events_leave_the_world_unchanged() {
        let world = World::new(integer(2), integer(3)).unwrap();
        let mut attempted = world.clone();
        assert!(
            attempted
                .transfer(Compartment::Left, Compartment::Right, integer(4))
                .is_err()
        );
        assert_eq!(attempted, world);

        let mut attempted = world.clone();
        assert!(attempted.output(Compartment::Right, integer(4)).is_err());
        assert_eq!(attempted, world);

        let mut attempted = world.clone();
        assert!(attempted.input(Compartment::Left, integer(-1)).is_err());
        assert_eq!(attempted, world);
    }

    proptest! {
        #[test]
        fn arbitrary_rejected_events_leave_no_partial_state(
            left in 0_i64..1_000_000,
            right in 0_i64..1_000_000,
            excess in 1_i64..1_000_000,
            negative_magnitude in 1_i64..1_000_000,
        ) {
            let world = World::new(integer(left), integer(right)).unwrap();

            let mut attempted = world.clone();
            prop_assert!(attempted.output(Compartment::Left, integer(left + excess)).is_err());
            prop_assert_eq!(&attempted, &world);

            let mut attempted = world.clone();
            prop_assert!(attempted.transfer(
                Compartment::Right,
                Compartment::Left,
                integer(right + excess),
            ).is_err());
            prop_assert_eq!(&attempted, &world);

            let mut attempted = world.clone();
            prop_assert!(attempted.input(Compartment::Left, integer(-negative_magnitude)).is_err());
            prop_assert_eq!(&attempted, &world);

            let mut attempted = world.clone();
            prop_assert!(attempted.transfer(
                Compartment::Left,
                Compartment::Left,
                integer(0),
            ).is_err());
            prop_assert_eq!(&attempted, &world);
        }

        #[test]
        fn splitting_and_merging_same_role_events_preserves_observable_balance(
            initial in 0_i64..1_000_000,
            first in 0_i64..1_000_000,
            second in 0_i64..1_000_000,
        ) {
            let mut merged_input = World::new(integer(initial), integer(0)).unwrap();
            merged_input.input(Compartment::Left, integer(first + second)).unwrap();
            let mut split_input = World::new(integer(initial), integer(0)).unwrap();
            split_input.input(Compartment::Left, integer(first)).unwrap();
            split_input.input(Compartment::Left, integer(second)).unwrap();
            prop_assert_eq!(merged_input.stock(Compartment::Left), split_input.stock(Compartment::Left));
            prop_assert_eq!(merged_input.stock(Compartment::Right), split_input.stock(Compartment::Right));
            prop_assert_eq!(merged_input.net_external(), split_input.net_external());
            prop_assert_eq!(merged_input.report(), split_input.report());

            let total = first + second;
            let mut merged_transfer = World::new(integer(total), integer(0)).unwrap();
            merged_transfer.transfer(Compartment::Left, Compartment::Right, integer(total)).unwrap();
            let mut split_transfer = World::new(integer(total), integer(0)).unwrap();
            split_transfer.transfer(Compartment::Left, Compartment::Right, integer(first)).unwrap();
            split_transfer.transfer(Compartment::Left, Compartment::Right, integer(second)).unwrap();
            prop_assert_eq!(merged_transfer.stock(Compartment::Left), split_transfer.stock(Compartment::Left));
            prop_assert_eq!(merged_transfer.stock(Compartment::Right), split_transfer.stock(Compartment::Right));
            prop_assert_eq!(merged_transfer.net_external(), split_transfer.net_external());
            prop_assert_eq!(merged_transfer.report(), split_transfer.report());
        }

        #[test]
        fn arbitrary_valid_event_sequences_preserve_the_derived_law(
            events in prop::collection::vec((0_u8..6, 0_u16..10_000), 1..64)
        ) {
            let mut world = World::new(integer(11), integer(13)).unwrap();
            for (event, raw_amount) in events {
                let amount = integer(i64::from(raw_amount));
                match event {
                    0 => world.input(Compartment::Left, amount).unwrap(),
                    1 => world.input(Compartment::Right, amount).unwrap(),
                    2 => {
                        world.input(Compartment::Left, amount.clone()).unwrap();
                        world.transfer(Compartment::Left, Compartment::Right, amount).unwrap();
                    }
                    3 => {
                        world.input(Compartment::Right, amount.clone()).unwrap();
                        world.transfer(Compartment::Right, Compartment::Left, amount).unwrap();
                    }
                    4 => {
                        world.input(Compartment::Left, amount.clone()).unwrap();
                        world.output(Compartment::Left, amount).unwrap();
                    }
                    _ => {
                        world.input(Compartment::Right, amount.clone()).unwrap();
                        world.output(Compartment::Right, amount).unwrap();
                    }
                }
            }
            prop_assert!(world.report().is_balanced());
            prop_assert!(world.satisfies_energy_law().unwrap());
            prop_assert!(world.satisfies_nonnegative_stock_sentences().unwrap());
        }
    }
}
