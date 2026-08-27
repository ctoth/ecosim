//! Configurable conservative trophic networks with exact and dense runtimes.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::sync::Arc;

use conservation_core::{AxisId, BalanceLaw, Grade, GradedLaw, KindId, Provenance};
use conservation_dynamics::{
    DenseState, DenseTolerance, ExactState, FlowSpec, FlowTopology, ProcessId, StockDefinition,
    StockFlowError, StockId,
};
use conservation_stock_flow::{
    BoundaryCorrespondence, BoundaryId, BoundaryVerdict, ChannelId, ExactAmounts,
    FlowConstraintVerdict, FlowId, LedgerDefinition, LedgerId, LinearFlowConstraint, OpenBalance,
    OpenBalanceVerdict, SentenceId, StockAxisDefinition, StockFlowCarrier, SymbolId,
    TransitionEquation, TransitionRecord, TransitionTrace, TransitionVerdict, certify_nullspace,
    check_boundary_correspondence, check_linear_flow_constraint, check_open_balance,
    check_transition_equation,
};
use conservation_trace::{LawVerdict, TraceState, check_law};
use institution::Institution;
use institution_conservation::{ConservationInstitution, ConservationSignature, TraceModel};
use num_rational::BigRational;
use num_traits::{One, Signed, Zero};

const NUTRIENT: &str = "nutrient";
const DETRITUS: &str = "detritus";
const MATERIAL: &str = "material-equivalent";
const CUMULATIVE_INPUT: &str = "cumulative_input";
const CUMULATIVE_OUTPUT: &str = "cumulative_output";

/// Absolute tolerance used for dense trophic open-balance diagnostics.
pub const TROPHIC_DENSE_BALANCE_ABSOLUTE_TOLERANCE: f64 = 256.0 * f64::EPSILON;
/// Relative tolerance used for dense trophic open-balance diagnostics.
pub const TROPHIC_DENSE_BALANCE_RELATIVE_TOLERANCE: f64 = 256.0 * f64::EPSILON;

/// The explicit numerical tolerance used by [`DenseTrophicNetwork::is_balanced`].
pub fn trophic_dense_balance_tolerance() -> DenseTolerance {
    DenseTolerance {
        absolute: TROPHIC_DENSE_BALANCE_ABSOLUTE_TOLERANCE,
        relative: TROPHIC_DENSE_BALANCE_RELATIVE_TOLERANCE,
    }
}

/// One nutrient-limited primary producer.
#[derive(Clone, Debug, PartialEq)]
pub struct ProducerSpec<N> {
    name: String,
    max_growth: N,
    nutrient_half_saturation: N,
    mortality: N,
}

impl<N> ProducerSpec<N> {
    /// Declares one producer and its growth and mortality parameters.
    pub fn new(
        name: impl Into<String>,
        max_growth: N,
        nutrient_half_saturation: N,
        mortality: N,
    ) -> Self {
        Self {
            name: name.into(),
            max_growth,
            nutrient_half_saturation,
            mortality,
        }
    }

    /// Stable stock name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// One consumer stock and its background mortality.
#[derive(Clone, Debug, PartialEq)]
pub struct ConsumerSpec<N> {
    name: String,
    mortality: N,
}

impl<N> ConsumerSpec<N> {
    /// Declares one consumer.
    pub fn new(name: impl Into<String>, mortality: N) -> Self {
        Self {
            name: name.into(),
            mortality,
        }
    }

    /// Stable stock name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// A saturating feeding relationship from one biotic resource to a consumer.
#[derive(Clone, Debug, PartialEq)]
pub struct FeedingSpec<N> {
    consumer: String,
    resource: String,
    max_rate: N,
    resource_half_saturation: N,
    assimilation_efficiency: N,
}

impl<N> FeedingSpec<N> {
    /// Declares one directed feeding relationship.
    pub fn new(
        consumer: impl Into<String>,
        resource: impl Into<String>,
        max_rate: N,
        resource_half_saturation: N,
        assimilation_efficiency: N,
    ) -> Self {
        Self {
            consumer: consumer.into(),
            resource: resource.into(),
            max_rate,
            resource_half_saturation,
            assimilation_efficiency,
        }
    }

    /// Consumer stock name.
    pub fn consumer(&self) -> &str {
        &self.consumer
    }

    /// Resource stock name.
    pub fn resource(&self) -> &str {
        &self.resource
    }
}

/// A complete trophic-network declaration over one material-equivalent currency.
#[derive(Clone, Debug, PartialEq)]
pub struct TrophicNetworkSpec<N> {
    producers: Vec<ProducerSpec<N>>,
    consumers: Vec<ConsumerSpec<N>>,
    feedings: Vec<FeedingSpec<N>>,
    decomposition: N,
}

impl<N> TrophicNetworkSpec<N> {
    /// Collects structural and process declarations for later validated compilation.
    pub fn new(
        producers: Vec<ProducerSpec<N>>,
        consumers: Vec<ConsumerSpec<N>>,
        feedings: Vec<FeedingSpec<N>>,
        decomposition: N,
    ) -> Self {
        Self {
            producers,
            consumers,
            feedings,
            decomposition,
        }
    }

    /// Producer declarations in stable compilation order.
    pub fn producers(&self) -> &[ProducerSpec<N>] {
        &self.producers
    }

    /// Consumer declarations in stable compilation order.
    pub fn consumers(&self) -> &[ConsumerSpec<N>] {
        &self.consumers
    }

    /// Feeding declarations in stable compilation order.
    pub fn feedings(&self) -> &[FeedingSpec<N>] {
        &self.feedings
    }

    /// Detritus-to-nutrient decomposition rate.
    pub fn decomposition(&self) -> &N {
        &self.decomposition
    }
}

/// An invalid trophic declaration, state, forcing, or settlement.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TrophicNetworkError {
    /// A user stock identifier was blank or otherwise invalid.
    InvalidStockName(String),
    /// A stock name was declared more than once or used a reserved name.
    DuplicateStock(String),
    /// A feeding edge named a stock that is not a declared consumer.
    UnknownConsumer(String),
    /// A feeding edge named a stock that is not a declared biotic resource.
    UnknownResource(String),
    /// A consumer cannot transfer material to itself through this first feeding law.
    SelfFeeding(String),
    /// The same consumer-resource feeding pair was declared twice.
    DuplicateFeeding { consumer: String, resource: String },
    /// A rate, duration, stock, input, or harvest was negative.
    NegativeAmount,
    /// A dense numeric value was NaN or infinite.
    NonFiniteAmount,
    /// A saturation constant was zero or negative.
    NonpositiveHalfSaturation,
    /// Assimilation efficiency was outside the closed unit interval.
    InvalidAssimilationEfficiency,
    /// An initial-state mapping omitted a compiled stock.
    MissingInitialStock(String),
    /// An initial-state mapping contained a stock outside the compiled network.
    UnknownInitialStock(String),
    /// Harvest forcing named a stock that is not a compiled consumer.
    UnknownHarvestStock(String),
    /// An ordered initial state did not match the compiled stock count.
    InitialStockCount { expected: usize, actual: usize },
    /// An ordered harvest vector did not match the compiled consumer count.
    HarvestCount { expected: usize, actual: usize },
    /// The shared stock-flow kernel rejected compiled data or arithmetic.
    Settlement(StockFlowError),
    /// Institutional trace or sentence construction failed.
    Evidence(String),
}

impl fmt::Display for TrophicNetworkError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidStockName(name) => write!(formatter, "invalid stock name: {name}"),
            Self::DuplicateStock(name) => write!(formatter, "duplicate or reserved stock: {name}"),
            Self::UnknownConsumer(name) => write!(formatter, "unknown consumer: {name}"),
            Self::UnknownResource(name) => write!(formatter, "unknown biotic resource: {name}"),
            Self::SelfFeeding(name) => write!(formatter, "self-feeding is not supported: {name}"),
            Self::DuplicateFeeding { consumer, resource } => {
                write!(
                    formatter,
                    "duplicate feeding edge: {consumer} consumes {resource}"
                )
            }
            Self::NegativeAmount => formatter.write_str("amounts and rates must be nonnegative"),
            Self::NonFiniteAmount => formatter.write_str("dense values must be finite"),
            Self::NonpositiveHalfSaturation => {
                formatter.write_str("half-saturation constants must be positive")
            }
            Self::InvalidAssimilationEfficiency => {
                formatter.write_str("assimilation efficiency must be between zero and one")
            }
            Self::MissingInitialStock(name) => write!(formatter, "missing initial stock: {name}"),
            Self::UnknownInitialStock(name) => write!(formatter, "unknown initial stock: {name}"),
            Self::UnknownHarvestStock(name) => write!(formatter, "unknown harvest stock: {name}"),
            Self::InitialStockCount { expected, actual } => write!(
                formatter,
                "initial state has {actual} stocks; compiled network requires {expected}"
            ),
            Self::HarvestCount { expected, actual } => write!(
                formatter,
                "harvest vector has {actual} consumers; compiled network requires {expected}"
            ),
            Self::Settlement(error) => error.fmt(formatter),
            Self::Evidence(message) => {
                write!(formatter, "institutional evidence failed: {message}")
            }
        }
    }
}

impl Error for TrophicNetworkError {}

impl From<StockFlowError> for TrophicNetworkError {
    fn from(value: StockFlowError) -> Self {
        Self::Settlement(value)
    }
}

#[derive(Clone, Debug)]
struct ProducerRate<N> {
    stock: usize,
    growth_flow: usize,
    mortality_flow: usize,
    max_growth: N,
    half_saturation: N,
    mortality: N,
}

#[derive(Clone, Debug)]
struct ConsumerRate<N> {
    stock: usize,
    mortality_flow: usize,
    harvest_flow: usize,
    mortality: N,
}

#[derive(Clone, Debug)]
struct FeedingRate<N> {
    consumer: String,
    resource: String,
    resource_stock: usize,
    consumer_stock: usize,
    assimilation_flow: usize,
    waste_flow: usize,
    max_rate: N,
    half_saturation: N,
    efficiency: N,
}

#[derive(Debug)]
struct NetworkLayout {
    topology: Arc<FlowTopology>,
    stock_names: Vec<String>,
    consumer_names: Vec<String>,
    stock_indices: BTreeMap<String, usize>,
    growth_flows: BTreeMap<String, usize>,
    feeding_flows: BTreeMap<(String, String), (usize, usize)>,
    mortality_flows: BTreeMap<String, usize>,
    harvest_flows: BTreeMap<String, usize>,
    nutrient_input_flow: usize,
    decomposition_flow: usize,
    flow_count: usize,
    flow_symbols: Vec<TrophicFlow>,
    boundary_symbols: Vec<TrophicBoundary>,
}

#[derive(Clone, Debug)]
struct CompiledNetwork<N> {
    layout: Arc<NetworkLayout>,
    producers: Vec<ProducerRate<N>>,
    consumers: Vec<ConsumerRate<N>>,
    feedings: Vec<FeedingRate<N>>,
    decomposition: N,
    nutrient_stock: usize,
    detritus_stock: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExactTrophicPlanIdentity {
    stock_names: Vec<String>,
    flow_symbols: Vec<TrophicFlow>,
    boundary_symbols: Vec<TrophicBoundary>,
    producers: Vec<(String, BigRational, BigRational, BigRational)>,
    consumers: Vec<(String, BigRational)>,
    feedings: Vec<(String, String, BigRational, BigRational, BigRational)>,
    decomposition: BigRational,
}

/// Stable semantic identity of one compiled trophic flow.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum TrophicFlow {
    /// External material entering the nutrient stock.
    NutrientInput,
    /// Nutrient material incorporated by one producer.
    ProducerGrowth(String),
    /// Consumed material incorporated by one consumer.
    FeedingAssimilation { consumer: String, resource: String },
    /// Consumed material transferred to detritus.
    FeedingWaste { consumer: String, resource: String },
    /// Background mortality transferred to detritus.
    Mortality(String),
    /// Detrital material returned to the nutrient stock.
    Decomposition,
    /// Material removed from one consumer.
    Harvest(String),
}

/// Stable semantic identity of one compiled trophic boundary port.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum TrophicBoundary {
    /// External nutrient input.
    NutrientInput,
    /// External harvest output for one consumer.
    Harvest(String),
}

/// One accepted exact transition with both proposed and kernel-settled flows.
#[derive(Clone, Debug)]
pub struct ExactTrophicTransition {
    layout: Arc<NetworkLayout>,
    index: usize,
    time_before: BigRational,
    time_after: BigRational,
    elapsed: BigRational,
    stocks_before: Vec<BigRational>,
    stocks_after: Vec<BigRational>,
    inputs_before: BigRational,
    inputs_after: BigRational,
    outputs_before: BigRational,
    outputs_after: BigRational,
    proposed: Vec<BigRational>,
    settled: Vec<BigRational>,
    record: TransitionRecord,
}

impl PartialEq for ExactTrophicTransition {
    fn eq(&self, other: &Self) -> bool {
        self.layout.stock_names == other.layout.stock_names
            && self.layout.flow_symbols == other.layout.flow_symbols
            && self.index == other.index
            && self.time_before == other.time_before
            && self.time_after == other.time_after
            && self.elapsed == other.elapsed
            && self.stocks_before == other.stocks_before
            && self.stocks_after == other.stocks_after
            && self.inputs_before == other.inputs_before
            && self.inputs_after == other.inputs_after
            && self.outputs_before == other.outputs_before
            && self.outputs_after == other.outputs_after
            && self.proposed == other.proposed
            && self.settled == other.settled
            && self.record == other.record
    }
}

impl Eq for ExactTrophicTransition {}

impl ExactTrophicTransition {
    /// Zero-based position in the accepted transition trace.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Exact model time before this transition.
    pub fn time_before(&self) -> &BigRational {
        &self.time_before
    }

    /// Exact model time after this transition.
    pub fn time_after(&self) -> &BigRational {
        &self.time_after
    }

    /// Declared exact step duration.
    pub fn elapsed(&self) -> &BigRational {
        &self.elapsed
    }

    /// Stable stock order used by both state vectors.
    pub fn stock_names(&self) -> &[String] {
        &self.layout.stock_names
    }

    /// Exact stock vector before settlement.
    pub fn stocks_before(&self) -> &[BigRational] {
        &self.stocks_before
    }

    /// Exact stock vector after settlement.
    pub fn stocks_after(&self) -> &[BigRational] {
        &self.stocks_after
    }

    /// Exact pre-transition stock amount by semantic name.
    pub fn stock_before(&self, name: &str) -> Option<&BigRational> {
        self.layout
            .stock_indices
            .get(name)
            .and_then(|index| self.stocks_before.get(*index))
    }

    /// Exact post-transition stock amount by semantic name.
    pub fn stock_after(&self, name: &str) -> Option<&BigRational> {
        self.layout
            .stock_indices
            .get(name)
            .and_then(|index| self.stocks_after.get(*index))
    }

    /// Cumulative material input before settlement.
    pub fn inputs_before(&self) -> &BigRational {
        &self.inputs_before
    }

    /// Cumulative material input after settlement.
    pub fn inputs_after(&self) -> &BigRational {
        &self.inputs_after
    }

    /// Cumulative material output before settlement.
    pub fn outputs_before(&self) -> &BigRational {
        &self.outputs_before
    }

    /// Cumulative material output after settlement.
    pub fn outputs_after(&self) -> &BigRational {
        &self.outputs_after
    }

    /// Stable semantic flow order used by both amount vectors.
    pub fn flow_symbols(&self) -> &[TrophicFlow] {
        &self.layout.flow_symbols
    }

    /// Requested exact flow amounts in [`Self::flow_symbols`] order.
    pub fn proposed_amounts(&self) -> &[BigRational] {
        &self.proposed
    }

    /// Kernel-settled exact flow amounts in [`Self::flow_symbols`] order.
    pub fn settled_amounts(&self) -> &[BigRational] {
        &self.settled
    }

    /// Requested amount for one semantic flow.
    pub fn proposed(&self, flow: &TrophicFlow) -> Option<&BigRational> {
        self.layout
            .flow_symbols
            .iter()
            .position(|candidate| candidate == flow)
            .and_then(|index| self.proposed.get(index))
    }

    /// Kernel-settled amount for one semantic flow.
    pub fn settled(&self, flow: &TrophicFlow) -> Option<&BigRational> {
        self.layout
            .flow_symbols
            .iter()
            .position(|candidate| candidate == flow)
            .and_then(|index| self.settled.get(index))
    }

    /// Domain-neutral typed stock-flow record built from the kernel settlement report.
    pub fn record(&self) -> &TransitionRecord {
        &self.record
    }
}

/// The semantic role of one compiled trophic-network sentence.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum TrophicLaw {
    /// Every stock delta equals the effects of settled internal and boundary flows.
    TransitionEquation,
    /// Assimilated and waste flows retain one feeding relationship's declared ratio.
    FeedingPartition { consumer: String, resource: String },
    /// The cumulative input ledger increment equals settled nutrient input.
    NutrientInputCorrespondence,
    /// One consumer's cumulative harvest ledger equals its settled output port.
    HarvestCorrespondence(String),
    /// Total physical material adjusted by cumulative boundary flows is invariant.
    MaterialInvariant,
    /// One named physical stock remains nonnegative.
    StockNonnegative(String),
    /// The cumulative input ledger never decreases.
    CumulativeInputNondecreasing,
    /// The cumulative output ledger never decreases.
    CumulativeOutputNondecreasing,
    /// Checked total material balance against all boundary ports.
    OpenMaterialBalance,
}

impl TrophicLaw {
    /// The single associated axis, when this sentence concerns one axis.
    pub fn axis_name(&self) -> Option<&str> {
        match self {
            Self::TransitionEquation
            | Self::FeedingPartition { .. }
            | Self::NutrientInputCorrespondence
            | Self::HarvestCorrespondence(_)
            | Self::MaterialInvariant
            | Self::OpenMaterialBalance => None,
            Self::StockNonnegative(name) => Some(name),
            Self::CumulativeInputNondecreasing => Some(CUMULATIVE_INPUT),
            Self::CumulativeOutputNondecreasing => Some(CUMULATIVE_OUTPUT),
        }
    }
}

/// Semantic checker family for one complete trophic-law verdict.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrophicSentenceFamily {
    /// Exact state-transition equation.
    Transition,
    /// Exact linear relation among internal settled flows.
    FlowConstraint,
    /// Exact cumulative-ledger/boundary correspondence.
    Boundary,
    /// Existing graded state-trace sentence.
    Graded,
    /// Certificate-derived direct open balance.
    OpenBalance,
}

/// Typed verdict from one member of the complete trophic-law suite.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TrophicLawVerdict {
    /// State-transition equation result.
    Transition(TransitionVerdict),
    /// Feeding-partition result.
    FlowConstraint(FlowConstraintVerdict),
    /// Boundary-ledger result.
    Boundary(BoundaryVerdict),
    /// Existing graded law result.
    Graded(LawVerdict),
    /// Derived open-balance result.
    OpenBalance(OpenBalanceVerdict),
}

impl TrophicLawVerdict {
    /// Whether the typed verdict carries positive evidence.
    pub fn is_satisfied(&self) -> bool {
        match self {
            Self::Transition(verdict) => verdict.is_satisfied(),
            Self::FlowConstraint(verdict) => verdict.is_satisfied(),
            Self::Boundary(verdict) => verdict.is_satisfied(),
            Self::Graded(verdict) => matches!(verdict, LawVerdict::Satisfied(_)),
            Self::OpenBalance(verdict) => verdict.is_satisfied(),
        }
    }
}

/// One named member of the complete process, boundary, and graded law suite.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompleteTrophicLawEvidence {
    law: TrophicLaw,
    family: TrophicSentenceFamily,
    symbols: Vec<SymbolId>,
    grade: Option<Grade>,
    verdict: TrophicLawVerdict,
}

impl CompleteTrophicLawEvidence {
    /// Domain semantic role of the sentence.
    pub fn law(&self) -> &TrophicLaw {
        &self.law
    }

    /// Checker family that produced the verdict.
    pub fn family(&self) -> TrophicSentenceFamily {
        self.family
    }

    /// Relevant exact carrier symbols in canonical order.
    pub fn symbols(&self) -> &[SymbolId] {
        &self.symbols
    }

    /// Grade for an embedded graded sentence, otherwise `None`.
    pub fn grade(&self) -> Option<Grade> {
        self.grade
    }

    /// Typed positive witness or first violation.
    pub fn verdict(&self) -> &TrophicLawVerdict {
        &self.verdict
    }

    /// Whether this member of the suite was satisfied.
    pub fn is_satisfied(&self) -> bool {
        self.verdict.is_satisfied()
    }
}

/// Complete exact trophic process, boundary, graded, and open-balance evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExactTrophicLawSuiteEvidence {
    laws: Vec<CompleteTrophicLawEvidence>,
}

impl ExactTrophicLawSuiteEvidence {
    /// Verdicts in stable family and semantic-symbol order.
    pub fn laws(&self) -> &[CompleteTrophicLawEvidence] {
        &self.laws
    }

    /// Whether every member of the complete suite was satisfied.
    pub fn is_satisfied(&self) -> bool {
        self.laws
            .iter()
            .all(CompleteTrophicLawEvidence::is_satisfied)
    }
}

/// One named graded sentence and its typed verdict for an exact run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrophicLawEvidence {
    law: TrophicLaw,
    grade: Grade,
    verdict: LawVerdict,
}

impl TrophicLawEvidence {
    /// Semantic role of the checked sentence.
    pub fn law(&self) -> &TrophicLaw {
        &self.law
    }

    /// Grade under which the sentence was checked.
    pub fn grade(&self) -> Grade {
        self.grade
    }

    /// Typed positive witness or first-offense violation.
    pub fn verdict(&self) -> &LawVerdict {
        &self.verdict
    }

    /// Whether this individual sentence was satisfied.
    pub fn is_satisfied(&self) -> bool {
        matches!(self.verdict, LawVerdict::Satisfied(_))
    }
}

/// Complete institutional evidence for one exact trophic-network trace.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExactTrophicNetworkEvidence {
    laws: Vec<TrophicLawEvidence>,
}

impl ExactTrophicNetworkEvidence {
    /// Verdicts in stable compiled order: invariant, stocks, input, output.
    pub fn laws(&self) -> &[TrophicLawEvidence] {
        &self.laws
    }

    /// Whether every compiled sentence was satisfied.
    pub fn is_satisfied(&self) -> bool {
        self.laws.iter().all(TrophicLawEvidence::is_satisfied)
    }
}

#[derive(Clone, Debug)]
struct NamedSentence {
    law: TrophicLaw,
    sentence: GradedLaw,
}

#[derive(Clone, Debug)]
enum NamedStockFlowSentence {
    Transition {
        law: TrophicLaw,
        sentence: TransitionEquation,
        symbols: Vec<SymbolId>,
    },
    FlowConstraint {
        law: TrophicLaw,
        sentence: LinearFlowConstraint,
        symbols: Vec<SymbolId>,
    },
    Boundary {
        law: TrophicLaw,
        sentence: BoundaryCorrespondence,
        symbols: Vec<SymbolId>,
    },
}

impl NamedStockFlowSentence {
    fn law(&self) -> &TrophicLaw {
        match self {
            Self::Transition { law, .. }
            | Self::FlowConstraint { law, .. }
            | Self::Boundary { law, .. } => law,
        }
    }
}

#[derive(Clone, Debug)]
struct ExactEvidencePlan {
    signature: ConservationSignature,
    carrier: Arc<StockFlowCarrier>,
    stock_axes: Vec<AxisId>,
    cumulative_input_axis: AxisId,
    cumulative_output_axis: AxisId,
    sentences: Vec<NamedSentence>,
    stock_flow_sentences: Vec<NamedStockFlowSentence>,
    open_balance: OpenBalance,
}

/// Stateful exact-arithmetic execution of a compiled trophic network.
#[derive(Clone, Debug)]
pub struct ExactTrophicNetwork {
    compiled: Arc<CompiledNetwork<BigRational>>,
    evidence_plan: Arc<ExactEvidencePlan>,
    state: ExactState,
    time: BigRational,
    trace: Vec<TraceState>,
    transitions: Vec<ExactTrophicTransition>,
    harvest_ledgers: BTreeMap<String, BigRational>,
}

impl ExactTrophicNetwork {
    /// Validates, compiles, and initializes an exact trophic network.
    pub fn new(
        spec: TrophicNetworkSpec<BigRational>,
        initial: BTreeMap<String, BigRational>,
    ) -> Result<Self, TrophicNetworkError> {
        ExactTrophicNetworkPlan::compile(spec)?.start(initial)
    }

    /// Exact model time.
    pub fn time(&self) -> &BigRational {
        &self.time
    }

    /// Exact stock amount by name.
    pub fn stock(&self, name: &str) -> Option<&BigRational> {
        self.compiled
            .layout
            .stock_indices
            .get(name)
            .and_then(|index| self.state.amounts().get(*index))
    }

    /// Stock names in stable compiled order.
    pub fn stock_names(&self) -> &[String] {
        &self.compiled.layout.stock_names
    }

    /// Exact amounts in [`Self::stock_names`] order.
    pub fn amounts(&self) -> &[BigRational] {
        self.state.amounts()
    }

    /// Exact cumulative boundary input.
    pub fn inputs(&self) -> BigRational {
        self.state.inputs(&material_kind())
    }

    /// Exact cumulative boundary output.
    pub fn outputs(&self) -> BigRational {
        self.state.outputs(&material_kind())
    }

    /// Exact open-system residual.
    pub fn balance_residual(&self) -> BigRational {
        self.state.balance_residual(&material_kind())
    }

    /// Whether the exact open-system ledger closes.
    pub fn is_balanced(&self) -> bool {
        self.balance_residual().is_zero()
    }

    /// Exact trace states, beginning with the initial state.
    pub fn trace(&self) -> &[TraceState] {
        &self.trace
    }

    /// Accepted exact transitions, excluding the initial state snapshot.
    pub fn transitions(&self) -> &[ExactTrophicTransition] {
        &self.transitions
    }

    /// Evaluates every compiled graded sentence for this run.
    ///
    /// At least one successful step is required because institutional trace
    /// satisfaction compares a minimum of two states.
    pub fn evidence(&self) -> Result<ExactTrophicNetworkEvidence, TrophicNetworkError> {
        evaluate_evidence(&self.evidence_plan, &self.trace)
    }

    /// Evaluates the complete exact transition, process, boundary, graded, and open-balance suite.
    pub fn law_evidence(&self) -> Result<ExactTrophicLawSuiteEvidence, TrophicNetworkError> {
        evaluate_law_suite(&self.evidence_plan, &self.trace, &self.transitions)
    }

    pub(crate) fn plan_identity(&self) -> ExactTrophicPlanIdentity {
        ExactTrophicPlanIdentity {
            stock_names: self.compiled.layout.stock_names.clone(),
            flow_symbols: self.compiled.layout.flow_symbols.clone(),
            boundary_symbols: self.compiled.layout.boundary_symbols.clone(),
            producers: self
                .compiled
                .producers
                .iter()
                .map(|producer| {
                    (
                        self.compiled.layout.stock_names[producer.stock].clone(),
                        producer.max_growth.clone(),
                        producer.half_saturation.clone(),
                        producer.mortality.clone(),
                    )
                })
                .collect(),
            consumers: self
                .compiled
                .consumers
                .iter()
                .map(|consumer| {
                    (
                        self.compiled.layout.stock_names[consumer.stock].clone(),
                        consumer.mortality.clone(),
                    )
                })
                .collect(),
            feedings: self
                .compiled
                .feedings
                .iter()
                .map(|feeding| {
                    (
                        feeding.consumer.clone(),
                        feeding.resource.clone(),
                        feeding.max_rate.clone(),
                        feeding.half_saturation.clone(),
                        feeding.efficiency.clone(),
                    )
                })
                .collect(),
            decomposition: self.compiled.decomposition.clone(),
        }
    }

    /// Settles all growth, feeding, mortality, decomposition, input, and harvest proposals.
    pub fn step(
        &mut self,
        elapsed: BigRational,
        nutrient_input: BigRational,
        harvests: BTreeMap<String, BigRational>,
    ) -> Result<ExactTrophicNetworkStep, TrophicNetworkError> {
        ensure_exact_nonnegative(&elapsed)?;
        ensure_exact_nonnegative(&nutrient_input)?;
        validate_exact_harvests(&self.compiled, &harvests)?;
        let requested = exact_requests(
            &self.compiled,
            self.state.amounts(),
            &elapsed,
            nutrient_input,
            &harvests,
        );
        let time_before = self.time.clone();
        let stocks_before = self.state.amounts().to_vec();
        let inputs_before = self.inputs();
        let outputs_before = self.outputs();
        let harvest_ledgers_before = self.harvest_ledgers.clone();
        let mut next_state = self.state.clone();
        let settlement = next_state.settle(&requested)?;
        let mut harvest_ledgers_after = harvest_ledgers_before.clone();
        for (consumer, flow) in &self.compiled.layout.harvest_flows {
            *harvest_ledgers_after
                .get_mut(consumer)
                .expect("compiled consumer has a harvest ledger") +=
                settlement.applied()[*flow].clone();
        }
        let next_time = &self.time + &elapsed;
        let next_trace_state = exact_trace_state(&self.evidence_plan, &next_state)?;
        let record = self
            .evidence_plan
            .carrier
            .record_from_settlement(
                &stocks_before,
                next_state.amounts(),
                &settlement,
                exact_ledger_amounts(
                    &self.evidence_plan.carrier,
                    &inputs_before,
                    &harvest_ledgers_before,
                )?,
                exact_ledger_amounts(
                    &self.evidence_plan.carrier,
                    &next_state.inputs(&material_kind()),
                    &harvest_ledgers_after,
                )?,
            )
            .map_err(evidence_error)?;
        let transition = ExactTrophicTransition {
            layout: Arc::clone(&self.compiled.layout),
            index: self.transitions.len(),
            time_before,
            time_after: next_time.clone(),
            elapsed,
            stocks_before,
            stocks_after: next_state.amounts().to_vec(),
            inputs_before,
            inputs_after: next_state.inputs(&material_kind()),
            outputs_before,
            outputs_after: next_state.outputs(&material_kind()),
            proposed: settlement.requested().to_vec(),
            settled: settlement.applied().to_vec(),
            record,
        };
        self.state = next_state;
        self.time = next_time;
        self.harvest_ledgers = harvest_ledgers_after;
        self.trace.push(next_trace_state);
        self.transitions.push(transition.clone());
        Ok(ExactTrophicNetworkStep { transition })
    }
}

/// An immutable exact trophic topology and process plan reusable across runs.
#[derive(Clone, Debug)]
pub struct ExactTrophicNetworkPlan {
    compiled: Arc<CompiledNetwork<BigRational>>,
    evidence: Arc<ExactEvidencePlan>,
}

impl ExactTrophicNetworkPlan {
    /// Validates and compiles an exact trophic declaration once.
    pub fn compile(spec: TrophicNetworkSpec<BigRational>) -> Result<Self, TrophicNetworkError> {
        validate_exact_spec(&spec)?;
        let compiled = Arc::new(compile_network(spec)?);
        let evidence = Arc::new(compile_evidence_plan(&compiled)?);
        Ok(Self { compiled, evidence })
    }

    /// Stable stock order used by every run from this plan.
    pub fn stock_names(&self) -> &[String] {
        &self.compiled.layout.stock_names
    }

    /// Stable consumer order used by ordered harvest vectors.
    pub fn consumer_names(&self) -> &[String] {
        &self.compiled.layout.consumer_names
    }

    /// Stable compiled flow order shared by every exact run from this plan.
    pub fn flow_symbols(&self) -> &[TrophicFlow] {
        &self.compiled.layout.flow_symbols
    }

    /// Stable compiled boundary-port order shared by every exact run from this plan.
    pub fn boundary_symbols(&self) -> &[TrophicBoundary] {
        &self.compiled.layout.boundary_symbols
    }

    /// Domain-neutral exact carrier used by persisted transitions and typed law checkers.
    pub fn stock_flow_carrier(&self) -> &StockFlowCarrier {
        &self.evidence.carrier
    }

    /// Stable evidence-axis order: physical stocks, cumulative input, cumulative output.
    pub fn evidence_axis_names(&self) -> impl Iterator<Item = &str> {
        self.evidence
            .stock_axes
            .iter()
            .chain([
                &self.evidence.cumulative_input_axis,
                &self.evidence.cumulative_output_axis,
            ])
            .map(AxisId::as_str)
    }

    /// Stable semantic roles of the compiled sentence suite.
    pub fn evidence_laws(&self) -> impl ExactSizeIterator<Item = &TrophicLaw> {
        self.evidence.sentences.iter().map(|named| &named.law)
    }

    /// Complete compiled law order, canonical across declaration-order permutations.
    pub fn law_suite_laws(&self) -> Vec<TrophicLaw> {
        let mut laws = self
            .evidence
            .stock_flow_sentences
            .iter()
            .map(NamedStockFlowSentence::law)
            .cloned()
            .collect::<Vec<_>>();
        let mut graded = self
            .evidence
            .sentences
            .iter()
            .map(|named| named.law.clone())
            .collect::<Vec<_>>();
        graded.sort();
        laws.extend(graded);
        laws.push(TrophicLaw::OpenMaterialBalance);
        laws
    }

    /// Starts an independent exact run without recompiling the topology.
    pub fn start(
        &self,
        initial: BTreeMap<String, BigRational>,
    ) -> Result<ExactTrophicNetwork, TrophicNetworkError> {
        let amounts = exact_initial(&self.compiled.layout, initial)?;
        let state = ExactState::new(Arc::clone(&self.compiled.layout.topology), amounts)?;
        let initial_trace_state = exact_trace_state(&self.evidence, &state)?;
        Ok(ExactTrophicNetwork {
            state,
            compiled: Arc::clone(&self.compiled),
            evidence_plan: Arc::clone(&self.evidence),
            time: BigRational::zero(),
            trace: vec![initial_trace_state],
            transitions: Vec::new(),
            harvest_ledgers: self
                .compiled
                .layout
                .consumer_names
                .iter()
                .cloned()
                .map(|consumer| (consumer, BigRational::zero()))
                .collect(),
        })
    }
}

/// Exact process observations from one trophic-network step.
#[derive(Clone, Debug)]
pub struct ExactTrophicNetworkStep {
    transition: ExactTrophicTransition,
}

impl ExactTrophicNetworkStep {
    /// Exact step duration.
    pub fn elapsed(&self) -> &BigRational {
        self.transition.elapsed()
    }

    /// The persisted transition represented by this step result.
    pub fn transition(&self) -> &ExactTrophicTransition {
        &self.transition
    }

    /// Settled producer growth.
    pub fn growth(&self, producer: &str) -> Option<&BigRational> {
        self.transition
            .layout
            .growth_flows
            .get(producer)
            .and_then(|index| self.transition.settled.get(*index))
    }

    /// Settled total resource consumption, before assimilation/waste partitioning.
    pub fn feeding(&self, consumer: &str, resource: &str) -> Option<BigRational> {
        self.transition
            .layout
            .feeding_flows
            .get(&(consumer.to_owned(), resource.to_owned()))
            .map(|(assimilation, waste)| {
                &self.transition.settled[*assimilation] + &self.transition.settled[*waste]
            })
    }

    /// Settled assimilated feeding material.
    pub fn feeding_assimilation(&self, consumer: &str, resource: &str) -> Option<&BigRational> {
        self.transition
            .layout
            .feeding_flows
            .get(&(consumer.to_owned(), resource.to_owned()))
            .and_then(|(assimilation, _)| self.transition.settled.get(*assimilation))
    }

    /// Settled unassimilated feeding waste.
    pub fn feeding_waste(&self, consumer: &str, resource: &str) -> Option<&BigRational> {
        self.transition
            .layout
            .feeding_flows
            .get(&(consumer.to_owned(), resource.to_owned()))
            .and_then(|(_, waste)| self.transition.settled.get(*waste))
    }

    /// Settled background mortality.
    pub fn mortality(&self, stock: &str) -> Option<&BigRational> {
        self.transition
            .layout
            .mortality_flows
            .get(stock)
            .and_then(|index| self.transition.settled.get(*index))
    }

    /// Settled consumer harvest.
    pub fn harvest(&self, consumer: &str) -> Option<&BigRational> {
        self.transition
            .layout
            .harvest_flows
            .get(consumer)
            .and_then(|index| self.transition.settled.get(*index))
    }

    /// Settled detritus decomposition.
    pub fn decomposition(&self) -> &BigRational {
        &self.transition.settled[self.transition.layout.decomposition_flow]
    }

    /// Settled nutrient input.
    pub fn nutrient_input(&self) -> &BigRational {
        &self.transition.settled[self.transition.layout.nutrient_input_flow]
    }
}

/// Stateful binary64 execution of a compiled trophic network.
#[derive(Clone, Debug)]
pub struct DenseTrophicNetwork {
    compiled: Arc<CompiledNetwork<f64>>,
    state: DenseState,
    time: f64,
}

impl DenseTrophicNetwork {
    /// Validates, compiles, and initializes a dense trophic network.
    pub fn new(
        spec: TrophicNetworkSpec<f64>,
        initial: BTreeMap<String, f64>,
    ) -> Result<Self, TrophicNetworkError> {
        DenseTrophicNetworkPlan::compile(spec)?.start(initial)
    }

    /// Model time.
    pub fn time(&self) -> f64 {
        self.time
    }

    /// Dense stock amount by name.
    pub fn stock(&self, name: &str) -> Option<f64> {
        self.compiled
            .layout
            .stock_indices
            .get(name)
            .and_then(|index| self.state.amounts().get(*index))
            .copied()
    }

    /// Stock names in stable compiled order.
    pub fn stock_names(&self) -> &[String] {
        &self.compiled.layout.stock_names
    }

    /// Dense amounts in [`Self::stock_names`] order.
    pub fn amounts(&self) -> &[f64] {
        self.state.amounts()
    }

    /// Cumulative boundary input.
    pub fn inputs(&self) -> f64 {
        self.state.inputs(&material_kind())
    }

    /// Cumulative boundary output.
    pub fn outputs(&self) -> f64 {
        self.state.outputs(&material_kind())
    }

    /// Dense open-system residual.
    pub fn balance_residual(&self) -> f64 {
        self.state.balance_residual(&material_kind())
    }

    /// Whether the dense ledger closes within the kernel's explicit tolerance.
    pub fn is_balanced(&self) -> bool {
        self.state
            .balance_within(&material_kind(), trophic_dense_balance_tolerance())
    }

    /// Settles one simultaneous process batch atomically.
    pub fn step(
        &mut self,
        elapsed: f64,
        nutrient_input: f64,
        harvests: BTreeMap<String, f64>,
    ) -> Result<DenseTrophicNetworkStep, TrophicNetworkError> {
        validate_dense_harvests(&self.compiled, &harvests)?;
        let ordered = self
            .compiled
            .layout
            .consumer_names
            .iter()
            .map(|name| harvests.get(name).copied().unwrap_or(0.0))
            .collect::<Vec<_>>();
        self.step_ordered(elapsed, nutrient_input, &ordered)
    }

    /// Settles one step with harvests in the compiled consumer order.
    pub fn step_ordered(
        &mut self,
        elapsed: f64,
        nutrient_input: f64,
        harvests: &[f64],
    ) -> Result<DenseTrophicNetworkStep, TrophicNetworkError> {
        let (next_time, requested) =
            self.prepare_ordered_step(elapsed, nutrient_input, harvests)?;
        let settlement = self.state.settle(&requested)?;
        self.time = next_time;
        Ok(DenseTrophicNetworkStep {
            layout: Arc::clone(&self.compiled.layout),
            applied: settlement.applied().to_vec(),
            elapsed,
        })
    }

    /// Settles an ordered step without allocating owned per-flow observations.
    pub fn step_discard_ordered(
        &mut self,
        elapsed: f64,
        nutrient_input: f64,
        harvests: &[f64],
    ) -> Result<(), TrophicNetworkError> {
        let (next_time, requested) =
            self.prepare_ordered_step(elapsed, nutrient_input, harvests)?;
        self.state.settle_discard(&requested)?;
        self.time = next_time;
        Ok(())
    }

    fn prepare_ordered_step(
        &self,
        elapsed: f64,
        nutrient_input: f64,
        harvests: &[f64],
    ) -> Result<(f64, Vec<f64>), TrophicNetworkError> {
        ensure_dense_values(&[elapsed, nutrient_input])?;
        ensure_dense_values(harvests)?;
        ensure_dense_nonnegative(elapsed)?;
        ensure_dense_nonnegative(nutrient_input)?;
        for harvest in harvests {
            ensure_dense_nonnegative(*harvest)?;
        }
        let expected = self.compiled.consumers.len();
        if harvests.len() != expected {
            return Err(TrophicNetworkError::HarvestCount {
                expected,
                actual: harvests.len(),
            });
        }
        let next_time = self.time + elapsed;
        if !next_time.is_finite() {
            return Err(TrophicNetworkError::Settlement(
                StockFlowError::ArithmeticOverflow,
            ));
        }
        let requested = dense_requests(
            &self.compiled,
            self.state.amounts(),
            elapsed,
            nutrient_input,
            harvests,
        );
        Ok((next_time, requested))
    }
}

/// An immutable dense trophic topology and process plan reusable across runs.
#[derive(Clone, Debug)]
pub struct DenseTrophicNetworkPlan {
    compiled: Arc<CompiledNetwork<f64>>,
}

impl DenseTrophicNetworkPlan {
    /// Validates and compiles a dense trophic declaration once.
    pub fn compile(spec: TrophicNetworkSpec<f64>) -> Result<Self, TrophicNetworkError> {
        validate_dense_spec(&spec)?;
        Ok(Self {
            compiled: Arc::new(compile_network(spec)?),
        })
    }

    /// Stable stock order used by every run from this plan.
    pub fn stock_names(&self) -> &[String] {
        &self.compiled.layout.stock_names
    }

    /// Stable consumer order used by ordered harvest vectors.
    pub fn consumer_names(&self) -> &[String] {
        &self.compiled.layout.consumer_names
    }

    /// Starts an independent dense run without recompiling the topology.
    pub fn start(
        &self,
        initial: BTreeMap<String, f64>,
    ) -> Result<DenseTrophicNetwork, TrophicNetworkError> {
        let amounts = dense_initial(&self.compiled.layout, initial)?;
        self.start_ordered(amounts)
    }

    /// Starts a run from amounts in [`Self::stock_names`] order.
    pub fn start_ordered(
        &self,
        amounts: Vec<f64>,
    ) -> Result<DenseTrophicNetwork, TrophicNetworkError> {
        let expected = self.compiled.layout.stock_names.len();
        if amounts.len() != expected {
            return Err(TrophicNetworkError::InitialStockCount {
                expected,
                actual: amounts.len(),
            });
        }
        ensure_dense_values(&amounts)?;
        for amount in &amounts {
            ensure_dense_nonnegative(*amount)?;
        }
        Ok(DenseTrophicNetwork {
            state: DenseState::new(Arc::clone(&self.compiled.layout.topology), amounts)?,
            compiled: Arc::clone(&self.compiled),
            time: 0.0,
        })
    }
}

/// Dense process observations from one trophic-network step.
#[derive(Clone, Debug)]
pub struct DenseTrophicNetworkStep {
    layout: Arc<NetworkLayout>,
    applied: Vec<f64>,
    elapsed: f64,
}

impl DenseTrophicNetworkStep {
    /// Step duration.
    pub fn elapsed(&self) -> f64 {
        self.elapsed
    }

    /// Settled producer growth.
    pub fn growth(&self, producer: &str) -> Option<f64> {
        self.layout
            .growth_flows
            .get(producer)
            .and_then(|index| self.applied.get(*index))
            .copied()
    }

    /// Settled total resource consumption, before assimilation/waste partitioning.
    pub fn feeding(&self, consumer: &str, resource: &str) -> Option<f64> {
        self.layout
            .feeding_flows
            .get(&(consumer.to_owned(), resource.to_owned()))
            .map(|(assimilation, waste)| self.applied[*assimilation] + self.applied[*waste])
    }

    /// Settled background mortality.
    pub fn mortality(&self, stock: &str) -> Option<f64> {
        self.layout
            .mortality_flows
            .get(stock)
            .and_then(|index| self.applied.get(*index))
            .copied()
    }

    /// Settled consumer harvest.
    pub fn harvest(&self, consumer: &str) -> Option<f64> {
        self.layout
            .harvest_flows
            .get(consumer)
            .and_then(|index| self.applied.get(*index))
            .copied()
    }

    /// Settled detritus decomposition.
    pub fn decomposition(&self) -> f64 {
        self.applied[self.layout.decomposition_flow]
    }
}

fn compile_evidence_plan(
    compiled: &CompiledNetwork<BigRational>,
) -> Result<ExactEvidencePlan, TrophicNetworkError> {
    let layout = &compiled.layout;
    let kind = material_kind();
    let stock_axes = layout
        .stock_names
        .iter()
        .map(|name| AxisId::new(name.clone()).map_err(evidence_error))
        .collect::<Result<Vec<_>, _>>()?;
    let cumulative_input_axis = AxisId::new(CUMULATIVE_INPUT).map_err(evidence_error)?;
    let cumulative_output_axis = AxisId::new(CUMULATIVE_OUTPUT).map_err(evidence_error)?;
    let stock_axis_definitions = layout
        .stock_names
        .iter()
        .zip(&stock_axes)
        .map(|(name, axis)| StockAxisDefinition {
            stock: StockId::new(name).expect("compiled stock names are valid"),
            axis: axis.clone(),
        })
        .collect::<Vec<_>>();
    let channels = layout
        .flow_symbols
        .iter()
        .map(stock_flow_channel)
        .collect::<Result<Vec<_>, _>>()?;
    let input_boundary = BoundaryId::new("nutrient-input").map_err(evidence_error)?;
    let harvest_boundaries = layout
        .boundary_symbols
        .iter()
        .filter_map(|boundary| match boundary {
            TrophicBoundary::NutrientInput => None,
            TrophicBoundary::Harvest(consumer) => {
                Some((consumer, BoundaryId::new(format!("harvest:{consumer}"))))
            }
        })
        .map(|(consumer, boundary)| Ok((consumer.clone(), boundary.map_err(evidence_error)?)))
        .collect::<Result<Vec<_>, TrophicNetworkError>>()?;
    let mut ledgers = vec![LedgerDefinition {
        id: input_ledger_id()?,
        axis: cumulative_input_axis.clone(),
        kind: kind.clone(),
        boundaries: vec![input_boundary],
    }];
    for (consumer, boundary) in harvest_boundaries {
        ledgers.push(LedgerDefinition {
            id: harvest_ledger_id(&consumer)?,
            axis: AxisId::new(format!("cumulative_harvest:{consumer}")).map_err(evidence_error)?,
            kind: kind.clone(),
            boundaries: vec![boundary],
        });
    }
    let carrier = Arc::new(
        StockFlowCarrier::new(
            Arc::clone(&layout.topology),
            stock_axis_definitions,
            channels,
            ledgers,
        )
        .map_err(evidence_error)?,
    );
    let mut transition_symbols = carrier
        .internal_effects()
        .axes()
        .cloned()
        .map(SymbolId::Axis)
        .collect::<Vec<_>>();
    transition_symbols.extend(
        carrier
            .internal_effects()
            .columns()
            .cloned()
            .map(SymbolId::Flow),
    );
    transition_symbols.extend(
        carrier
            .boundary_effects()
            .columns()
            .cloned()
            .map(SymbolId::Boundary),
    );
    let mut stock_flow_sentences = vec![NamedStockFlowSentence::Transition {
        law: TrophicLaw::TransitionEquation,
        sentence: TransitionEquation::new(
            SentenceId::new("trophic.transition-equation").map_err(evidence_error)?,
        ),
        symbols: transition_symbols,
    }];
    let mut feedings = compiled.feedings.iter().collect::<Vec<_>>();
    feedings.sort_by(|left, right| {
        (&left.consumer, &left.resource).cmp(&(&right.consumer, &right.resource))
    });
    for feeding in feedings {
        let assimilation = FlowId::new(format!(
            "feeding-assimilated:{}:{}",
            feeding.consumer, feeding.resource
        ))
        .map_err(evidence_error)?;
        let waste = FlowId::new(format!(
            "feeding-waste:{}:{}",
            feeding.consumer, feeding.resource
        ))
        .map_err(evidence_error)?;
        let sentence = LinearFlowConstraint::new(
            &carrier,
            SentenceId::new(format!(
                "trophic.feeding-partition:{}:{}",
                feeding.consumer, feeding.resource
            ))
            .map_err(evidence_error)?,
            kind.clone(),
            [
                (
                    assimilation.clone(),
                    BigRational::one() - &feeding.efficiency,
                ),
                (waste.clone(), -feeding.efficiency.clone()),
            ],
            BigRational::zero(),
        )
        .map_err(evidence_error)?;
        sentence.validate(&carrier).map_err(evidence_error)?;
        stock_flow_sentences.push(NamedStockFlowSentence::FlowConstraint {
            law: TrophicLaw::FeedingPartition {
                consumer: feeding.consumer.clone(),
                resource: feeding.resource.clone(),
            },
            sentence,
            symbols: vec![SymbolId::Flow(assimilation), SymbolId::Flow(waste)],
        });
    }
    let input_ledger = input_ledger_id()?;
    stock_flow_sentences.push(NamedStockFlowSentence::Boundary {
        law: TrophicLaw::NutrientInputCorrespondence,
        sentence: BoundaryCorrespondence::new(
            SentenceId::new("trophic.nutrient-input-correspondence").map_err(evidence_error)?,
            input_ledger.clone(),
        ),
        symbols: vec![
            SymbolId::Ledger(input_ledger),
            SymbolId::Boundary(BoundaryId::new("nutrient-input").map_err(evidence_error)?),
        ],
    });
    for consumer in layout.harvest_flows.keys() {
        let ledger = harvest_ledger_id(consumer)?;
        let boundary = BoundaryId::new(format!("harvest:{consumer}")).map_err(evidence_error)?;
        stock_flow_sentences.push(NamedStockFlowSentence::Boundary {
            law: TrophicLaw::HarvestCorrespondence(consumer.clone()),
            sentence: BoundaryCorrespondence::new(
                SentenceId::new(format!("trophic.harvest-correspondence:{consumer}"))
                    .map_err(evidence_error)?,
                ledger.clone(),
            ),
            symbols: vec![SymbolId::Ledger(ledger), SymbolId::Boundary(boundary)],
        });
    }
    let certificate = certify_nullspace(
        &carrier,
        kind.clone(),
        stock_axes
            .iter()
            .cloned()
            .map(|axis| (axis, BigRational::one())),
    )
    .map_err(evidence_error)?;
    let open_balance = certificate
        .open_balance(SentenceId::new("trophic.open-material-balance").map_err(evidence_error)?);
    let signature = ConservationSignature::new(
        stock_axes
            .iter()
            .chain([&cumulative_input_axis, &cumulative_output_axis])
            .cloned()
            .map(|axis| (axis, kind.clone())),
    )
    .map_err(evidence_error)?;

    let mut total_coefficients = stock_axes
        .iter()
        .cloned()
        .map(|axis| (axis, BigRational::one()))
        .collect::<Vec<_>>();
    total_coefficients.push((cumulative_input_axis.clone(), -BigRational::one()));
    total_coefficients.push((cumulative_output_axis.clone(), BigRational::one()));
    let invariant = BalanceLaw::new(kind.clone(), total_coefficients, Provenance::Declared)
        .map_err(evidence_error)?;
    let mut sentences = vec![NamedSentence {
        law: TrophicLaw::MaterialInvariant,
        sentence: GradedLaw::new(invariant, Grade::Invariant),
    }];

    for (name, axis) in layout.stock_names.iter().zip(&stock_axes) {
        let form = BalanceLaw::new(
            kind.clone(),
            [(axis.clone(), BigRational::one())],
            Provenance::Declared,
        )
        .map_err(evidence_error)?;
        sentences.push(NamedSentence {
            law: TrophicLaw::StockNonnegative(name.clone()),
            sentence: GradedLaw::new(form, Grade::Nonnegative),
        });
    }
    for (law, axis) in [
        (
            TrophicLaw::CumulativeInputNondecreasing,
            cumulative_input_axis.clone(),
        ),
        (
            TrophicLaw::CumulativeOutputNondecreasing,
            cumulative_output_axis.clone(),
        ),
    ] {
        let form = BalanceLaw::new(
            kind.clone(),
            [(axis, BigRational::one())],
            Provenance::Declared,
        )
        .map_err(evidence_error)?;
        sentences.push(NamedSentence {
            law,
            sentence: GradedLaw::new(form, Grade::Nondecreasing),
        });
    }

    Ok(ExactEvidencePlan {
        signature,
        carrier,
        stock_axes,
        cumulative_input_axis,
        cumulative_output_axis,
        sentences,
        stock_flow_sentences,
        open_balance,
    })
}

fn stock_flow_channel(flow: &TrophicFlow) -> Result<ChannelId, TrophicNetworkError> {
    match flow {
        TrophicFlow::NutrientInput => BoundaryId::new("nutrient-input")
            .map(ChannelId::Boundary)
            .map_err(evidence_error),
        TrophicFlow::ProducerGrowth(producer) => FlowId::new(format!("growth:{producer}"))
            .map(ChannelId::Internal)
            .map_err(evidence_error),
        TrophicFlow::FeedingAssimilation { consumer, resource } => {
            FlowId::new(format!("feeding-assimilated:{consumer}:{resource}"))
                .map(ChannelId::Internal)
                .map_err(evidence_error)
        }
        TrophicFlow::FeedingWaste { consumer, resource } => {
            FlowId::new(format!("feeding-waste:{consumer}:{resource}"))
                .map(ChannelId::Internal)
                .map_err(evidence_error)
        }
        TrophicFlow::Mortality(stock) => FlowId::new(format!("mortality:{stock}"))
            .map(ChannelId::Internal)
            .map_err(evidence_error),
        TrophicFlow::Decomposition => FlowId::new("decomposition")
            .map(ChannelId::Internal)
            .map_err(evidence_error),
        TrophicFlow::Harvest(consumer) => BoundaryId::new(format!("harvest:{consumer}"))
            .map(ChannelId::Boundary)
            .map_err(evidence_error),
    }
}

fn input_ledger_id() -> Result<LedgerId, TrophicNetworkError> {
    LedgerId::new(CUMULATIVE_INPUT).map_err(evidence_error)
}

fn harvest_ledger_id(consumer: &str) -> Result<LedgerId, TrophicNetworkError> {
    LedgerId::new(format!("cumulative_harvest:{consumer}")).map_err(evidence_error)
}

fn exact_ledger_amounts(
    carrier: &StockFlowCarrier,
    inputs: &BigRational,
    harvests: &BTreeMap<String, BigRational>,
) -> Result<ExactAmounts<LedgerId>, TrophicNetworkError> {
    let input = input_ledger_id()?;
    ExactAmounts::new(
        carrier
            .identity()
            .ledgers()
            .iter()
            .map(|(ledger, identity)| {
                let amount = if ledger == &input {
                    inputs.clone()
                } else {
                    let consumer = ledger
                        .as_str()
                        .strip_prefix("cumulative_harvest:")
                        .expect("non-input trophic ledgers are per-consumer harvest ledgers");
                    harvests[consumer].clone()
                };
                (ledger.clone(), identity.kind().clone(), amount)
            }),
    )
    .map_err(evidence_error)
}

fn exact_trace_state(
    plan: &ExactEvidencePlan,
    state: &ExactState,
) -> Result<TraceState, TrophicNetworkError> {
    let mut values = plan
        .stock_axes
        .iter()
        .cloned()
        .zip(state.amounts().iter().cloned())
        .collect::<Vec<_>>();
    values.push((
        plan.cumulative_input_axis.clone(),
        state.inputs(&material_kind()),
    ));
    values.push((
        plan.cumulative_output_axis.clone(),
        state.outputs(&material_kind()),
    ));
    TraceState::new(values).map_err(evidence_error)
}

fn evaluate_evidence(
    plan: &ExactEvidencePlan,
    states: &[TraceState],
) -> Result<ExactTrophicNetworkEvidence, TrophicNetworkError> {
    let model = TraceModel::new(plan.signature.clone(), states.to_vec()).map_err(evidence_error)?;
    let mut laws = Vec::with_capacity(plan.sentences.len());
    for named in &plan.sentences {
        let institution_satisfied = ConservationInstitution
            .satisfies(&plan.signature, &model, &named.sentence)
            .map_err(evidence_error)?;
        let verdict = check_law(&named.sentence, model.states()).map_err(evidence_error)?;
        let verdict_satisfied = matches!(verdict, LawVerdict::Satisfied(_));
        if institution_satisfied != verdict_satisfied {
            return Err(TrophicNetworkError::Evidence(
                "institution satisfaction disagreed with its typed trace verdict".to_owned(),
            ));
        }
        laws.push(TrophicLawEvidence {
            law: named.law.clone(),
            grade: named.sentence.grade(),
            verdict,
        });
    }
    Ok(ExactTrophicNetworkEvidence { laws })
}

fn evaluate_law_suite(
    plan: &ExactEvidencePlan,
    states: &[TraceState],
    transitions: &[ExactTrophicTransition],
) -> Result<ExactTrophicLawSuiteEvidence, TrophicNetworkError> {
    let transition_trace = TransitionTrace::new(
        Arc::clone(&plan.carrier),
        transitions
            .iter()
            .map(|transition| transition.record.clone())
            .collect(),
    )
    .map_err(evidence_error)?;
    let mut laws = Vec::new();
    for named in &plan.stock_flow_sentences {
        let evidence = match named {
            NamedStockFlowSentence::Transition {
                law,
                sentence,
                symbols,
            } => CompleteTrophicLawEvidence {
                law: law.clone(),
                family: TrophicSentenceFamily::Transition,
                symbols: symbols.clone(),
                grade: None,
                verdict: TrophicLawVerdict::Transition(
                    check_transition_equation(sentence, &transition_trace)
                        .map_err(evidence_error)?,
                ),
            },
            NamedStockFlowSentence::FlowConstraint {
                law,
                sentence,
                symbols,
            } => CompleteTrophicLawEvidence {
                law: law.clone(),
                family: TrophicSentenceFamily::FlowConstraint,
                symbols: symbols.clone(),
                grade: None,
                verdict: TrophicLawVerdict::FlowConstraint(
                    check_linear_flow_constraint(sentence, &transition_trace)
                        .map_err(evidence_error)?,
                ),
            },
            NamedStockFlowSentence::Boundary {
                law,
                sentence,
                symbols,
            } => CompleteTrophicLawEvidence {
                law: law.clone(),
                family: TrophicSentenceFamily::Boundary,
                symbols: symbols.clone(),
                grade: None,
                verdict: TrophicLawVerdict::Boundary(
                    check_boundary_correspondence(sentence, &transition_trace)
                        .map_err(evidence_error)?,
                ),
            },
        };
        laws.push(evidence);
    }

    let mut graded = evaluate_evidence(plan, states)?.laws;
    graded.sort_by(|left, right| left.law.cmp(&right.law));
    for evidence in graded {
        let symbols = match evidence.law.axis_name() {
            Some(axis) => vec![SymbolId::Axis(
                AxisId::new(axis).expect("compiled trophic evidence axes are valid"),
            )],
            None => {
                let mut axes = plan
                    .stock_axes
                    .iter()
                    .cloned()
                    .chain([
                        plan.cumulative_input_axis.clone(),
                        plan.cumulative_output_axis.clone(),
                    ])
                    .collect::<Vec<_>>();
                axes.sort();
                axes.into_iter().map(SymbolId::Axis).collect()
            }
        };
        laws.push(CompleteTrophicLawEvidence {
            law: evidence.law,
            family: TrophicSentenceFamily::Graded,
            symbols,
            grade: Some(evidence.grade),
            verdict: TrophicLawVerdict::Graded(evidence.verdict),
        });
    }

    let mut open_axes = plan.stock_axes.clone();
    open_axes.sort();
    let mut open_symbols = open_axes
        .into_iter()
        .map(SymbolId::Axis)
        .collect::<Vec<_>>();
    open_symbols.extend(
        plan.carrier
            .boundary_effects()
            .columns()
            .cloned()
            .map(SymbolId::Boundary),
    );
    laws.push(CompleteTrophicLawEvidence {
        law: TrophicLaw::OpenMaterialBalance,
        family: TrophicSentenceFamily::OpenBalance,
        symbols: open_symbols,
        grade: None,
        verdict: TrophicLawVerdict::OpenBalance(
            check_open_balance(&plan.open_balance, &transition_trace).map_err(evidence_error)?,
        ),
    });
    Ok(ExactTrophicLawSuiteEvidence { laws })
}

fn evidence_error(error: impl fmt::Display) -> TrophicNetworkError {
    TrophicNetworkError::Evidence(error.to_string())
}

fn compile_network<N: Clone>(
    spec: TrophicNetworkSpec<N>,
) -> Result<CompiledNetwork<N>, TrophicNetworkError> {
    validate_structure(&spec)?;
    let kind = material_kind();
    let mut stock_names = vec![NUTRIENT.to_owned()];
    stock_names.extend(spec.producers.iter().map(|producer| producer.name.clone()));
    stock_names.extend(spec.consumers.iter().map(|consumer| consumer.name.clone()));
    stock_names.push(DETRITUS.to_owned());
    let stock_indices = stock_names
        .iter()
        .enumerate()
        .map(|(index, name)| (name.clone(), index))
        .collect::<BTreeMap<_, _>>();
    let consumer_names = spec
        .consumers
        .iter()
        .map(|consumer| consumer.name.clone())
        .collect::<Vec<_>>();
    let stock_definitions = stock_names
        .iter()
        .map(|name| StockDefinition {
            id: StockId::new(name).expect("stock identifiers were validated"),
            kind: kind.clone(),
        })
        .collect::<Vec<_>>();
    let mut flows = Vec::new();
    let nutrient_input_flow = push_flow(&mut flows, "nutrient-input", None, Some(NUTRIENT), &kind);

    let mut growth_flows = BTreeMap::new();
    for (index, producer) in spec.producers.iter().enumerate() {
        let flow = push_flow(
            &mut flows,
            &format!("producer-growth-{index}"),
            Some(NUTRIENT),
            Some(&producer.name),
            &kind,
        );
        growth_flows.insert(producer.name.clone(), flow);
    }

    let mut feeding_flows = BTreeMap::new();
    for (index, feeding) in spec.feedings.iter().enumerate() {
        let assimilation = push_flow(
            &mut flows,
            &format!("feeding-assimilation-{index}"),
            Some(&feeding.resource),
            Some(&feeding.consumer),
            &kind,
        );
        let waste = push_flow(
            &mut flows,
            &format!("feeding-waste-{index}"),
            Some(&feeding.resource),
            Some(DETRITUS),
            &kind,
        );
        feeding_flows.insert(
            (feeding.consumer.clone(), feeding.resource.clone()),
            (assimilation, waste),
        );
    }

    let mut mortality_flows = BTreeMap::new();
    for (index, name) in spec
        .producers
        .iter()
        .map(|producer| producer.name.as_str())
        .chain(spec.consumers.iter().map(|consumer| consumer.name.as_str()))
        .enumerate()
    {
        let flow = push_flow(
            &mut flows,
            &format!("mortality-{index}"),
            Some(name),
            Some(DETRITUS),
            &kind,
        );
        mortality_flows.insert(name.to_owned(), flow);
    }

    let decomposition_flow = push_flow(
        &mut flows,
        "decomposition",
        Some(DETRITUS),
        Some(NUTRIENT),
        &kind,
    );
    let mut harvest_flows = BTreeMap::new();
    for (index, consumer) in spec.consumers.iter().enumerate() {
        let flow = push_flow(
            &mut flows,
            &format!("harvest-{index}"),
            Some(&consumer.name),
            None,
            &kind,
        );
        harvest_flows.insert(consumer.name.clone(), flow);
    }

    let flow_count = flows.len();
    let mut flow_symbols = vec![None; flow_count];
    flow_symbols[nutrient_input_flow] = Some(TrophicFlow::NutrientInput);
    for producer in &spec.producers {
        flow_symbols[growth_flows[&producer.name]] =
            Some(TrophicFlow::ProducerGrowth(producer.name.clone()));
    }
    for feeding in &spec.feedings {
        let (assimilation, waste) =
            feeding_flows[&(feeding.consumer.clone(), feeding.resource.clone())];
        flow_symbols[assimilation] = Some(TrophicFlow::FeedingAssimilation {
            consumer: feeding.consumer.clone(),
            resource: feeding.resource.clone(),
        });
        flow_symbols[waste] = Some(TrophicFlow::FeedingWaste {
            consumer: feeding.consumer.clone(),
            resource: feeding.resource.clone(),
        });
    }
    for name in spec
        .producers
        .iter()
        .map(|producer| producer.name.as_str())
        .chain(spec.consumers.iter().map(|consumer| consumer.name.as_str()))
    {
        flow_symbols[mortality_flows[name]] = Some(TrophicFlow::Mortality(name.to_owned()));
    }
    flow_symbols[decomposition_flow] = Some(TrophicFlow::Decomposition);
    for consumer in &spec.consumers {
        flow_symbols[harvest_flows[&consumer.name]] =
            Some(TrophicFlow::Harvest(consumer.name.clone()));
    }
    let flow_symbols = flow_symbols
        .into_iter()
        .map(|symbol| symbol.expect("every compiled flow has one semantic symbol"))
        .collect();
    let mut boundary_symbols = vec![TrophicBoundary::NutrientInput];
    boundary_symbols.extend(harvest_flows.keys().cloned().map(TrophicBoundary::Harvest));
    let topology = Arc::new(FlowTopology::new(stock_definitions, flows)?);
    let layout = Arc::new(NetworkLayout {
        topology,
        stock_names,
        consumer_names,
        stock_indices: stock_indices.clone(),
        growth_flows: growth_flows.clone(),
        feeding_flows: feeding_flows.clone(),
        mortality_flows: mortality_flows.clone(),
        harvest_flows: harvest_flows.clone(),
        nutrient_input_flow,
        decomposition_flow,
        flow_count,
        flow_symbols,
        boundary_symbols,
    });
    let producers = spec
        .producers
        .into_iter()
        .map(|producer| ProducerRate {
            stock: stock_indices[&producer.name],
            growth_flow: growth_flows[&producer.name],
            mortality_flow: mortality_flows[&producer.name],
            max_growth: producer.max_growth,
            half_saturation: producer.nutrient_half_saturation,
            mortality: producer.mortality,
        })
        .collect();
    let consumers = spec
        .consumers
        .into_iter()
        .map(|consumer| ConsumerRate {
            stock: stock_indices[&consumer.name],
            mortality_flow: mortality_flows[&consumer.name],
            harvest_flow: harvest_flows[&consumer.name],
            mortality: consumer.mortality,
        })
        .collect();
    let feedings = spec
        .feedings
        .into_iter()
        .map(|feeding| {
            let flows = feeding_flows[&(feeding.consumer.clone(), feeding.resource.clone())];
            FeedingRate {
                consumer: feeding.consumer.clone(),
                resource: feeding.resource.clone(),
                resource_stock: stock_indices[&feeding.resource],
                consumer_stock: stock_indices[&feeding.consumer],
                assimilation_flow: flows.0,
                waste_flow: flows.1,
                max_rate: feeding.max_rate,
                half_saturation: feeding.resource_half_saturation,
                efficiency: feeding.assimilation_efficiency,
            }
        })
        .collect();
    Ok(CompiledNetwork {
        layout,
        producers,
        consumers,
        feedings,
        decomposition: spec.decomposition,
        nutrient_stock: stock_indices[NUTRIENT],
        detritus_stock: stock_indices[DETRITUS],
    })
}

fn validate_structure<N>(spec: &TrophicNetworkSpec<N>) -> Result<(), TrophicNetworkError> {
    let mut names = BTreeSet::from([
        NUTRIENT.to_owned(),
        DETRITUS.to_owned(),
        CUMULATIVE_INPUT.to_owned(),
        CUMULATIVE_OUTPUT.to_owned(),
    ]);
    for name in spec
        .producers
        .iter()
        .map(|producer| &producer.name)
        .chain(spec.consumers.iter().map(|consumer| &consumer.name))
    {
        StockId::new(name).map_err(|_| TrophicNetworkError::InvalidStockName(name.clone()))?;
        if !names.insert(name.clone()) {
            return Err(TrophicNetworkError::DuplicateStock(name.clone()));
        }
    }
    let consumers = spec
        .consumers
        .iter()
        .map(|consumer| consumer.name.as_str())
        .collect::<BTreeSet<_>>();
    let resources = spec
        .producers
        .iter()
        .map(|producer| producer.name.as_str())
        .chain(spec.consumers.iter().map(|consumer| consumer.name.as_str()))
        .collect::<BTreeSet<_>>();
    let mut pairs = BTreeSet::new();
    for feeding in &spec.feedings {
        if !consumers.contains(feeding.consumer.as_str()) {
            return Err(TrophicNetworkError::UnknownConsumer(
                feeding.consumer.clone(),
            ));
        }
        if !resources.contains(feeding.resource.as_str()) {
            return Err(TrophicNetworkError::UnknownResource(
                feeding.resource.clone(),
            ));
        }
        if feeding.consumer == feeding.resource {
            return Err(TrophicNetworkError::SelfFeeding(feeding.consumer.clone()));
        }
        if !pairs.insert((feeding.consumer.as_str(), feeding.resource.as_str())) {
            return Err(TrophicNetworkError::DuplicateFeeding {
                consumer: feeding.consumer.clone(),
                resource: feeding.resource.clone(),
            });
        }
    }
    Ok(())
}

fn validate_exact_spec(spec: &TrophicNetworkSpec<BigRational>) -> Result<(), TrophicNetworkError> {
    for producer in &spec.producers {
        ensure_exact_nonnegative(&producer.max_growth)?;
        ensure_exact_positive(&producer.nutrient_half_saturation)?;
        ensure_exact_nonnegative(&producer.mortality)?;
    }
    for consumer in &spec.consumers {
        ensure_exact_nonnegative(&consumer.mortality)?;
    }
    for feeding in &spec.feedings {
        ensure_exact_nonnegative(&feeding.max_rate)?;
        ensure_exact_positive(&feeding.resource_half_saturation)?;
        if feeding.assimilation_efficiency.is_negative()
            || feeding.assimilation_efficiency > BigRational::one()
        {
            return Err(TrophicNetworkError::InvalidAssimilationEfficiency);
        }
    }
    ensure_exact_nonnegative(&spec.decomposition)
}

fn validate_dense_spec(spec: &TrophicNetworkSpec<f64>) -> Result<(), TrophicNetworkError> {
    let mut values = vec![spec.decomposition];
    for producer in &spec.producers {
        values.extend([
            producer.max_growth,
            producer.nutrient_half_saturation,
            producer.mortality,
        ]);
    }
    for consumer in &spec.consumers {
        values.push(consumer.mortality);
    }
    for feeding in &spec.feedings {
        values.extend([
            feeding.max_rate,
            feeding.resource_half_saturation,
            feeding.assimilation_efficiency,
        ]);
    }
    ensure_dense_values(&values)?;
    for producer in &spec.producers {
        ensure_dense_nonnegative(producer.max_growth)?;
        ensure_dense_positive(producer.nutrient_half_saturation)?;
        ensure_dense_nonnegative(producer.mortality)?;
    }
    for consumer in &spec.consumers {
        ensure_dense_nonnegative(consumer.mortality)?;
    }
    for feeding in &spec.feedings {
        ensure_dense_nonnegative(feeding.max_rate)?;
        ensure_dense_positive(feeding.resource_half_saturation)?;
        if !(0.0..=1.0).contains(&feeding.assimilation_efficiency) {
            return Err(TrophicNetworkError::InvalidAssimilationEfficiency);
        }
    }
    ensure_dense_nonnegative(spec.decomposition)
}

fn exact_initial(
    layout: &NetworkLayout,
    initial: BTreeMap<String, BigRational>,
) -> Result<Vec<BigRational>, TrophicNetworkError> {
    validate_initial_keys(layout, &initial)?;
    layout
        .stock_names
        .iter()
        .map(|name| {
            let amount = initial
                .get(name)
                .expect("initial keys were validated")
                .clone();
            ensure_exact_nonnegative(&amount)?;
            Ok(amount)
        })
        .collect()
}

fn dense_initial(
    layout: &NetworkLayout,
    initial: BTreeMap<String, f64>,
) -> Result<Vec<f64>, TrophicNetworkError> {
    validate_initial_keys(layout, &initial)?;
    layout
        .stock_names
        .iter()
        .map(|name| {
            let amount = initial[name];
            ensure_dense_values(&[amount])?;
            ensure_dense_nonnegative(amount)?;
            Ok(amount)
        })
        .collect()
}

fn validate_initial_keys<N>(
    layout: &NetworkLayout,
    initial: &BTreeMap<String, N>,
) -> Result<(), TrophicNetworkError> {
    for name in initial.keys() {
        if !layout.stock_indices.contains_key(name) {
            return Err(TrophicNetworkError::UnknownInitialStock(name.clone()));
        }
    }
    for name in &layout.stock_names {
        if !initial.contains_key(name) {
            return Err(TrophicNetworkError::MissingInitialStock(name.clone()));
        }
    }
    Ok(())
}

fn exact_requests(
    compiled: &CompiledNetwork<BigRational>,
    amounts: &[BigRational],
    elapsed: &BigRational,
    nutrient_input: BigRational,
    harvests: &BTreeMap<String, BigRational>,
) -> Vec<BigRational> {
    let mut requested = vec![BigRational::zero(); compiled.layout.flow_count];
    requested[compiled.layout.nutrient_input_flow] = nutrient_input;
    let nutrient = &amounts[compiled.nutrient_stock];
    for producer in &compiled.producers {
        requested[producer.growth_flow] = saturating_exact(
            &producer.max_growth,
            nutrient,
            &producer.half_saturation,
            &amounts[producer.stock],
            elapsed,
        );
        requested[producer.mortality_flow] =
            &producer.mortality * &amounts[producer.stock] * elapsed;
    }
    for feeding in &compiled.feedings {
        let consumption = saturating_exact(
            &feeding.max_rate,
            &amounts[feeding.resource_stock],
            &feeding.half_saturation,
            &amounts[feeding.consumer_stock],
            elapsed,
        );
        requested[feeding.assimilation_flow] = &consumption * &feeding.efficiency;
        requested[feeding.waste_flow] = consumption * (BigRational::one() - &feeding.efficiency);
    }
    for consumer in &compiled.consumers {
        requested[consumer.mortality_flow] =
            &consumer.mortality * &amounts[consumer.stock] * elapsed;
    }
    requested[compiled.layout.decomposition_flow] =
        &compiled.decomposition * &amounts[compiled.detritus_stock] * elapsed;
    for (consumer, amount) in harvests {
        requested[compiled.layout.harvest_flows[consumer]] = amount.clone();
    }
    requested
}

fn dense_requests(
    compiled: &CompiledNetwork<f64>,
    amounts: &[f64],
    elapsed: f64,
    nutrient_input: f64,
    harvests: &[f64],
) -> Vec<f64> {
    let mut requested = vec![0.0; compiled.layout.flow_count];
    requested[compiled.layout.nutrient_input_flow] = nutrient_input;
    let nutrient = amounts[compiled.nutrient_stock];
    for producer in &compiled.producers {
        requested[producer.growth_flow] = saturating_dense(
            producer.max_growth,
            nutrient,
            producer.half_saturation,
            amounts[producer.stock],
            elapsed,
        );
        requested[producer.mortality_flow] = producer.mortality * amounts[producer.stock] * elapsed;
    }
    for feeding in &compiled.feedings {
        let consumption = saturating_dense(
            feeding.max_rate,
            amounts[feeding.resource_stock],
            feeding.half_saturation,
            amounts[feeding.consumer_stock],
            elapsed,
        );
        requested[feeding.assimilation_flow] = consumption * feeding.efficiency;
        requested[feeding.waste_flow] = consumption * (1.0 - feeding.efficiency);
    }
    for consumer in &compiled.consumers {
        requested[consumer.mortality_flow] = consumer.mortality * amounts[consumer.stock] * elapsed;
    }
    requested[compiled.layout.decomposition_flow] =
        compiled.decomposition * amounts[compiled.detritus_stock] * elapsed;
    for (consumer, amount) in compiled.consumers.iter().zip(harvests) {
        requested[consumer.harvest_flow] = *amount;
    }
    requested
}

fn validate_exact_harvests(
    compiled: &CompiledNetwork<BigRational>,
    harvests: &BTreeMap<String, BigRational>,
) -> Result<(), TrophicNetworkError> {
    for (name, amount) in harvests {
        if !compiled.layout.harvest_flows.contains_key(name) {
            return Err(TrophicNetworkError::UnknownHarvestStock(name.clone()));
        }
        ensure_exact_nonnegative(amount)?;
    }
    Ok(())
}

fn validate_dense_harvests(
    compiled: &CompiledNetwork<f64>,
    harvests: &BTreeMap<String, f64>,
) -> Result<(), TrophicNetworkError> {
    for (name, amount) in harvests {
        if !compiled.layout.harvest_flows.contains_key(name) {
            return Err(TrophicNetworkError::UnknownHarvestStock(name.clone()));
        }
        ensure_dense_values(&[*amount])?;
        ensure_dense_nonnegative(*amount)?;
    }
    Ok(())
}

fn push_flow(
    flows: &mut Vec<FlowSpec>,
    process: &str,
    source: Option<&str>,
    target: Option<&str>,
    kind: &KindId,
) -> usize {
    let index = flows.len();
    flows.push(FlowSpec {
        process: ProcessId::new(process).expect("generated process identifiers are nonblank"),
        kind: kind.clone(),
        source: source.map(|name| StockId::new(name).expect("stock identifiers were validated")),
        target: target.map(|name| StockId::new(name).expect("stock identifiers were validated")),
    });
    index
}

fn saturating_exact(
    maximum_rate: &BigRational,
    resource: &BigRational,
    half_saturation: &BigRational,
    actor: &BigRational,
    elapsed: &BigRational,
) -> BigRational {
    maximum_rate * resource / (half_saturation + resource) * actor * elapsed
}

fn saturating_dense(
    maximum_rate: f64,
    resource: f64,
    half_saturation: f64,
    actor: f64,
    elapsed: f64,
) -> f64 {
    if maximum_rate == 0.0 || resource == 0.0 || actor == 0.0 || elapsed == 0.0 {
        return 0.0;
    }
    let response = if resource <= half_saturation {
        let scaled = resource / half_saturation;
        scaled / (1.0 + scaled)
    } else {
        1.0 / (1.0 + half_saturation / resource)
    };
    maximum_rate * response * actor * elapsed
}

fn ensure_exact_nonnegative(value: &BigRational) -> Result<(), TrophicNetworkError> {
    if value.is_negative() {
        Err(TrophicNetworkError::NegativeAmount)
    } else {
        Ok(())
    }
}

fn ensure_exact_positive(value: &BigRational) -> Result<(), TrophicNetworkError> {
    if value.is_positive() {
        Ok(())
    } else {
        Err(TrophicNetworkError::NonpositiveHalfSaturation)
    }
}

fn ensure_dense_values(values: &[f64]) -> Result<(), TrophicNetworkError> {
    if values.iter().all(|value| value.is_finite()) {
        Ok(())
    } else {
        Err(TrophicNetworkError::NonFiniteAmount)
    }
}

fn ensure_dense_nonnegative(value: f64) -> Result<(), TrophicNetworkError> {
    if value < 0.0 {
        Err(TrophicNetworkError::NegativeAmount)
    } else {
        Ok(())
    }
}

fn ensure_dense_positive(value: f64) -> Result<(), TrophicNetworkError> {
    if value > 0.0 {
        Ok(())
    } else {
        Err(TrophicNetworkError::NonpositiveHalfSaturation)
    }
}

fn material_kind() -> KindId {
    KindId::new(MATERIAL).expect("literal kind identifier is nonblank")
}

#[cfg(test)]
mod evidence_tests {
    use super::*;
    use crate::kinetics::{AllocationSentence, AllocationVerdict, AllocationViolationReason};
    use conservation_trace::LawViolation;

    fn integer(value: i64) -> BigRational {
        BigRational::from_integer(value.into())
    }

    fn state(
        plan: &ExactEvidencePlan,
        stock: i64,
        cumulative_input: i64,
        cumulative_output: i64,
    ) -> TraceState {
        let mut values = plan
            .stock_axes
            .iter()
            .cloned()
            .map(|axis| {
                let amount = if axis.as_str() == NUTRIENT {
                    integer(stock)
                } else {
                    integer(0)
                };
                (axis, amount)
            })
            .collect::<Vec<_>>();
        values.push((
            plan.cumulative_input_axis.clone(),
            integer(cumulative_input),
        ));
        values.push((
            plan.cumulative_output_axis.clone(),
            integer(cumulative_output),
        ));
        TraceState::new(values).unwrap()
    }

    fn evidence_plan() -> ExactEvidencePlan {
        let compiled = compile_network(TrophicNetworkSpec::new(
            Vec::new(),
            Vec::new(),
            Vec::new(),
            integer(0),
        ))
        .unwrap();
        compile_evidence_plan(&compiled).unwrap()
    }

    #[test]
    fn compiled_stock_sentence_exposes_negative_stock_hidden_by_invariant_total() {
        let plan = evidence_plan();
        let evidence =
            evaluate_evidence(&plan, &[state(&plan, 1, 0, 0), state(&plan, -1, 0, 2)]).unwrap();

        assert!(
            evidence
                .laws()
                .iter()
                .find(|law| law.law() == &TrophicLaw::MaterialInvariant)
                .unwrap()
                .is_satisfied()
        );
        assert!(matches!(
            evidence
                .laws()
                .iter()
                .find(|law| {
                    law.law() == &TrophicLaw::StockNonnegative("nutrient".to_owned())
                })
                .unwrap()
                .verdict(),
            LawVerdict::Violated(LawViolation::Negative { state_index: 1, observed })
                if observed == &integer(-1)
        ));
    }

    #[test]
    fn compiled_boundary_sentence_reports_first_decreasing_ledger_state() {
        let plan = evidence_plan();
        let evidence =
            evaluate_evidence(&plan, &[state(&plan, 1, 1, 0), state(&plan, 0, 0, 0)]).unwrap();

        assert!(matches!(
            evidence
                .laws()
                .iter()
                .find(|law| law.law() == &TrophicLaw::CumulativeInputNondecreasing)
                .unwrap()
                .verdict(),
            LawVerdict::Violated(LawViolation::Decrease {
                state_index: 1,
                previous,
                observed,
            }) if previous == &integer(1) && observed == &integer(0)
        ));
    }

    #[test]
    fn allocation_sentence_rejects_settlement_that_exceeds_its_proposal() {
        let spec = TrophicNetworkSpec::new(
            vec![ProducerSpec::new(
                "plant",
                integer(1),
                integer(1),
                integer(0),
            )],
            Vec::new(),
            Vec::new(),
            integer(0),
        );
        let initial = BTreeMap::from([
            (NUTRIENT.to_owned(), integer(1)),
            ("plant".to_owned(), integer(1)),
            (DETRITUS.to_owned(), integer(0)),
        ]);
        let mut network = ExactTrophicNetwork::new(spec, initial).unwrap();
        network
            .step(integer(1), integer(0), BTreeMap::new())
            .unwrap();
        let mut transition = network.transitions()[0].clone();
        let flow = TrophicFlow::ProducerGrowth("plant".to_owned());
        let flow_index = transition
            .flow_symbols()
            .iter()
            .position(|candidate| candidate == &flow)
            .unwrap();
        transition.settled[flow_index] = &transition.proposed[flow_index] + integer(1);
        let sentence =
            AllocationSentence::new("plant-source-allocation", NUTRIENT, [flow.clone()]).unwrap();

        assert!(matches!(
            sentence.evaluate(&[transition]).unwrap(),
            AllocationVerdict::Violated(violation)
                if violation.flow == flow
                    && violation.reason == AllocationViolationReason::OutOfBounds
                    && violation.settled > violation.proposed
        ));
    }
}
