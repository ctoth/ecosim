//! Exact and dense nutrient-producer-consumer-detritus process models.

use std::error::Error;
use std::fmt;
use std::sync::{Arc, OnceLock};

use conservation_core::KindId;
use conservation_dynamics::{
    CompiledSettlementReport, DenseState, DenseTolerance, ExactState, FlowSpec, FlowTopology,
    ProcessId, SettlementReport, StockDefinition, StockFlowError, StockId,
};
use num_rational::BigRational;
use num_traits::{Signed, Zero};

const NUTRIENT: &str = "nutrient";
const PRODUCER: &str = "producer";
const CONSUMER: &str = "consumer";
const DETRITUS: &str = "detritus";
const MATERIAL: &str = "material-equivalent";

const NUTRIENT_INPUT: &str = "nutrient-input";
const PRODUCER_GROWTH: &str = "producer-growth";
const GRAZING: &str = "grazing";
const PRODUCER_MORTALITY: &str = "producer-mortality";
const CONSUMER_MORTALITY: &str = "consumer-mortality";
const DECOMPOSITION: &str = "decomposition";
const HARVEST: &str = "harvest";

const NUTRIENT_INDEX: usize = 0;
const PRODUCER_INDEX: usize = 1;
const CONSUMER_INDEX: usize = 2;
const DETRITUS_INDEX: usize = 3;

/// Exact parameters for the four-stock process model.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoodWebParameters {
    max_growth: BigRational,
    nutrient_half_saturation: BigRational,
    max_grazing: BigRational,
    producer_half_saturation: BigRational,
    producer_mortality: BigRational,
    consumer_mortality: BigRational,
    decomposition: BigRational,
}

impl FoodWebParameters {
    /// Validates a parameter set. Half-saturation constants must be positive;
    /// all rates must be nonnegative.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        max_growth: BigRational,
        nutrient_half_saturation: BigRational,
        max_grazing: BigRational,
        producer_half_saturation: BigRational,
        producer_mortality: BigRational,
        consumer_mortality: BigRational,
        decomposition: BigRational,
    ) -> Result<Self, FoodWebError> {
        for rate in [
            &max_growth,
            &max_grazing,
            &producer_mortality,
            &consumer_mortality,
            &decomposition,
        ] {
            ensure_nonnegative(rate)?;
        }
        if !nutrient_half_saturation.is_positive() || !producer_half_saturation.is_positive() {
            return Err(FoodWebError::NonpositiveHalfSaturation);
        }
        Ok(Self {
            max_growth,
            nutrient_half_saturation,
            max_grazing,
            producer_half_saturation,
            producer_mortality,
            consumer_mortality,
            decomposition,
        })
    }

    /// Maximum producer growth rate.
    pub fn max_growth(&self) -> &BigRational {
        &self.max_growth
    }

    /// Nutrient level at half the maximum producer growth response.
    pub fn nutrient_half_saturation(&self) -> &BigRational {
        &self.nutrient_half_saturation
    }

    /// Maximum consumer grazing rate.
    pub fn max_grazing(&self) -> &BigRational {
        &self.max_grazing
    }

    /// Producer level at half the maximum grazing response.
    pub fn producer_half_saturation(&self) -> &BigRational {
        &self.producer_half_saturation
    }

    /// Producer mortality rate.
    pub fn producer_mortality(&self) -> &BigRational {
        &self.producer_mortality
    }

    /// Consumer mortality rate.
    pub fn consumer_mortality(&self) -> &BigRational {
        &self.consumer_mortality
    }

    /// Detritus decomposition rate.
    pub fn decomposition(&self) -> &BigRational {
        &self.decomposition
    }
}

impl Default for FoodWebParameters {
    fn default() -> Self {
        Self {
            max_growth: ratio(1, 2),
            nutrient_half_saturation: ratio(10, 1),
            max_grazing: ratio(2, 5),
            producer_half_saturation: ratio(10, 1),
            producer_mortality: ratio(1, 20),
            consumer_mortality: ratio(1, 25),
            decomposition: ratio(1, 10),
        }
    }
}

/// Binary64 parameters for the same four-stock process model.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DenseFoodWebParameters {
    /// Maximum producer growth rate.
    max_growth: f64,
    /// Nutrient level at half the maximum producer growth response.
    nutrient_half_saturation: f64,
    /// Maximum consumer grazing rate.
    max_grazing: f64,
    /// Producer level at half the maximum grazing response.
    producer_half_saturation: f64,
    /// Producer mortality rate.
    producer_mortality: f64,
    /// Consumer mortality rate.
    consumer_mortality: f64,
    /// Detritus decomposition rate.
    decomposition: f64,
}

impl DenseFoodWebParameters {
    /// Validates finite rates and strictly positive half-saturation constants.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        max_growth: f64,
        nutrient_half_saturation: f64,
        max_grazing: f64,
        producer_half_saturation: f64,
        producer_mortality: f64,
        consumer_mortality: f64,
        decomposition: f64,
    ) -> Result<Self, FoodWebError> {
        let values = [
            max_growth,
            nutrient_half_saturation,
            max_grazing,
            producer_half_saturation,
            producer_mortality,
            consumer_mortality,
            decomposition,
        ];
        ensure_dense_finite(&values)?;
        for rate in [
            max_growth,
            max_grazing,
            producer_mortality,
            consumer_mortality,
            decomposition,
        ] {
            ensure_dense_nonnegative(rate)?;
        }
        if nutrient_half_saturation <= 0.0 || producer_half_saturation <= 0.0 {
            return Err(FoodWebError::NonpositiveHalfSaturation);
        }
        Ok(Self {
            max_growth,
            nutrient_half_saturation,
            max_grazing,
            producer_half_saturation,
            producer_mortality,
            consumer_mortality,
            decomposition,
        })
    }

    /// Maximum producer growth rate.
    pub fn max_growth(self) -> f64 {
        self.max_growth
    }

    /// Nutrient level at half the maximum producer growth response.
    pub fn nutrient_half_saturation(self) -> f64 {
        self.nutrient_half_saturation
    }

    /// Maximum consumer grazing rate.
    pub fn max_grazing(self) -> f64 {
        self.max_grazing
    }

    /// Producer level at half the maximum grazing response.
    pub fn producer_half_saturation(self) -> f64 {
        self.producer_half_saturation
    }

    /// Producer mortality rate.
    pub fn producer_mortality(self) -> f64 {
        self.producer_mortality
    }

    /// Consumer mortality rate.
    pub fn consumer_mortality(self) -> f64 {
        self.consumer_mortality
    }

    /// Detritus decomposition rate.
    pub fn decomposition(self) -> f64 {
        self.decomposition
    }
}

impl Default for DenseFoodWebParameters {
    fn default() -> Self {
        Self {
            max_growth: 0.5,
            nutrient_half_saturation: 10.0,
            max_grazing: 0.4,
            producer_half_saturation: 10.0,
            producer_mortality: 0.05,
            consumer_mortality: 0.04,
            decomposition: 0.1,
        }
    }
}

/// An invalid model construction or step.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum FoodWebError {
    /// A stock, rate, step duration, input, or harvest was negative.
    NegativeAmount,
    /// A half-saturation constant was zero or negative.
    NonpositiveHalfSaturation,
    /// The generic stock-flow foundation rejected internally generated data.
    Settlement(StockFlowError),
}

impl fmt::Display for FoodWebError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NegativeAmount => {
                formatter.write_str("amounts, rates, and duration must be nonnegative")
            }
            Self::NonpositiveHalfSaturation => {
                formatter.write_str("half-saturation constants must be positive")
            }
            Self::Settlement(error) => error.fmt(formatter),
        }
    }
}

impl Error for FoodWebError {}

impl From<StockFlowError> for FoodWebError {
    fn from(value: StockFlowError) -> Self {
        Self::Settlement(value)
    }
}

/// Exact observations from one settled model step.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoodWebStep {
    settlement: SettlementReport,
    elapsed: BigRational,
}

impl FoodWebStep {
    /// Exact duration of this step.
    pub fn elapsed(&self) -> &BigRational {
        &self.elapsed
    }

    /// Exact amount settled for a named process, or zero if absent.
    pub fn applied(&self, process: &str) -> BigRational {
        ProcessId::new(process)
            .map(|process| self.settlement.applied_by(&process))
            .unwrap_or_default()
    }

    /// Gives access to the complete typed settlement evidence.
    pub fn settlement(&self) -> &SettlementReport {
        &self.settlement
    }
}

/// A well-mixed, discrete-time four-stock food-web model using exact arithmetic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoodWeb {
    state: ExactState,
    parameters: FoodWebParameters,
    time: BigRational,
}

impl FoodWeb {
    /// Constructs a model with exact nonnegative initial stocks.
    pub fn new(
        nutrient: BigRational,
        producer: BigRational,
        consumer: BigRational,
        detritus: BigRational,
        parameters: FoodWebParameters,
    ) -> Result<Self, FoodWebError> {
        for amount in [&nutrient, &producer, &consumer, &detritus] {
            ensure_nonnegative(amount)?;
        }
        Ok(Self {
            state: ExactState::new(
                food_web_topology(),
                vec![nutrient, producer, consumer, detritus],
            )?,
            parameters,
            time: BigRational::zero(),
        })
    }

    /// Constructs a model using the documented default process rates.
    pub fn with_defaults(
        nutrient: BigRational,
        producer: BigRational,
        consumer: BigRational,
        detritus: BigRational,
    ) -> Result<Self, FoodWebError> {
        Self::new(
            nutrient,
            producer,
            consumer,
            detritus,
            FoodWebParameters::default(),
        )
    }

    /// Exact model time after accepted steps.
    pub fn time(&self) -> &BigRational {
        &self.time
    }

    /// Exact amount in a named model stock.
    pub fn stock(&self, name: &str) -> Option<&BigRational> {
        StockId::new(name)
            .ok()
            .and_then(|stock| self.state.amount(&stock))
    }

    /// Exact cumulative material-equivalent boundary input.
    pub fn inputs(&self) -> BigRational {
        self.state.inputs(&material_kind())
    }

    /// Exact cumulative material-equivalent boundary output.
    pub fn outputs(&self) -> BigRational {
        self.state.outputs(&material_kind())
    }

    /// Exact open-system accounting residual.
    pub fn balance_residual(&self) -> BigRational {
        self.state.balance_residual(&material_kind())
    }

    /// Whether the exact open-system accounting residual is zero.
    pub fn is_balanced(&self) -> bool {
        self.balance_residual().is_zero()
    }

    /// Proposes all mechanisms from the pre-step state and settles them simultaneously.
    pub fn step(
        &mut self,
        elapsed: BigRational,
        nutrient_input: BigRational,
        harvest: BigRational,
    ) -> Result<FoodWebStep, FoodWebError> {
        for amount in [&elapsed, &nutrient_input, &harvest] {
            ensure_nonnegative(amount)?;
        }
        let amounts = self.state.amounts();
        let requested =
            exact_requests(amounts, &self.parameters, &elapsed, nutrient_input, harvest);
        let mut next_state = self.state.clone();
        let compiled = next_state.settle(&requested)?;
        let settlement = next_state.topology().materialize_exact_report(&compiled)?;
        self.state = next_state;
        self.time += &elapsed;
        Ok(FoodWebStep {
            settlement,
            elapsed,
        })
    }
}

/// Observations from one dense model step.
#[derive(Clone, Debug, PartialEq)]
pub struct DenseFoodWebStep {
    settlement: CompiledSettlementReport<f64>,
    elapsed: f64,
}

impl DenseFoodWebStep {
    /// Duration of this step.
    pub fn elapsed(&self) -> f64 {
        self.elapsed
    }

    /// Amount settled for a named process, or zero if absent.
    pub fn applied(&self, process: &str) -> f64 {
        process_flow_index(process)
            .and_then(|index| self.settlement.applied().get(index))
            .copied()
            .unwrap_or(0.0)
    }

    /// Complete requested and applied arrays in stable flow-slot order.
    pub fn settlement(&self) -> &CompiledSettlementReport<f64> {
        &self.settlement
    }
}

/// A binary64 twin of [`FoodWeb`] for long trajectories and ensembles.
#[derive(Clone, Debug, PartialEq)]
pub struct DenseFoodWeb {
    state: DenseState,
    parameters: DenseFoodWebParameters,
    time: f64,
}

impl DenseFoodWeb {
    /// Constructs a model with finite, nonnegative initial stocks.
    pub fn new(
        nutrient: f64,
        producer: f64,
        consumer: f64,
        detritus: f64,
        parameters: DenseFoodWebParameters,
    ) -> Result<Self, FoodWebError> {
        Ok(Self {
            state: DenseState::new(
                food_web_topology(),
                vec![nutrient, producer, consumer, detritus],
            )?,
            parameters,
            time: 0.0,
        })
    }

    /// Constructs a model using the same rates as [`FoodWeb::with_defaults`].
    pub fn with_defaults(
        nutrient: f64,
        producer: f64,
        consumer: f64,
        detritus: f64,
    ) -> Result<Self, FoodWebError> {
        Self::new(
            nutrient,
            producer,
            consumer,
            detritus,
            DenseFoodWebParameters::default(),
        )
    }

    /// Model time after accepted steps.
    pub fn time(&self) -> f64 {
        self.time
    }

    /// Amount in a named model stock.
    pub fn stock(&self, name: &str) -> Option<f64> {
        StockId::new(name)
            .ok()
            .and_then(|stock| self.state.amount(&stock))
    }

    /// Stock amounts in nutrient, producer, consumer, detritus order.
    pub fn amounts(&self) -> &[f64] {
        self.state.amounts()
    }

    /// Cumulative material-equivalent boundary input.
    pub fn inputs(&self) -> f64 {
        self.state.inputs(&material_kind())
    }

    /// Cumulative material-equivalent boundary output.
    pub fn outputs(&self) -> f64 {
        self.state.outputs(&material_kind())
    }

    /// Floating-point open-system accounting residual.
    pub fn balance_residual(&self) -> f64 {
        self.state.balance_residual(&material_kind())
    }

    /// Whether accounting is balanced within the dense engine's explicit budget.
    pub fn is_balanced(&self) -> bool {
        self.state
            .balance_within(&material_kind(), DenseTolerance::default())
    }

    /// Settles one simultaneous dense process batch atomically.
    pub fn step(
        &mut self,
        elapsed: f64,
        nutrient_input: f64,
        harvest: f64,
    ) -> Result<DenseFoodWebStep, FoodWebError> {
        let (next_time, requested) = self.prepare_step(elapsed, nutrient_input, harvest)?;
        let settlement = self.state.settle(&requested)?;
        self.time = next_time;
        Ok(DenseFoodWebStep {
            settlement,
            elapsed,
        })
    }

    /// Settles one step without allocating owned per-flow evidence.
    ///
    /// This is the trajectory-oriented twin of [`Self::step`]. State,
    /// accounting, validation, and process semantics are identical.
    pub fn step_discard(
        &mut self,
        elapsed: f64,
        nutrient_input: f64,
        harvest: f64,
    ) -> Result<(), FoodWebError> {
        let (next_time, requested) = self.prepare_step(elapsed, nutrient_input, harvest)?;
        self.state.settle_discard(&requested)?;
        self.time = next_time;
        Ok(())
    }

    fn prepare_step(
        &self,
        elapsed: f64,
        nutrient_input: f64,
        harvest: f64,
    ) -> Result<(f64, Vec<f64>), FoodWebError> {
        ensure_dense_finite(&[elapsed, nutrient_input, harvest])?;
        for value in [elapsed, nutrient_input, harvest] {
            ensure_dense_nonnegative(value)?;
        }
        let next_time = self.time + elapsed;
        if !next_time.is_finite() {
            return Err(StockFlowError::ArithmeticOverflow.into());
        }
        Ok((
            next_time,
            dense_requests(
                self.state.amounts(),
                self.parameters,
                elapsed,
                nutrient_input,
                harvest,
            ),
        ))
    }
}

fn exact_requests(
    amounts: &[BigRational],
    parameters: &FoodWebParameters,
    elapsed: &BigRational,
    nutrient_input: BigRational,
    harvest: BigRational,
) -> Vec<BigRational> {
    let nutrient = &amounts[NUTRIENT_INDEX];
    let producer = &amounts[PRODUCER_INDEX];
    let consumer = &amounts[CONSUMER_INDEX];
    let detritus = &amounts[DETRITUS_INDEX];
    vec![
        nutrient_input,
        saturating_process(
            &parameters.max_growth,
            nutrient,
            &parameters.nutrient_half_saturation,
            producer,
            elapsed,
        ),
        saturating_process(
            &parameters.max_grazing,
            producer,
            &parameters.producer_half_saturation,
            consumer,
            elapsed,
        ),
        &parameters.producer_mortality * producer * elapsed,
        &parameters.consumer_mortality * consumer * elapsed,
        &parameters.decomposition * detritus * elapsed,
        harvest,
    ]
}

fn dense_requests(
    amounts: &[f64],
    parameters: DenseFoodWebParameters,
    elapsed: f64,
    nutrient_input: f64,
    harvest: f64,
) -> Vec<f64> {
    let nutrient = amounts[NUTRIENT_INDEX];
    let producer = amounts[PRODUCER_INDEX];
    let consumer = amounts[CONSUMER_INDEX];
    let detritus = amounts[DETRITUS_INDEX];
    vec![
        nutrient_input,
        dense_saturating_process(
            parameters.max_growth,
            nutrient,
            parameters.nutrient_half_saturation,
            producer,
            elapsed,
        ),
        dense_saturating_process(
            parameters.max_grazing,
            producer,
            parameters.producer_half_saturation,
            consumer,
            elapsed,
        ),
        parameters.producer_mortality * producer * elapsed,
        parameters.consumer_mortality * consumer * elapsed,
        parameters.decomposition * detritus * elapsed,
        harvest,
    ]
}

fn dense_saturating_process(
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

fn food_web_topology() -> Arc<FlowTopology> {
    static TOPOLOGY: OnceLock<Arc<FlowTopology>> = OnceLock::new();
    Arc::clone(TOPOLOGY.get_or_init(|| {
        let kind = material_kind();
        Arc::new(
            FlowTopology::new(
                [
                    stock_definition(NUTRIENT, &kind),
                    stock_definition(PRODUCER, &kind),
                    stock_definition(CONSUMER, &kind),
                    stock_definition(DETRITUS, &kind),
                ],
                flow_specs(),
            )
            .expect("the fixed food-web topology is valid"),
        )
    }))
}

fn flow_specs() -> [FlowSpec; 7] {
    let kind = material_kind();
    [
        flow_spec(NUTRIENT_INPUT, None, Some(NUTRIENT), &kind),
        flow_spec(PRODUCER_GROWTH, Some(NUTRIENT), Some(PRODUCER), &kind),
        flow_spec(GRAZING, Some(PRODUCER), Some(CONSUMER), &kind),
        flow_spec(PRODUCER_MORTALITY, Some(PRODUCER), Some(DETRITUS), &kind),
        flow_spec(CONSUMER_MORTALITY, Some(CONSUMER), Some(DETRITUS), &kind),
        flow_spec(DECOMPOSITION, Some(DETRITUS), Some(NUTRIENT), &kind),
        flow_spec(HARVEST, Some(CONSUMER), None, &kind),
    ]
}

fn process_flow_index(process: &str) -> Option<usize> {
    [
        NUTRIENT_INPUT,
        PRODUCER_GROWTH,
        GRAZING,
        PRODUCER_MORTALITY,
        CONSUMER_MORTALITY,
        DECOMPOSITION,
        HARVEST,
    ]
    .iter()
    .position(|candidate| *candidate == process)
}

fn stock_definition(name: &str, kind: &KindId) -> StockDefinition {
    StockDefinition {
        id: stock_id(name),
        kind: kind.clone(),
    }
}

fn flow_spec(process: &str, source: Option<&str>, target: Option<&str>, kind: &KindId) -> FlowSpec {
    FlowSpec {
        process: process_id(process),
        kind: kind.clone(),
        source: source.map(stock_id),
        target: target.map(stock_id),
    }
}

fn saturating_process(
    maximum_rate: &BigRational,
    resource: &BigRational,
    half_saturation: &BigRational,
    actor: &BigRational,
    elapsed: &BigRational,
) -> BigRational {
    maximum_rate * resource / (half_saturation + resource) * actor * elapsed
}

fn ensure_nonnegative(value: &BigRational) -> Result<(), FoodWebError> {
    if value.is_negative() {
        Err(FoodWebError::NegativeAmount)
    } else {
        Ok(())
    }
}

fn ensure_dense_finite(values: &[f64]) -> Result<(), FoodWebError> {
    if values.iter().all(|value| value.is_finite()) {
        Ok(())
    } else {
        Err(StockFlowError::NonFiniteAmount.into())
    }
}

fn ensure_dense_nonnegative(value: f64) -> Result<(), FoodWebError> {
    if value < 0.0 {
        Err(FoodWebError::NegativeAmount)
    } else {
        Ok(())
    }
}

fn ratio(numerator: i64, denominator: i64) -> BigRational {
    BigRational::new(numerator.into(), denominator.into())
}

fn material_kind() -> KindId {
    KindId::new(MATERIAL).expect("literal kind identifier is nonblank")
}

fn stock_id(value: &str) -> StockId {
    StockId::new(value).expect("literal stock identifier is nonblank")
}

fn process_id(value: &str) -> ProcessId {
    ProcessId::new(value).expect("literal process identifier is nonblank")
}
