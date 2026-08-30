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
const KIND_SEPARATOR: char = ':';

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
    tracer_kinds: Vec<String>,
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
            tracer_kinds: Vec::new(),
        }
    }

    /// Declares additional conserved kinds transported passively by the material flows.
    ///
    /// Each tracer kind gets one stock per material stock and one flow per
    /// material flow. Tracer flows move in exact proportion to the settled
    /// material flow and the source stock's pre-transition tracer composition,
    /// so each kind's books close independently. Kinds compile in sorted
    /// order, so declaration order never changes the compiled plan.
    pub fn with_tracer_kinds(
        producers: Vec<ProducerSpec<N>>,
        consumers: Vec<ConsumerSpec<N>>,
        feedings: Vec<FeedingSpec<N>>,
        decomposition: N,
        tracer_kinds: Vec<String>,
    ) -> Self {
        Self {
            producers,
            consumers,
            feedings,
            decomposition,
            tracer_kinds,
        }
    }

    /// Declared tracer kinds in stable compilation order.
    pub fn tracer_kinds(&self) -> &[String] {
        &self.tracer_kinds
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
    /// A tracer kind name was blank, reserved, or contained the kind separator.
    InvalidKindName(String),
    /// A tracer kind was declared more than once.
    DuplicateKind(String),
    /// A tracer input or initial state named an undeclared kind.
    UnknownKind(String),
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
            Self::InvalidKindName(name) => write!(formatter, "invalid tracer kind name: {name}"),
            Self::DuplicateKind(name) => write!(formatter, "duplicate tracer kind: {name}"),
            Self::UnknownKind(name) => write!(formatter, "unknown tracer kind: {name}"),
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
    tracer_kinds: Vec<String>,
    tracer_kind_ids: Vec<KindId>,
}

impl NetworkLayout {
    /// Stocks in the full compiled topology: one block per kind.
    fn total_stock_count(&self) -> usize {
        self.stock_names.len() * (1 + self.tracer_kinds.len())
    }

    /// Flows in the full compiled topology: one block per kind.
    fn total_flow_count(&self) -> usize {
        self.flow_count * (1 + self.tracer_kinds.len())
    }

    /// Zero-based tracer block for a declared kind name.
    fn tracer_block(&self, kind: &str) -> Option<usize> {
        self.tracer_kinds.iter().position(|name| name == kind)
    }

    /// Full-topology stock index of one tracer block's mirror of a base stock.
    fn tracer_stock_index(&self, block: usize, base_stock: usize) -> usize {
        self.stock_names.len() * (1 + block) + base_stock
    }

    /// Full-topology flow index of one tracer block's mirror of a base flow.
    fn tracer_flow_index(&self, block: usize, base_flow: usize) -> usize {
        self.flow_count * (1 + block) + base_flow
    }
}

/// Suffixes one base identifier with a tracer kind name.
fn tracer_name(base: &str, kind: &str) -> String {
    format!("{base}{KIND_SEPARATOR}{kind}")
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
    tracer_kinds: Vec<String>,
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
        &self.stocks_before[..self.layout.stock_names.len()]
    }

    /// Exact stock vector after settlement.
    pub fn stocks_after(&self) -> &[BigRational] {
        &self.stocks_after[..self.layout.stock_names.len()]
    }

    /// Exact pre-transition tracer stocks for one kind in [`Self::stock_names`] order.
    pub fn tracer_stocks_before(&self, kind: &str) -> Option<&[BigRational]> {
        let block = self.layout.tracer_block(kind)?;
        let start = self.layout.tracer_stock_index(block, 0);
        Some(&self.stocks_before[start..start + self.layout.stock_names.len()])
    }

    /// Exact post-transition tracer stocks for one kind in [`Self::stock_names`] order.
    pub fn tracer_stocks_after(&self, kind: &str) -> Option<&[BigRational]> {
        let block = self.layout.tracer_block(kind)?;
        let start = self.layout.tracer_stock_index(block, 0);
        Some(&self.stocks_after[start..start + self.layout.stock_names.len()])
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
        &self.proposed[..self.layout.flow_count]
    }

    /// Kernel-settled exact flow amounts in [`Self::flow_symbols`] order.
    pub fn settled_amounts(&self) -> &[BigRational] {
        &self.settled[..self.layout.flow_count]
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

    /// Requested tracer amount for one kind's mirror of a semantic flow.
    pub fn tracer_proposed(&self, kind: &str, flow: &TrophicFlow) -> Option<&BigRational> {
        self.tracer_flow_position(kind, flow)
            .and_then(|index| self.proposed.get(index))
    }

    /// Kernel-settled tracer amount for one kind's mirror of a semantic flow.
    pub fn tracer_settled(&self, kind: &str, flow: &TrophicFlow) -> Option<&BigRational> {
        self.tracer_flow_position(kind, flow)
            .and_then(|index| self.settled.get(index))
    }

    fn tracer_flow_position(&self, kind: &str, flow: &TrophicFlow) -> Option<usize> {
        let block = self.layout.tracer_block(kind)?;
        self.layout
            .flow_symbols
            .iter()
            .position(|candidate| candidate == flow)
            .map(|index| self.layout.tracer_flow_index(block, index))
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
    /// Every settled flow of one tracer kind moves in exact source-composition
    /// proportion to its settled material flow. Always wrapped in [`Self::Tracer`].
    TracerTransport,
    /// One base law transported to a declared tracer kind's mirrored block.
    Tracer {
        /// The declared tracer kind name.
        kind: String,
        /// The base-kind law this sentence mirrors.
        law: Box<TrophicLaw>,
    },
}

impl TrophicLaw {
    /// The single associated axis, when this sentence concerns one axis.
    pub fn axis_name(&self) -> Option<String> {
        match self {
            Self::TransitionEquation
            | Self::FeedingPartition { .. }
            | Self::NutrientInputCorrespondence
            | Self::HarvestCorrespondence(_)
            | Self::MaterialInvariant
            | Self::OpenMaterialBalance
            | Self::TracerTransport => None,
            Self::StockNonnegative(name) => Some(name.clone()),
            Self::CumulativeInputNondecreasing => Some(CUMULATIVE_INPUT.to_owned()),
            Self::CumulativeOutputNondecreasing => Some(CUMULATIVE_OUTPUT.to_owned()),
            Self::Tracer { kind, law } => law.axis_name().map(|axis| tracer_name(&axis, kind)),
        }
    }

    /// Wraps one base law for a tracer kind's mirrored block.
    fn for_tracer(self, kind: &str) -> Self {
        Self::Tracer {
            kind: kind.to_owned(),
            law: Box::new(self),
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
    /// Exact tracer source-composition transport.
    Transport,
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
    /// Tracer source-composition transport result.
    Transport(TracerTransportVerdict),
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
            Self::Transport(verdict) => verdict.is_satisfied(),
        }
    }
}

/// One checked pairing of a material channel with its tracer mirror.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TracerTransportChannel {
    /// Material-kind channel whose settled amount drives the tracer.
    pub material: ChannelId,
    /// Mirrored tracer-kind channel.
    pub tracer: ChannelId,
    /// Material source-stock axis.
    pub material_source: AxisId,
    /// Tracer source-stock axis.
    pub tracer_source: AxisId,
}

/// Exact source-composition transport sentence for one tracer kind.
///
/// For every accepted transition and every sourced flow it claims
/// `tracer_settled * material_source_before == material_settled * tracer_source_before`,
/// and that no tracer leaves a stock whose material amount is zero. Together
/// these pin the settled tracer flow to the settled material flow scaled by the
/// source stock's exact pre-transition composition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TracerTransportSentence {
    id: SentenceId,
    kind: String,
    channels: Vec<TracerTransportChannel>,
}

impl TracerTransportSentence {
    /// Stable sentence identifier.
    pub fn id(&self) -> &SentenceId {
        &self.id
    }

    /// Declared tracer kind name.
    pub fn kind(&self) -> &str {
        &self.kind
    }

    /// Checked channel pairings in stable compiled flow order.
    pub fn channels(&self) -> &[TracerTransportChannel] {
        &self.channels
    }
}

/// Positive tracer-transport evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TracerTransportWitness {
    /// Sentence identifier.
    pub sentence: SentenceId,
    /// Declared tracer kind name.
    pub kind: String,
    /// Number of accepted transitions checked.
    pub transitions_checked: usize,
    /// Number of channel pairings checked per transition.
    pub channels_checked: usize,
}

/// First exact tracer-transport mismatch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TracerTransportViolation {
    /// Sentence identifier.
    pub sentence: SentenceId,
    /// First mismatching transition.
    pub transition: usize,
    /// Declared tracer kind name.
    pub kind: String,
    /// Material-kind channel of the mismatching pairing.
    pub material: ChannelId,
    /// Tracer-kind channel of the mismatching pairing.
    pub tracer: ChannelId,
    /// Recorded settled material amount.
    pub material_settled: BigRational,
    /// Recorded settled tracer amount.
    pub tracer_settled: BigRational,
    /// Material source stock before the transition.
    pub material_source_before: BigRational,
    /// Tracer source stock before the transition.
    pub tracer_source_before: BigRational,
}

/// Typed exact tracer-transport result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TracerTransportVerdict {
    /// Every sourced tracer flow kept exact source proportion.
    Satisfied(TracerTransportWitness),
    /// First flow that broke exact source proportion.
    Violated(Box<TracerTransportViolation>),
}

impl TracerTransportVerdict {
    /// Whether the verdict carries positive evidence.
    pub fn is_satisfied(&self) -> bool {
        matches!(self, Self::Satisfied(_))
    }
}

/// Checks one tracer kind's settled flows for exact source-composition transport.
pub fn check_tracer_transport(
    sentence: &TracerTransportSentence,
    trace: &TransitionTrace,
) -> Result<TracerTransportVerdict, TrophicNetworkError> {
    for (transition, record) in trace.records().iter().enumerate() {
        for channel in &sentence.channels {
            let material_settled = settled_channel_amount(record, &channel.material)?;
            let tracer_settled = settled_channel_amount(record, &channel.tracer)?;
            let material_before = before_amount(record, &channel.material_source)?;
            let tracer_before = before_amount(record, &channel.tracer_source)?;
            let proportional = tracer_settled * material_before == material_settled * tracer_before;
            let stranded_moved = material_before.is_zero() && !tracer_settled.is_zero();
            if !proportional || stranded_moved {
                return Ok(TracerTransportVerdict::Violated(Box::new(
                    TracerTransportViolation {
                        sentence: sentence.id.clone(),
                        transition,
                        kind: sentence.kind.clone(),
                        material: channel.material.clone(),
                        tracer: channel.tracer.clone(),
                        material_settled: material_settled.clone(),
                        tracer_settled: tracer_settled.clone(),
                        material_source_before: material_before.clone(),
                        tracer_source_before: tracer_before.clone(),
                    },
                )));
            }
        }
    }
    Ok(TracerTransportVerdict::Satisfied(TracerTransportWitness {
        sentence: sentence.id.clone(),
        kind: sentence.kind.clone(),
        transitions_checked: trace.records().len(),
        channels_checked: sentence.channels.len(),
    }))
}

fn settled_channel_amount<'a>(
    record: &'a TransitionRecord,
    channel: &ChannelId,
) -> Result<&'a BigRational, TrophicNetworkError> {
    match channel {
        ChannelId::Internal(flow) => record.settled_internal().amount(flow),
        ChannelId::Boundary(boundary) => record.settled_boundary().amount(boundary),
    }
    .ok_or_else(|| evidence_error(format!("record lacks transport channel {channel:?}")))
}

fn before_amount<'a>(
    record: &'a TransitionRecord,
    axis: &AxisId,
) -> Result<&'a BigRational, TrophicNetworkError> {
    record
        .before()
        .amount(axis)
        .ok_or_else(|| evidence_error(format!("record lacks source axis {axis}")))
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
struct NamedTracerTransport {
    law: TrophicLaw,
    sentence: TracerTransportSentence,
    symbols: Vec<SymbolId>,
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

/// Cumulative boundary bookkeeping for one conserved kind.
#[derive(Clone, Debug)]
struct CumulativeAxes {
    kind: KindId,
    input_axis: AxisId,
    output_axis: AxisId,
    input_ledger: LedgerId,
}

#[derive(Clone, Debug)]
struct ExactEvidencePlan {
    signature: ConservationSignature,
    carrier: Arc<StockFlowCarrier>,
    /// Every stock axis in full-topology stock order: base block, then one block per tracer kind.
    stock_axes: Vec<AxisId>,
    /// Per-kind cumulative axes and input ledgers: material first, then tracer kinds.
    cumulative_axes: Vec<CumulativeAxes>,
    sentences: Vec<NamedSentence>,
    stock_flow_sentences: Vec<NamedStockFlowSentence>,
    /// One source-composition transport sentence per tracer kind.
    tracer_transport: Vec<NamedTracerTransport>,
    /// Per-kind certificate-derived open balances in `cumulative_axes` order.
    open_balances: Vec<(TrophicLaw, OpenBalance)>,
    /// Every harvest ledger and its settled full-topology flow index.
    harvest_ledger_flows: Vec<(LedgerId, usize)>,
}

impl ExactEvidencePlan {
    fn base_stock_count(&self) -> usize {
        self.stock_axes.len() / self.cumulative_axes.len()
    }

    /// Stock axes of one kind block (0 is the material block).
    fn block_stock_axes(&self, block: usize) -> &[AxisId] {
        let base = self.base_stock_count();
        &self.stock_axes[base * block..base * (block + 1)]
    }

    /// Kind-block index for a tracer kind name.
    fn block_for_tracer(&self, kind: &str) -> Option<usize> {
        self.cumulative_axes
            .iter()
            .position(|cumulative| cumulative.kind.as_str() == kind)
    }
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
    harvest_ledgers: BTreeMap<LedgerId, BigRational>,
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
        &self.state.amounts()[..self.compiled.layout.stock_names.len()]
    }

    /// Declared tracer kinds in stable compiled order.
    pub fn tracer_kinds(&self) -> &[String] {
        &self.compiled.layout.tracer_kinds
    }

    /// Exact tracer amounts for one kind in [`Self::stock_names`] order.
    pub fn tracer_amounts(&self, kind: &str) -> Option<&[BigRational]> {
        let layout = &self.compiled.layout;
        let block = layout.tracer_block(kind)?;
        let start = layout.tracer_stock_index(block, 0);
        Some(&self.state.amounts()[start..start + layout.stock_names.len()])
    }

    /// Exact tracer stock amount by kind and stock name.
    pub fn tracer_stock(&self, kind: &str, name: &str) -> Option<&BigRational> {
        let index = *self.compiled.layout.stock_indices.get(name)?;
        self.tracer_amounts(kind)?.get(index)
    }

    /// Exact cumulative material-kind boundary input.
    pub fn inputs(&self) -> BigRational {
        self.state.inputs(&material_kind())
    }

    /// Exact cumulative material-kind boundary output.
    pub fn outputs(&self) -> BigRational {
        self.state.outputs(&material_kind())
    }

    /// Exact material-kind open-system residual.
    pub fn balance_residual(&self) -> BigRational {
        self.state.balance_residual(&material_kind())
    }

    /// Whether the exact material-kind ledger closes; tracer kinds have their
    /// own residuals via [`Self::tracer_balance_residual`].
    pub fn is_balanced(&self) -> bool {
        self.balance_residual().is_zero()
    }

    /// Exact cumulative tracer boundary input for one declared kind.
    pub fn tracer_inputs(&self, kind: &str) -> Option<BigRational> {
        self.tracer_kind_id(kind)
            .map(|kind| self.state.inputs(kind))
    }

    /// Exact cumulative tracer boundary output for one declared kind.
    pub fn tracer_outputs(&self, kind: &str) -> Option<BigRational> {
        self.tracer_kind_id(kind)
            .map(|kind| self.state.outputs(kind))
    }

    /// Exact open-system residual for one declared tracer kind.
    pub fn tracer_balance_residual(&self, kind: &str) -> Option<BigRational> {
        self.tracer_kind_id(kind)
            .map(|kind| self.state.balance_residual(kind))
    }

    fn tracer_kind_id(&self, kind: &str) -> Option<&KindId> {
        let block = self.compiled.layout.tracer_block(kind)?;
        self.compiled.layout.tracer_kind_ids.get(block)
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
            tracer_kinds: self.compiled.layout.tracer_kinds.clone(),
        }
    }

    /// Settles all growth, feeding, mortality, decomposition, input, and harvest proposals.
    pub fn step(
        &mut self,
        elapsed: BigRational,
        nutrient_input: BigRational,
        harvests: BTreeMap<String, BigRational>,
    ) -> Result<ExactTrophicNetworkStep, TrophicNetworkError> {
        self.step_with_tracer_inputs(elapsed, nutrient_input, BTreeMap::new(), harvests)
    }

    /// Settles one step with explicit per-kind tracer boundary inputs.
    ///
    /// Tracer internal flows and harvests need no forcing: they follow the
    /// settled material flows in exact proportion to each source stock's
    /// pre-transition tracer composition.
    pub fn step_with_tracer_inputs(
        &mut self,
        elapsed: BigRational,
        nutrient_input: BigRational,
        tracer_inputs: BTreeMap<String, BigRational>,
        harvests: BTreeMap<String, BigRational>,
    ) -> Result<ExactTrophicNetworkStep, TrophicNetworkError> {
        ensure_exact_nonnegative(&elapsed)?;
        ensure_exact_nonnegative(&nutrient_input)?;
        for (kind, amount) in &tracer_inputs {
            if self.compiled.layout.tracer_block(kind).is_none() {
                return Err(TrophicNetworkError::UnknownKind(kind.clone()));
            }
            ensure_exact_nonnegative(amount)?;
        }
        validate_exact_harvests(&self.compiled, &harvests)?;
        let requested = exact_requests(
            &self.compiled,
            self.state.amounts(),
            &elapsed,
            nutrient_input,
            &tracer_inputs,
            &harvests,
        );
        let time_before = self.time.clone();
        let stocks_before = self.state.amounts().to_vec();
        let inputs_before = self.inputs();
        let outputs_before = self.outputs();
        let input_ledgers_before = exact_input_ledger_amounts(&self.evidence_plan, &self.state);
        let harvest_ledgers_before = self.harvest_ledgers.clone();
        let mut next_state = self.state.clone();
        let settlement = next_state.settle(&requested)?;
        let mut harvest_ledgers_after = harvest_ledgers_before.clone();
        for (ledger, flow) in &self.evidence_plan.harvest_ledger_flows {
            *harvest_ledgers_after
                .get_mut(ledger)
                .expect("every compiled harvest ledger is initialized") +=
                settlement.applied()[*flow].clone();
        }
        let next_time = &self.time + &elapsed;
        let next_trace_state = exact_trace_state(&self.evidence_plan, &next_state)?;
        let input_ledgers_after = exact_input_ledger_amounts(&self.evidence_plan, &next_state);
        let record = self
            .evidence_plan
            .carrier
            .record_from_settlement(
                &stocks_before,
                next_state.amounts(),
                &settlement,
                exact_ledger_amounts(
                    &self.evidence_plan,
                    &input_ledgers_before,
                    &harvest_ledgers_before,
                )?,
                exact_ledger_amounts(
                    &self.evidence_plan,
                    &input_ledgers_after,
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

    /// Declared tracer kinds in stable compiled order.
    pub fn tracer_kinds(&self) -> &[String] {
        &self.compiled.layout.tracer_kinds
    }

    /// One compiled source-composition transport sentence per tracer kind.
    pub fn tracer_transport_sentences(&self) -> impl Iterator<Item = &TracerTransportSentence> {
        self.evidence
            .tracer_transport
            .iter()
            .map(|named| &named.sentence)
    }

    /// Stable evidence-axis order: physical stocks, then per-kind cumulative input and output.
    pub fn evidence_axis_names(&self) -> impl Iterator<Item = &str> {
        self.evidence
            .stock_axes
            .iter()
            .chain(
                self.evidence
                    .cumulative_axes
                    .iter()
                    .flat_map(|cumulative| [&cumulative.input_axis, &cumulative.output_axis]),
            )
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
        laws.extend(
            self.evidence
                .tracer_transport
                .iter()
                .map(|named| named.law.clone()),
        );
        let mut graded = self
            .evidence
            .sentences
            .iter()
            .map(|named| named.law.clone())
            .collect::<Vec<_>>();
        graded.sort();
        laws.extend(graded);
        laws.extend(
            self.evidence
                .open_balances
                .iter()
                .map(|(law, _)| law.clone()),
        );
        laws
    }

    /// Starts an independent exact run with every tracer stock empty.
    pub fn start(
        &self,
        initial: BTreeMap<String, BigRational>,
    ) -> Result<ExactTrophicNetwork, TrophicNetworkError> {
        self.start_with_tracers(initial, BTreeMap::new())
    }

    /// Starts an independent exact run with explicit per-kind tracer stocks.
    ///
    /// A declared kind absent from `tracers` starts with every tracer stock
    /// zero; a kind that is present must map every compiled stock.
    pub fn start_with_tracers(
        &self,
        initial: BTreeMap<String, BigRational>,
        tracers: BTreeMap<String, BTreeMap<String, BigRational>>,
    ) -> Result<ExactTrophicNetwork, TrophicNetworkError> {
        let mut amounts = exact_initial(&self.compiled.layout, initial)?;
        amounts.extend(tracer_initial(
            &self.compiled.layout,
            tracers,
            BigRational::zero(),
            ensure_exact_nonnegative,
        )?);
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
                .evidence
                .harvest_ledger_flows
                .iter()
                .map(|(ledger, _)| (ledger.clone(), BigRational::zero()))
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
        &self.state.amounts()[..self.compiled.layout.stock_names.len()]
    }

    /// Declared tracer kinds in stable compiled order.
    pub fn tracer_kinds(&self) -> &[String] {
        &self.compiled.layout.tracer_kinds
    }

    /// Dense tracer amounts for one kind in [`Self::stock_names`] order.
    pub fn tracer_amounts(&self, kind: &str) -> Option<&[f64]> {
        let layout = &self.compiled.layout;
        let block = layout.tracer_block(kind)?;
        let start = layout.tracer_stock_index(block, 0);
        Some(&self.state.amounts()[start..start + layout.stock_names.len()])
    }

    /// Dense tracer stock amount by kind and stock name.
    pub fn tracer_stock(&self, kind: &str, name: &str) -> Option<f64> {
        let index = *self.compiled.layout.stock_indices.get(name)?;
        self.tracer_amounts(kind)?.get(index).copied()
    }

    /// Dense cumulative tracer boundary input for one declared kind.
    pub fn tracer_inputs(&self, kind: &str) -> Option<f64> {
        self.tracer_kind_id(kind)
            .map(|kind| self.state.inputs(kind))
    }

    /// Dense cumulative tracer boundary output for one declared kind.
    pub fn tracer_outputs(&self, kind: &str) -> Option<f64> {
        self.tracer_kind_id(kind)
            .map(|kind| self.state.outputs(kind))
    }

    /// Dense open-system residual for one declared tracer kind.
    pub fn tracer_balance_residual(&self, kind: &str) -> Option<f64> {
        self.tracer_kind_id(kind)
            .map(|kind| self.state.balance_residual(kind))
    }

    fn tracer_kind_id(&self, kind: &str) -> Option<&KindId> {
        let block = self.compiled.layout.tracer_block(kind)?;
        self.compiled.layout.tracer_kind_ids.get(block)
    }

    /// Cumulative material-kind boundary input.
    pub fn inputs(&self) -> f64 {
        self.state.inputs(&material_kind())
    }

    /// Cumulative material-kind boundary output.
    pub fn outputs(&self) -> f64 {
        self.state.outputs(&material_kind())
    }

    /// Dense material-kind open-system residual.
    pub fn balance_residual(&self) -> f64 {
        self.state.balance_residual(&material_kind())
    }

    /// Whether the material-kind dense ledger closes within the kernel's explicit tolerance.
    pub fn is_balanced(&self) -> bool {
        self.state
            .balance_within(&material_kind(), trophic_dense_balance_tolerance())
    }

    /// Whether one tracer kind's dense ledger closes within the kernel's explicit tolerance.
    pub fn tracer_is_balanced(&self, kind: &str) -> Option<bool> {
        self.tracer_kind_id(kind).map(|kind| {
            self.state
                .balance_within(kind, trophic_dense_balance_tolerance())
        })
    }

    /// Settles one simultaneous process batch atomically.
    pub fn step(
        &mut self,
        elapsed: f64,
        nutrient_input: f64,
        harvests: BTreeMap<String, f64>,
    ) -> Result<DenseTrophicNetworkStep, TrophicNetworkError> {
        self.step_with_tracer_inputs(elapsed, nutrient_input, BTreeMap::new(), harvests)
    }

    /// Settles one step with explicit per-kind tracer boundary inputs.
    ///
    /// Extreme tracer-to-material compositions can overflow binary64 during
    /// the source-proportion scaling; such a step fails with
    /// [`TrophicNetworkError::NonFiniteAmount`] and leaves the state unchanged.
    pub fn step_with_tracer_inputs(
        &mut self,
        elapsed: f64,
        nutrient_input: f64,
        tracer_inputs: BTreeMap<String, f64>,
        harvests: BTreeMap<String, f64>,
    ) -> Result<DenseTrophicNetworkStep, TrophicNetworkError> {
        for (kind, amount) in &tracer_inputs {
            if self.compiled.layout.tracer_block(kind).is_none() {
                return Err(TrophicNetworkError::UnknownKind(kind.clone()));
            }
            ensure_dense_values(&[*amount])?;
            ensure_dense_nonnegative(*amount)?;
        }
        validate_dense_harvests(&self.compiled, &harvests)?;
        let ordered = self
            .compiled
            .layout
            .consumer_names
            .iter()
            .map(|name| harvests.get(name).copied().unwrap_or(0.0))
            .collect::<Vec<_>>();
        let (next_time, requested) =
            self.prepare_ordered_step(elapsed, nutrient_input, &tracer_inputs, &ordered)?;
        let settlement = self.state.settle(&requested)?;
        self.time = next_time;
        Ok(DenseTrophicNetworkStep {
            layout: Arc::clone(&self.compiled.layout),
            applied: settlement.applied().to_vec(),
            elapsed,
        })
    }

    /// Settles one step with harvests in the compiled consumer order.
    ///
    /// Tracer boundary inputs are zero; tracer stocks still ride the settled
    /// material flows in source proportion.
    pub fn step_ordered(
        &mut self,
        elapsed: f64,
        nutrient_input: f64,
        harvests: &[f64],
    ) -> Result<DenseTrophicNetworkStep, TrophicNetworkError> {
        let (next_time, requested) =
            self.prepare_ordered_step(elapsed, nutrient_input, &BTreeMap::new(), harvests)?;
        let settlement = self.state.settle(&requested)?;
        self.time = next_time;
        Ok(DenseTrophicNetworkStep {
            layout: Arc::clone(&self.compiled.layout),
            applied: settlement.applied().to_vec(),
            elapsed,
        })
    }

    /// Settles an ordered step without allocating owned per-flow observations.
    ///
    /// Tracer boundary inputs are zero; tracer stocks still ride the settled
    /// material flows in source proportion.
    pub fn step_discard_ordered(
        &mut self,
        elapsed: f64,
        nutrient_input: f64,
        harvests: &[f64],
    ) -> Result<(), TrophicNetworkError> {
        let (next_time, requested) =
            self.prepare_ordered_step(elapsed, nutrient_input, &BTreeMap::new(), harvests)?;
        self.state.settle_discard(&requested)?;
        self.time = next_time;
        Ok(())
    }

    fn prepare_ordered_step(
        &self,
        elapsed: f64,
        nutrient_input: f64,
        tracer_inputs: &BTreeMap<String, f64>,
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
            tracer_inputs,
            harvests,
        );
        // Extreme tracer compositions can overflow binary64 during the
        // source-proportion scaling; fail here with a trophic error instead of
        // an opaque kernel rejection.
        ensure_dense_values(&requested)?;
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

    /// Starts an independent dense run with every tracer stock empty.
    pub fn start(
        &self,
        initial: BTreeMap<String, f64>,
    ) -> Result<DenseTrophicNetwork, TrophicNetworkError> {
        let amounts = dense_initial(&self.compiled.layout, initial)?;
        self.start_ordered(amounts)
    }

    /// Starts an independent dense run with explicit per-kind tracer stocks.
    ///
    /// A declared kind absent from `tracers` starts with every tracer stock
    /// zero; a kind that is present must map every compiled stock.
    pub fn start_with_tracers(
        &self,
        initial: BTreeMap<String, f64>,
        tracers: BTreeMap<String, BTreeMap<String, f64>>,
    ) -> Result<DenseTrophicNetwork, TrophicNetworkError> {
        let mut amounts = dense_initial(&self.compiled.layout, initial)?;
        amounts.extend(tracer_initial(
            &self.compiled.layout,
            tracers,
            0.0,
            |amount| {
                ensure_dense_values(&[*amount])?;
                ensure_dense_nonnegative(*amount)
            },
        )?);
        self.start_full(amounts)
    }

    /// Starts a run from amounts in [`Self::stock_names`] order and empty tracers.
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
        let mut amounts = amounts;
        amounts.resize(self.compiled.layout.total_stock_count(), 0.0);
        self.start_full(amounts)
    }

    fn start_full(&self, amounts: Vec<f64>) -> Result<DenseTrophicNetwork, TrophicNetworkError> {
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

/// Suffixes one base identifier when compiling a tracer block, else keeps it.
fn kinded_name(base: &str, tracer: Option<&str>) -> String {
    match tracer {
        None => base.to_owned(),
        Some(kind) => tracer_name(base, kind),
    }
}

/// Wraps one base law when compiling a tracer block, else keeps it.
fn kinded_law(law: TrophicLaw, tracer: Option<&str>) -> TrophicLaw {
    match tracer {
        None => law,
        Some(kind) => law.for_tracer(kind),
    }
}

fn compile_evidence_plan(
    compiled: &CompiledNetwork<BigRational>,
) -> Result<ExactEvidencePlan, TrophicNetworkError> {
    let layout = &compiled.layout;
    // Kind blocks in carrier order: the material block, then one per tracer kind.
    let mut kinds = vec![(None::<&str>, material_kind())];
    kinds.extend(
        layout
            .tracer_kinds
            .iter()
            .map(|name| Some(name.as_str()))
            .zip(layout.tracer_kind_ids.iter().cloned()),
    );
    let base_count = layout.stock_names.len();

    let mut stock_axes = Vec::with_capacity(layout.total_stock_count());
    for (tracer, _) in &kinds {
        for name in &layout.stock_names {
            stock_axes.push(AxisId::new(kinded_name(name, *tracer)).map_err(evidence_error)?);
        }
    }
    let stock_axis_definitions = stock_axes
        .iter()
        .map(|axis| StockAxisDefinition {
            stock: StockId::new(axis.as_str()).expect("compiled stock names are valid"),
            axis: axis.clone(),
        })
        .collect::<Vec<_>>();

    let mut channels = Vec::with_capacity(layout.total_flow_count());
    for (tracer, _) in &kinds {
        for flow in &layout.flow_symbols {
            channels.push(stock_flow_channel_for(flow, *tracer)?);
        }
    }

    let mut cumulative_axes = Vec::with_capacity(kinds.len());
    let mut ledgers = Vec::new();
    let mut harvest_ledger_flows = Vec::new();
    for (block, (tracer, kind)) in kinds.iter().enumerate() {
        let input_name = kinded_name(CUMULATIVE_INPUT, *tracer);
        let input_axis = AxisId::new(input_name.clone()).map_err(evidence_error)?;
        let output_axis =
            AxisId::new(kinded_name(CUMULATIVE_OUTPUT, *tracer)).map_err(evidence_error)?;
        let input_ledger = LedgerId::new(input_name).map_err(evidence_error)?;
        ledgers.push(LedgerDefinition {
            id: input_ledger.clone(),
            axis: input_axis.clone(),
            kind: kind.clone(),
            boundaries: vec![
                BoundaryId::new(kinded_name("nutrient-input", *tracer)).map_err(evidence_error)?,
            ],
        });
        for (consumer, flow) in &layout.harvest_flows {
            let ledger_name = kinded_name(&format!("cumulative_harvest:{consumer}"), *tracer);
            let ledger = LedgerId::new(ledger_name.clone()).map_err(evidence_error)?;
            ledgers.push(LedgerDefinition {
                id: ledger.clone(),
                axis: AxisId::new(ledger_name).map_err(evidence_error)?,
                kind: kind.clone(),
                boundaries: vec![
                    BoundaryId::new(kinded_name(&format!("harvest:{consumer}"), *tracer))
                        .map_err(evidence_error)?,
                ],
            });
            let flow_index = if block == 0 {
                *flow
            } else {
                layout.tracer_flow_index(block - 1, *flow)
            };
            harvest_ledger_flows.push((ledger, flow_index));
        }
        cumulative_axes.push(CumulativeAxes {
            kind: kind.clone(),
            input_axis,
            output_axis,
            input_ledger,
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
    for (block, (tracer, kind)) in kinds.iter().enumerate() {
        for feeding in &feedings {
            let assimilation = FlowId::new(kinded_name(
                &format!(
                    "feeding-assimilated:{}:{}",
                    feeding.consumer, feeding.resource
                ),
                *tracer,
            ))
            .map_err(evidence_error)?;
            let waste = FlowId::new(kinded_name(
                &format!("feeding-waste:{}:{}", feeding.consumer, feeding.resource),
                *tracer,
            ))
            .map_err(evidence_error)?;
            let sentence = LinearFlowConstraint::new(
                &carrier,
                SentenceId::new(kinded_name(
                    &format!(
                        "trophic.feeding-partition:{}:{}",
                        feeding.consumer, feeding.resource
                    ),
                    *tracer,
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
                law: kinded_law(
                    TrophicLaw::FeedingPartition {
                        consumer: feeding.consumer.clone(),
                        resource: feeding.resource.clone(),
                    },
                    *tracer,
                ),
                sentence,
                symbols: vec![SymbolId::Flow(assimilation), SymbolId::Flow(waste)],
            });
        }
        let input_ledger = cumulative_axes[block].input_ledger.clone();
        stock_flow_sentences.push(NamedStockFlowSentence::Boundary {
            law: kinded_law(TrophicLaw::NutrientInputCorrespondence, *tracer),
            sentence: BoundaryCorrespondence::new(
                SentenceId::new(kinded_name(
                    "trophic.nutrient-input-correspondence",
                    *tracer,
                ))
                .map_err(evidence_error)?,
                input_ledger.clone(),
            ),
            symbols: vec![
                SymbolId::Ledger(input_ledger),
                SymbolId::Boundary(
                    BoundaryId::new(kinded_name("nutrient-input", *tracer))
                        .map_err(evidence_error)?,
                ),
            ],
        });
        for consumer in layout.harvest_flows.keys() {
            let ledger = LedgerId::new(kinded_name(
                &format!("cumulative_harvest:{consumer}"),
                *tracer,
            ))
            .map_err(evidence_error)?;
            let boundary = BoundaryId::new(kinded_name(&format!("harvest:{consumer}"), *tracer))
                .map_err(evidence_error)?;
            stock_flow_sentences.push(NamedStockFlowSentence::Boundary {
                law: kinded_law(TrophicLaw::HarvestCorrespondence(consumer.clone()), *tracer),
                sentence: BoundaryCorrespondence::new(
                    SentenceId::new(kinded_name(
                        &format!("trophic.harvest-correspondence:{consumer}"),
                        *tracer,
                    ))
                    .map_err(evidence_error)?,
                    ledger.clone(),
                ),
                symbols: vec![SymbolId::Ledger(ledger), SymbolId::Boundary(boundary)],
            });
        }
    }
    let mut tracer_transport = Vec::with_capacity(kinds.len() - 1);
    for (block, (tracer, _)) in kinds.iter().enumerate() {
        let Some(kind_name) = tracer else {
            continue;
        };
        let mut transport_channels = Vec::new();
        for (base_flow, flow) in layout.topology.flows()[..layout.flow_count]
            .iter()
            .enumerate()
        {
            let Some(source) = flow.source() else {
                continue;
            };
            transport_channels.push(TracerTransportChannel {
                material: stock_flow_channel_for(&layout.flow_symbols[base_flow], None)?,
                tracer: stock_flow_channel_for(&layout.flow_symbols[base_flow], Some(kind_name))?,
                material_source: stock_axes[source].clone(),
                tracer_source: stock_axes[base_count * block + source].clone(),
            });
        }
        let mut symbols = stock_axes[base_count * block..base_count * (block + 1)]
            .iter()
            .cloned()
            .map(SymbolId::Axis)
            .collect::<Vec<_>>();
        symbols.extend(
            transport_channels
                .iter()
                .map(|channel| match &channel.tracer {
                    ChannelId::Internal(flow) => SymbolId::Flow(flow.clone()),
                    ChannelId::Boundary(boundary) => SymbolId::Boundary(boundary.clone()),
                }),
        );
        tracer_transport.push(NamedTracerTransport {
            law: TrophicLaw::TracerTransport.for_tracer(kind_name),
            sentence: TracerTransportSentence {
                id: SentenceId::new(tracer_name("trophic.tracer-transport", kind_name))
                    .map_err(evidence_error)?,
                kind: (*kind_name).to_owned(),
                channels: transport_channels,
            },
            symbols,
        });
    }

    let mut signature_axes = Vec::new();
    for (block, (_, kind)) in kinds.iter().enumerate() {
        for axis in &stock_axes[base_count * block..base_count * (block + 1)] {
            signature_axes.push((axis.clone(), kind.clone()));
        }
    }
    for cumulative in &cumulative_axes {
        signature_axes.push((cumulative.input_axis.clone(), cumulative.kind.clone()));
        signature_axes.push((cumulative.output_axis.clone(), cumulative.kind.clone()));
    }
    let signature = ConservationSignature::new(signature_axes).map_err(evidence_error)?;

    let mut open_balances = Vec::with_capacity(kinds.len());
    let mut sentences = Vec::new();
    for (block, (tracer, kind)) in kinds.iter().enumerate() {
        let block_axes = &stock_axes[base_count * block..base_count * (block + 1)];
        let certificate = certify_nullspace(
            &carrier,
            kind.clone(),
            block_axes
                .iter()
                .cloned()
                .map(|axis| (axis, BigRational::one())),
        )
        .map_err(evidence_error)?;
        open_balances.push((
            kinded_law(TrophicLaw::OpenMaterialBalance, *tracer),
            certificate.open_balance(
                SentenceId::new(kinded_name("trophic.open-material-balance", *tracer))
                    .map_err(evidence_error)?,
            ),
        ));

        let cumulative = &cumulative_axes[block];
        let mut total_coefficients = block_axes
            .iter()
            .cloned()
            .map(|axis| (axis, BigRational::one()))
            .collect::<Vec<_>>();
        total_coefficients.push((cumulative.input_axis.clone(), -BigRational::one()));
        total_coefficients.push((cumulative.output_axis.clone(), BigRational::one()));
        let invariant = BalanceLaw::new(kind.clone(), total_coefficients, Provenance::Declared)
            .map_err(evidence_error)?;
        sentences.push(NamedSentence {
            law: kinded_law(TrophicLaw::MaterialInvariant, *tracer),
            sentence: GradedLaw::new(invariant, Grade::Invariant),
        });

        for (name, axis) in layout.stock_names.iter().zip(block_axes) {
            let form = BalanceLaw::new(
                kind.clone(),
                [(axis.clone(), BigRational::one())],
                Provenance::Declared,
            )
            .map_err(evidence_error)?;
            sentences.push(NamedSentence {
                law: kinded_law(TrophicLaw::StockNonnegative(name.clone()), *tracer),
                sentence: GradedLaw::new(form, Grade::Nonnegative),
            });
        }
        for (law, axis) in [
            (
                TrophicLaw::CumulativeInputNondecreasing,
                cumulative.input_axis.clone(),
            ),
            (
                TrophicLaw::CumulativeOutputNondecreasing,
                cumulative.output_axis.clone(),
            ),
        ] {
            let form = BalanceLaw::new(
                kind.clone(),
                [(axis, BigRational::one())],
                Provenance::Declared,
            )
            .map_err(evidence_error)?;
            sentences.push(NamedSentence {
                law: kinded_law(law, *tracer),
                sentence: GradedLaw::new(form, Grade::Nondecreasing),
            });
        }
    }

    Ok(ExactEvidencePlan {
        signature,
        carrier,
        stock_axes,
        cumulative_axes,
        sentences,
        stock_flow_sentences,
        tracer_transport,
        open_balances,
        harvest_ledger_flows,
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

/// Compiles one base flow's carrier channel, suffixed for a tracer block.
fn stock_flow_channel_for(
    flow: &TrophicFlow,
    tracer: Option<&str>,
) -> Result<ChannelId, TrophicNetworkError> {
    let channel = stock_flow_channel(flow)?;
    let Some(kind) = tracer else {
        return Ok(channel);
    };
    match channel {
        ChannelId::Internal(id) => FlowId::new(tracer_name(id.as_str(), kind))
            .map(ChannelId::Internal)
            .map_err(evidence_error),
        ChannelId::Boundary(id) => BoundaryId::new(tracer_name(id.as_str(), kind))
            .map(ChannelId::Boundary)
            .map_err(evidence_error),
    }
}

/// Cumulative per-kind boundary inputs keyed by input-ledger identity.
fn exact_input_ledger_amounts(
    plan: &ExactEvidencePlan,
    state: &ExactState,
) -> BTreeMap<LedgerId, BigRational> {
    plan.cumulative_axes
        .iter()
        .map(|cumulative| {
            (
                cumulative.input_ledger.clone(),
                state.inputs(&cumulative.kind),
            )
        })
        .collect()
}

fn exact_ledger_amounts(
    plan: &ExactEvidencePlan,
    inputs: &BTreeMap<LedgerId, BigRational>,
    harvests: &BTreeMap<LedgerId, BigRational>,
) -> Result<ExactAmounts<LedgerId>, TrophicNetworkError> {
    ExactAmounts::new(
        plan.carrier
            .identity()
            .ledgers()
            .iter()
            .map(|(ledger, identity)| {
                let amount = inputs
                    .get(ledger)
                    .or_else(|| harvests.get(ledger))
                    .expect("every carrier ledger is a cumulative input or harvest ledger")
                    .clone();
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
    for cumulative in &plan.cumulative_axes {
        values.push((
            cumulative.input_axis.clone(),
            state.inputs(&cumulative.kind),
        ));
        values.push((
            cumulative.output_axis.clone(),
            state.outputs(&cumulative.kind),
        ));
    }
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

    for named in &plan.tracer_transport {
        laws.push(CompleteTrophicLawEvidence {
            law: named.law.clone(),
            family: TrophicSentenceFamily::Transport,
            symbols: named.symbols.clone(),
            grade: None,
            verdict: TrophicLawVerdict::Transport(check_tracer_transport(
                &named.sentence,
                &transition_trace,
            )?),
        });
    }

    let mut graded = evaluate_evidence(plan, states)?.laws;
    graded.sort_by(|left, right| left.law.cmp(&right.law));
    for evidence in graded {
        let symbols = match evidence.law.axis_name() {
            Some(axis) => vec![SymbolId::Axis(
                AxisId::new(axis).expect("compiled trophic evidence axes are valid"),
            )],
            None => {
                let block = invariant_block(plan, &evidence.law);
                let cumulative = &plan.cumulative_axes[block];
                let mut axes = plan
                    .block_stock_axes(block)
                    .iter()
                    .cloned()
                    .chain([
                        cumulative.input_axis.clone(),
                        cumulative.output_axis.clone(),
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

    for (block, (law, open_balance)) in plan.open_balances.iter().enumerate() {
        let kind = &plan.cumulative_axes[block].kind;
        let mut open_axes = plan.block_stock_axes(block).to_vec();
        open_axes.sort();
        let mut open_symbols = open_axes
            .into_iter()
            .map(SymbolId::Axis)
            .collect::<Vec<_>>();
        open_symbols.extend(
            plan.carrier
                .boundary_effects()
                .columns()
                .filter(|column| plan.carrier.boundary_effects().column_kind(column) == Some(kind))
                .cloned()
                .map(SymbolId::Boundary),
        );
        laws.push(CompleteTrophicLawEvidence {
            law: law.clone(),
            family: TrophicSentenceFamily::OpenBalance,
            symbols: open_symbols,
            grade: None,
            verdict: TrophicLawVerdict::OpenBalance(
                check_open_balance(open_balance, &transition_trace).map_err(evidence_error)?,
            ),
        });
    }
    Ok(ExactTrophicLawSuiteEvidence { laws })
}

/// Kind-block index of one graded invariant law (0 is the material block).
fn invariant_block(plan: &ExactEvidencePlan, law: &TrophicLaw) -> usize {
    match law {
        TrophicLaw::Tracer { kind, .. } => plan
            .block_for_tracer(kind)
            .expect("compiled tracer laws name declared kinds"),
        _ => 0,
    }
}

fn evidence_error(error: impl fmt::Display) -> TrophicNetworkError {
    TrophicNetworkError::Evidence(error.to_string())
}

fn compile_network<N: Clone>(
    spec: TrophicNetworkSpec<N>,
) -> Result<CompiledNetwork<N>, TrophicNetworkError> {
    validate_structure(&spec)?;
    // Canonicalize kind order so every compiled artifact — axes, blocks, and
    // the law suite — is identical across declaration-order permutations.
    let mut spec = spec;
    spec.tracer_kinds.sort();
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
    let mut stock_definitions = stock_names
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
    let tracer_kind_ids = spec
        .tracer_kinds
        .iter()
        .map(|name| KindId::new(name).expect("tracer kind names were validated"))
        .collect::<Vec<_>>();
    for (tracer, tracer_kind) in spec.tracer_kinds.iter().zip(&tracer_kind_ids) {
        for name in &stock_names[..stock_names.len()] {
            stock_definitions.push(StockDefinition {
                id: StockId::new(tracer_name(name, tracer))
                    .expect("suffixed stock identifiers are nonblank"),
                kind: tracer_kind.clone(),
            });
        }
    }
    let base_flows = flows[..flow_count].to_vec();
    for (tracer, tracer_kind) in spec.tracer_kinds.iter().zip(&tracer_kind_ids) {
        for flow in &base_flows {
            flows.push(FlowSpec {
                process: ProcessId::new(tracer_name(flow.process.as_str(), tracer))
                    .expect("suffixed process identifiers are nonblank"),
                kind: tracer_kind.clone(),
                source: flow.source.as_ref().map(|stock| {
                    StockId::new(tracer_name(stock.as_str(), tracer))
                        .expect("suffixed stock identifiers are nonblank")
                }),
                target: flow.target.as_ref().map(|stock| {
                    StockId::new(tracer_name(stock.as_str(), tracer))
                        .expect("suffixed stock identifiers are nonblank")
                }),
            });
        }
    }
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
        tracer_kinds: spec.tracer_kinds.clone(),
        tracer_kind_ids,
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
    let mut kinds = BTreeSet::new();
    for kind in &spec.tracer_kinds {
        if KindId::new(kind).is_err() || kind == MATERIAL || kind.contains(KIND_SEPARATOR) {
            return Err(TrophicNetworkError::InvalidKindName(kind.clone()));
        }
        if !kinds.insert(kind.as_str()) {
            return Err(TrophicNetworkError::DuplicateKind(kind.clone()));
        }
    }
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
        if !spec.tracer_kinds.is_empty() && name.contains(KIND_SEPARATOR) {
            return Err(TrophicNetworkError::InvalidStockName(name.clone()));
        }
        // The harvest-ledger axis namespace is reserved: a stock named into it
        // would collide with `cumulative_harvest:{consumer}` evidence axes.
        if name == "cumulative_harvest" || name.starts_with("cumulative_harvest:") {
            return Err(TrophicNetworkError::InvalidStockName(name.clone()));
        }
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

/// Validates per-kind tracer stocks and lays them out in tracer-block order.
///
/// A declared kind absent from `tracers` starts all-zero; a present kind must
/// map every compiled stock, mirroring the base initial-state contract.
fn tracer_initial<N: Clone>(
    layout: &NetworkLayout,
    tracers: BTreeMap<String, BTreeMap<String, N>>,
    zero: N,
    validate: impl Fn(&N) -> Result<(), TrophicNetworkError>,
) -> Result<Vec<N>, TrophicNetworkError> {
    for kind in tracers.keys() {
        if layout.tracer_block(kind).is_none() {
            return Err(TrophicNetworkError::UnknownKind(kind.clone()));
        }
    }
    let mut amounts = Vec::with_capacity(layout.stock_names.len() * layout.tracer_kinds.len());
    for kind in &layout.tracer_kinds {
        let Some(initial) = tracers.get(kind) else {
            amounts.extend(std::iter::repeat_n(zero.clone(), layout.stock_names.len()));
            continue;
        };
        for name in initial.keys() {
            if !layout.stock_indices.contains_key(name) {
                return Err(TrophicNetworkError::UnknownInitialStock(tracer_name(
                    name, kind,
                )));
            }
        }
        for name in &layout.stock_names {
            let Some(amount) = initial.get(name) else {
                return Err(TrophicNetworkError::MissingInitialStock(tracer_name(
                    name, kind,
                )));
            };
            validate(amount)?;
            amounts.push(amount.clone());
        }
    }
    Ok(amounts)
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
    tracer_inputs: &BTreeMap<String, BigRational>,
    harvests: &BTreeMap<String, BigRational>,
) -> Vec<BigRational> {
    let mut requested = vec![BigRational::zero(); compiled.layout.total_flow_count()];
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
    // Tracer kinds ride the material flow field: each tracer flow proposes the
    // base request scaled by the source stock's pre-transition composition.
    // The kernel's per-source rationing applies the same exact scale to every
    // outflow of a stock, so settled tracer flows keep this proportion exactly.
    for (block, kind) in compiled.layout.tracer_kinds.iter().enumerate() {
        if let Some(amount) = tracer_inputs.get(kind) {
            requested[compiled
                .layout
                .tracer_flow_index(block, compiled.layout.nutrient_input_flow)] = amount.clone();
        }
        for (base_flow, flow) in compiled.layout.topology.flows()[..compiled.layout.flow_count]
            .iter()
            .enumerate()
        {
            let Some(source) = flow.source() else {
                continue;
            };
            if requested[base_flow].is_zero() || amounts[source].is_zero() {
                continue;
            }
            let tracer_stock = compiled.layout.tracer_stock_index(block, source);
            let amount = &requested[base_flow] * &amounts[tracer_stock] / &amounts[source];
            requested[compiled.layout.tracer_flow_index(block, base_flow)] = amount;
        }
    }
    requested
}

fn dense_requests(
    compiled: &CompiledNetwork<f64>,
    amounts: &[f64],
    elapsed: f64,
    nutrient_input: f64,
    tracer_inputs: &BTreeMap<String, f64>,
    harvests: &[f64],
) -> Vec<f64> {
    let mut requested = vec![0.0; compiled.layout.total_flow_count()];
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
    // Dense tracer transport mirrors the exact rule; the proportionality is
    // only as exact as binary64 division, so the exact runtime stays the oracle.
    for (block, kind) in compiled.layout.tracer_kinds.iter().enumerate() {
        if let Some(amount) = tracer_inputs.get(kind) {
            requested[compiled
                .layout
                .tracer_flow_index(block, compiled.layout.nutrient_input_flow)] = *amount;
        }
        for (base_flow, flow) in compiled.layout.topology.flows()[..compiled.layout.flow_count]
            .iter()
            .enumerate()
        {
            let Some(source) = flow.source() else {
                continue;
            };
            if requested[base_flow] == 0.0 || amounts[source] == 0.0 {
                continue;
            }
            let tracer_stock = compiled.layout.tracer_stock_index(block, source);
            requested[compiled.layout.tracer_flow_index(block, base_flow)] =
                requested[base_flow] * amounts[tracer_stock] / amounts[source];
        }
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
        for cumulative in &plan.cumulative_axes {
            values.push((cumulative.input_axis.clone(), integer(cumulative_input)));
            values.push((cumulative.output_axis.clone(), integer(cumulative_output)));
        }
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
