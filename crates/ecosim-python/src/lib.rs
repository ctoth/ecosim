use std::collections::BTreeMap;

use ecosim_core::{
    BalanceReport, Compartment, ConsumerSpec, DenseFoodWeb, DenseFoodWebParameters,
    DenseFoodWebStep, DenseTrophicNetwork, DenseTrophicNetworkPlan, DenseTrophicNetworkStep,
    ExactTrophicNetwork, ExactTrophicNetworkPlan, ExactTrophicNetworkStep, FeedingSpec, FoodWeb,
    FoodWebError, FoodWebParameters, FoodWebStep, ProducerSpec, TrophicLaw, TrophicNetworkError,
    TrophicNetworkSpec, World, WorldError, energy_law, integer_amount,
};
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::ToPrimitive;
use numpy::ndarray::Array3;
use numpy::{PyArray3, PyReadonlyArray2, PyReadonlyArray3, PyUntypedArrayMethods};
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyModule};

type PyTrophicLawEvidence = (String, Option<String>, String, bool);

fn invalid(error: WorldError) -> PyErr {
    PyValueError::new_err(error.to_string())
}

fn invalid_food_web(error: FoodWebError) -> PyErr {
    PyValueError::new_err(error.to_string())
}

fn invalid_trophic_network(error: TrophicNetworkError) -> PyErr {
    PyValueError::new_err(error.to_string())
}

fn rational(value: f64) -> PyResult<BigRational> {
    if !value.is_finite() {
        return Err(PyValueError::new_err("amounts must be finite"));
    }
    BigRational::from_float(value)
        .ok_or_else(|| PyValueError::new_err("amount could not be represented exactly"))
}

fn approximate(value: &BigRational) -> PyResult<f64> {
    value
        .to_f64()
        .ok_or_else(|| PyRuntimeError::new_err("exact value is outside the Python float range"))
}

fn integer(value: BigInt) -> BigRational {
    integer_amount(value)
}

fn exact_integer(value: &BigRational) -> PyResult<BigInt> {
    if value.denom() != &BigInt::from(1) {
        return Err(PyRuntimeError::new_err(
            "the integer Python facade encountered a fractional core value",
        ));
    }
    Ok(value.numer().clone())
}

#[pyclass(name = "BalanceReport", frozen)]
struct PyBalanceReport {
    inner: BalanceReport,
}

#[pymethods]
impl PyBalanceReport {
    #[getter]
    fn initial_stock(&self) -> PyResult<BigInt> {
        exact_integer(self.inner.initial_stock())
    }

    #[getter]
    fn inputs(&self) -> PyResult<BigInt> {
        exact_integer(self.inner.inputs())
    }

    #[getter]
    fn outputs(&self) -> PyResult<BigInt> {
        exact_integer(self.inner.outputs())
    }

    #[getter]
    fn final_stock(&self) -> PyResult<BigInt> {
        exact_integer(self.inner.final_stock())
    }

    #[getter]
    fn residual(&self) -> PyResult<BigInt> {
        exact_integer(self.inner.residual())
    }

    #[getter]
    fn balanced(&self) -> bool {
        self.inner.is_balanced()
    }
}

#[pyclass(name = "World")]
struct PyWorld {
    inner: World,
}

#[pymethods]
impl PyWorld {
    #[new]
    fn new(left: BigInt, right: BigInt) -> PyResult<Self> {
        Ok(Self {
            inner: World::new(integer(left), integer(right)).map_err(invalid)?,
        })
    }

    #[getter]
    fn left(&self) -> PyResult<BigInt> {
        exact_integer(self.inner.stock(Compartment::Left))
    }

    #[getter]
    fn right(&self) -> PyResult<BigInt> {
        exact_integer(self.inner.stock(Compartment::Right))
    }

    #[getter]
    fn net_external(&self) -> PyResult<BigInt> {
        exact_integer(&self.inner.net_external())
    }

    #[getter]
    fn history_length(&self) -> usize {
        self.inner.history_len()
    }

    fn transfer(&mut self, source: &str, target: &str, amount: BigInt) -> PyResult<()> {
        self.inner
            .transfer(
                Compartment::parse(source).map_err(invalid)?,
                Compartment::parse(target).map_err(invalid)?,
                integer(amount),
            )
            .map_err(invalid)
    }

    fn input(&mut self, target: &str, amount: BigInt) -> PyResult<()> {
        self.inner
            .input(
                Compartment::parse(target).map_err(invalid)?,
                integer(amount),
            )
            .map_err(invalid)
    }

    fn output(&mut self, source: &str, amount: BigInt) -> PyResult<()> {
        self.inner
            .output(
                Compartment::parse(source).map_err(invalid)?,
                integer(amount),
            )
            .map_err(invalid)
    }

    fn report(&self) -> PyBalanceReport {
        PyBalanceReport {
            inner: self.inner.report(),
        }
    }

    fn conserves_total_energy(&self) -> PyResult<bool> {
        self.inner.satisfies_energy_law().map_err(invalid)
    }
}

#[pyclass(name = "FoodWebStep", frozen)]
struct PyFoodWebStep {
    inner: FoodWebStep,
}

#[pyclass(name = "FoodWebParameters", frozen)]
struct PyFoodWebParameters {
    exact: FoodWebParameters,
    dense: DenseFoodWebParameters,
}

#[pymethods]
impl PyFoodWebParameters {
    #[new]
    #[pyo3(signature = (
        max_growth=0.5,
        nutrient_half_saturation=10.0,
        max_grazing=0.4,
        producer_half_saturation=10.0,
        producer_mortality=0.05,
        consumer_mortality=0.04,
        decomposition=0.1,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        max_growth: f64,
        nutrient_half_saturation: f64,
        max_grazing: f64,
        producer_half_saturation: f64,
        producer_mortality: f64,
        consumer_mortality: f64,
        decomposition: f64,
    ) -> PyResult<Self> {
        let dense = DenseFoodWebParameters::new(
            max_growth,
            nutrient_half_saturation,
            max_grazing,
            producer_half_saturation,
            producer_mortality,
            consumer_mortality,
            decomposition,
        )
        .map_err(invalid_food_web)?;
        let exact = FoodWebParameters::new(
            rational(max_growth)?,
            rational(nutrient_half_saturation)?,
            rational(max_grazing)?,
            rational(producer_half_saturation)?,
            rational(producer_mortality)?,
            rational(consumer_mortality)?,
            rational(decomposition)?,
        )
        .map_err(invalid_food_web)?;
        Ok(Self { exact, dense })
    }

    #[getter]
    fn max_growth(&self) -> f64 {
        self.dense.max_growth()
    }

    #[getter]
    fn nutrient_half_saturation(&self) -> f64 {
        self.dense.nutrient_half_saturation()
    }

    #[getter]
    fn max_grazing(&self) -> f64 {
        self.dense.max_grazing()
    }

    #[getter]
    fn producer_half_saturation(&self) -> f64 {
        self.dense.producer_half_saturation()
    }

    #[getter]
    fn producer_mortality(&self) -> f64 {
        self.dense.producer_mortality()
    }

    #[getter]
    fn consumer_mortality(&self) -> f64 {
        self.dense.consumer_mortality()
    }

    #[getter]
    fn decomposition(&self) -> f64 {
        self.dense.decomposition()
    }
}

#[pymethods]
impl PyFoodWebStep {
    #[getter]
    fn elapsed(&self) -> PyResult<f64> {
        approximate(self.inner.elapsed())
    }

    fn applied(&self, process: &str) -> PyResult<f64> {
        approximate(&self.inner.applied(process))
    }
}

#[pyclass(name = "FoodWeb")]
struct PyFoodWeb {
    inner: FoodWeb,
}

#[pymethods]
impl PyFoodWeb {
    #[new]
    #[pyo3(signature = (nutrient, producer, consumer, detritus, parameters=None))]
    fn new(
        nutrient: f64,
        producer: f64,
        consumer: f64,
        detritus: f64,
        parameters: Option<PyRef<'_, PyFoodWebParameters>>,
    ) -> PyResult<Self> {
        let parameters = parameters
            .map(|parameters| parameters.exact.clone())
            .unwrap_or_default();
        Ok(Self {
            inner: FoodWeb::new(
                rational(nutrient)?,
                rational(producer)?,
                rational(consumer)?,
                rational(detritus)?,
                parameters,
            )
            .map_err(invalid_food_web)?,
        })
    }

    #[getter]
    fn time(&self) -> PyResult<f64> {
        approximate(self.inner.time())
    }

    #[getter]
    fn nutrient(&self) -> PyResult<f64> {
        self.stock("nutrient")
    }

    #[getter]
    fn producer(&self) -> PyResult<f64> {
        self.stock("producer")
    }

    #[getter]
    fn consumer(&self) -> PyResult<f64> {
        self.stock("consumer")
    }

    #[getter]
    fn detritus(&self) -> PyResult<f64> {
        self.stock("detritus")
    }

    #[getter]
    fn inputs(&self) -> PyResult<f64> {
        approximate(&self.inner.inputs())
    }

    #[getter]
    fn outputs(&self) -> PyResult<f64> {
        approximate(&self.inner.outputs())
    }

    #[getter]
    fn balance_residual(&self) -> PyResult<f64> {
        approximate(&self.inner.balance_residual())
    }

    #[getter]
    fn balanced(&self) -> bool {
        self.inner.is_balanced()
    }

    #[pyo3(signature = (elapsed, nutrient_input=0.0, harvest=0.0))]
    fn step(&mut self, elapsed: f64, nutrient_input: f64, harvest: f64) -> PyResult<PyFoodWebStep> {
        Ok(PyFoodWebStep {
            inner: self
                .inner
                .step(
                    rational(elapsed)?,
                    rational(nutrient_input)?,
                    rational(harvest)?,
                )
                .map_err(invalid_food_web)?,
        })
    }
}

#[pyclass(name = "DenseFoodWebStep", frozen)]
struct PyDenseFoodWebStep {
    inner: DenseFoodWebStep,
}

#[pymethods]
impl PyDenseFoodWebStep {
    #[getter]
    fn elapsed(&self) -> f64 {
        self.inner.elapsed()
    }

    fn applied(&self, process: &str) -> f64 {
        self.inner.applied(process)
    }
}

#[pyclass(name = "DenseFoodWeb")]
struct PyDenseFoodWeb {
    inner: DenseFoodWeb,
}

#[pymethods]
impl PyDenseFoodWeb {
    #[new]
    #[pyo3(signature = (nutrient, producer, consumer, detritus, parameters=None))]
    fn new(
        nutrient: f64,
        producer: f64,
        consumer: f64,
        detritus: f64,
        parameters: Option<PyRef<'_, PyFoodWebParameters>>,
    ) -> PyResult<Self> {
        let parameters = parameters
            .map(|parameters| parameters.dense)
            .unwrap_or_default();
        Ok(Self {
            inner: DenseFoodWeb::new(nutrient, producer, consumer, detritus, parameters)
                .map_err(invalid_food_web)?,
        })
    }

    #[getter]
    fn time(&self) -> f64 {
        self.inner.time()
    }

    #[getter]
    fn nutrient(&self) -> f64 {
        self.stock("nutrient")
    }

    #[getter]
    fn producer(&self) -> f64 {
        self.stock("producer")
    }

    #[getter]
    fn consumer(&self) -> f64 {
        self.stock("consumer")
    }

    #[getter]
    fn detritus(&self) -> f64 {
        self.stock("detritus")
    }

    #[getter]
    fn inputs(&self) -> f64 {
        self.inner.inputs()
    }

    #[getter]
    fn outputs(&self) -> f64 {
        self.inner.outputs()
    }

    #[getter]
    fn balance_residual(&self) -> f64 {
        self.inner.balance_residual()
    }

    #[getter]
    fn balanced(&self) -> bool {
        self.inner.is_balanced()
    }

    #[pyo3(signature = (elapsed, nutrient_input=0.0, harvest=0.0))]
    fn step(
        &mut self,
        elapsed: f64,
        nutrient_input: f64,
        harvest: f64,
    ) -> PyResult<PyDenseFoodWebStep> {
        Ok(PyDenseFoodWebStep {
            inner: self
                .inner
                .step(elapsed, nutrient_input, harvest)
                .map_err(invalid_food_web)?,
        })
    }
}

type ProducerTuple = (String, f64, f64, f64);
type ConsumerTuple = (String, f64);
type FeedingTuple = (String, String, f64, f64, f64);

fn dense_trophic_spec(
    producers: Vec<ProducerTuple>,
    consumers: Vec<ConsumerTuple>,
    feedings: Vec<FeedingTuple>,
    decomposition: f64,
) -> TrophicNetworkSpec<f64> {
    TrophicNetworkSpec::new(
        producers
            .into_iter()
            .map(|(name, max_growth, half_saturation, mortality)| {
                ProducerSpec::new(name, max_growth, half_saturation, mortality)
            })
            .collect(),
        consumers
            .into_iter()
            .map(|(name, mortality)| ConsumerSpec::new(name, mortality))
            .collect(),
        feedings
            .into_iter()
            .map(
                |(consumer, resource, max_rate, half_saturation, efficiency)| {
                    FeedingSpec::new(consumer, resource, max_rate, half_saturation, efficiency)
                },
            )
            .collect(),
        decomposition,
    )
}

fn exact_trophic_spec(
    producers: Vec<ProducerTuple>,
    consumers: Vec<ConsumerTuple>,
    feedings: Vec<FeedingTuple>,
    decomposition: f64,
) -> PyResult<TrophicNetworkSpec<BigRational>> {
    Ok(TrophicNetworkSpec::new(
        producers
            .into_iter()
            .map(|(name, max_growth, half_saturation, mortality)| {
                Ok(ProducerSpec::new(
                    name,
                    rational(max_growth)?,
                    rational(half_saturation)?,
                    rational(mortality)?,
                ))
            })
            .collect::<PyResult<_>>()?,
        consumers
            .into_iter()
            .map(|(name, mortality)| Ok(ConsumerSpec::new(name, rational(mortality)?)))
            .collect::<PyResult<_>>()?,
        feedings
            .into_iter()
            .map(
                |(consumer, resource, max_rate, half_saturation, efficiency)| {
                    Ok(FeedingSpec::new(
                        consumer,
                        resource,
                        rational(max_rate)?,
                        rational(half_saturation)?,
                        rational(efficiency)?,
                    ))
                },
            )
            .collect::<PyResult<_>>()?,
        rational(decomposition)?,
    ))
}

fn exact_stock_map(values: BTreeMap<String, f64>) -> PyResult<BTreeMap<String, BigRational>> {
    values
        .into_iter()
        .map(|(name, value)| Ok((name, rational(value)?)))
        .collect()
}

#[pyclass(name = "TrophicNetworkStep", frozen)]
struct PyTrophicNetworkStep {
    inner: ExactTrophicNetworkStep,
}

#[pymethods]
impl PyTrophicNetworkStep {
    #[getter]
    fn elapsed(&self) -> PyResult<f64> {
        approximate(self.inner.elapsed())
    }

    fn growth(&self, producer: &str) -> PyResult<Option<f64>> {
        self.inner.growth(producer).map(approximate).transpose()
    }

    fn feeding(&self, consumer: &str, resource: &str) -> PyResult<Option<f64>> {
        self.inner
            .feeding(consumer, resource)
            .as_ref()
            .map(approximate)
            .transpose()
    }

    fn mortality(&self, stock: &str) -> PyResult<Option<f64>> {
        self.inner.mortality(stock).map(approximate).transpose()
    }

    fn harvest(&self, consumer: &str) -> PyResult<Option<f64>> {
        self.inner.harvest(consumer).map(approximate).transpose()
    }

    fn decomposition(&self) -> PyResult<f64> {
        approximate(self.inner.decomposition())
    }
}

#[pyclass(name = "NativeTrophicNetwork")]
struct PyTrophicNetwork {
    inner: ExactTrophicNetwork,
}

#[pyclass(name = "NativeTrophicNetworkPlan", frozen)]
struct PyTrophicNetworkPlan {
    inner: ExactTrophicNetworkPlan,
}

#[pymethods]
impl PyTrophicNetworkPlan {
    #[new]
    fn new(
        producers: Vec<ProducerTuple>,
        consumers: Vec<ConsumerTuple>,
        feedings: Vec<FeedingTuple>,
        decomposition: f64,
    ) -> PyResult<Self> {
        Ok(Self {
            inner: ExactTrophicNetworkPlan::compile(exact_trophic_spec(
                producers,
                consumers,
                feedings,
                decomposition,
            )?)
            .map_err(invalid_trophic_network)?,
        })
    }

    #[getter]
    fn stock_names(&self) -> Vec<String> {
        self.inner.stock_names().to_vec()
    }

    #[getter]
    fn consumer_names(&self) -> Vec<String> {
        self.inner.consumer_names().to_vec()
    }

    #[getter]
    fn evidence_axis_names(&self) -> Vec<String> {
        self.inner
            .evidence_axis_names()
            .map(str::to_owned)
            .collect()
    }

    #[getter]
    fn evidence_laws(&self) -> Vec<(String, Option<String>, String)> {
        self.inner
            .evidence_laws()
            .map(|law| {
                (
                    trophic_law_name(law).to_owned(),
                    law.axis_name().map(str::to_owned),
                    trophic_law_grade(law).to_owned(),
                )
            })
            .collect()
    }

    fn start(&self, initial: BTreeMap<String, f64>) -> PyResult<PyTrophicNetwork> {
        Ok(PyTrophicNetwork {
            inner: self
                .inner
                .start(exact_stock_map(initial)?)
                .map_err(invalid_trophic_network)?,
        })
    }
}

#[pymethods]
impl PyTrophicNetwork {
    #[new]
    fn new(
        producers: Vec<ProducerTuple>,
        consumers: Vec<ConsumerTuple>,
        feedings: Vec<FeedingTuple>,
        decomposition: f64,
        initial: BTreeMap<String, f64>,
    ) -> PyResult<Self> {
        Ok(Self {
            inner: ExactTrophicNetwork::new(
                exact_trophic_spec(producers, consumers, feedings, decomposition)?,
                exact_stock_map(initial)?,
            )
            .map_err(invalid_trophic_network)?,
        })
    }

    #[getter]
    fn time(&self) -> PyResult<f64> {
        approximate(self.inner.time())
    }

    #[getter]
    fn stock_names(&self) -> Vec<String> {
        self.inner.stock_names().to_vec()
    }

    fn stock(&self, name: &str) -> PyResult<f64> {
        self.inner
            .stock(name)
            .ok_or_else(|| PyValueError::new_err(format!("unknown trophic stock: {name}")))
            .and_then(approximate)
    }

    #[getter]
    fn inputs(&self) -> PyResult<f64> {
        approximate(&self.inner.inputs())
    }

    #[getter]
    fn outputs(&self) -> PyResult<f64> {
        approximate(&self.inner.outputs())
    }

    #[getter]
    fn balance_residual(&self) -> PyResult<f64> {
        approximate(&self.inner.balance_residual())
    }

    #[getter]
    fn balanced(&self) -> bool {
        self.inner.is_balanced()
    }

    #[getter]
    fn trace_length(&self) -> usize {
        self.inner.trace().len()
    }

    fn evidence(&self) -> PyResult<Vec<PyTrophicLawEvidence>> {
        Ok(self
            .inner
            .evidence()
            .map_err(invalid_trophic_network)?
            .laws()
            .iter()
            .map(|evidence| {
                (
                    trophic_law_name(evidence.law()).to_owned(),
                    evidence.law().axis_name().map(str::to_owned),
                    evidence.grade().to_string(),
                    evidence.is_satisfied(),
                )
            })
            .collect())
    }

    #[pyo3(signature = (elapsed, nutrient_input=0.0, harvests=None))]
    fn step(
        &mut self,
        elapsed: f64,
        nutrient_input: f64,
        harvests: Option<BTreeMap<String, f64>>,
    ) -> PyResult<PyTrophicNetworkStep> {
        Ok(PyTrophicNetworkStep {
            inner: self
                .inner
                .step(
                    rational(elapsed)?,
                    rational(nutrient_input)?,
                    exact_stock_map(harvests.unwrap_or_default())?,
                )
                .map_err(invalid_trophic_network)?,
        })
    }
}

fn trophic_law_name(law: &TrophicLaw) -> &'static str {
    match law {
        TrophicLaw::MaterialInvariant => "material_invariant",
        TrophicLaw::StockNonnegative(_) => "stock_nonnegative",
        TrophicLaw::CumulativeInputNondecreasing => "cumulative_input_nondecreasing",
        TrophicLaw::CumulativeOutputNondecreasing => "cumulative_output_nondecreasing",
    }
}

fn trophic_law_grade(law: &TrophicLaw) -> &'static str {
    match law {
        TrophicLaw::MaterialInvariant => "invariant",
        TrophicLaw::StockNonnegative(_) => "nonnegative",
        TrophicLaw::CumulativeInputNondecreasing | TrophicLaw::CumulativeOutputNondecreasing => {
            "nondecreasing"
        }
    }
}

#[pyclass(name = "DenseTrophicNetworkStep", frozen)]
struct PyDenseTrophicNetworkStep {
    inner: DenseTrophicNetworkStep,
}

#[pymethods]
impl PyDenseTrophicNetworkStep {
    #[getter]
    fn elapsed(&self) -> f64 {
        self.inner.elapsed()
    }

    fn growth(&self, producer: &str) -> Option<f64> {
        self.inner.growth(producer)
    }

    fn feeding(&self, consumer: &str, resource: &str) -> Option<f64> {
        self.inner.feeding(consumer, resource)
    }

    fn mortality(&self, stock: &str) -> Option<f64> {
        self.inner.mortality(stock)
    }

    fn harvest(&self, consumer: &str) -> Option<f64> {
        self.inner.harvest(consumer)
    }

    fn decomposition(&self) -> f64 {
        self.inner.decomposition()
    }
}

#[pyclass(name = "NativeDenseTrophicNetwork")]
struct PyDenseTrophicNetwork {
    inner: DenseTrophicNetwork,
}

#[pyclass(name = "NativeDenseTrophicNetworkPlan", frozen)]
struct PyDenseTrophicNetworkPlan {
    inner: DenseTrophicNetworkPlan,
}

#[pymethods]
impl PyDenseTrophicNetworkPlan {
    #[new]
    fn new(
        producers: Vec<ProducerTuple>,
        consumers: Vec<ConsumerTuple>,
        feedings: Vec<FeedingTuple>,
        decomposition: f64,
    ) -> PyResult<Self> {
        Ok(Self {
            inner: DenseTrophicNetworkPlan::compile(dense_trophic_spec(
                producers,
                consumers,
                feedings,
                decomposition,
            ))
            .map_err(invalid_trophic_network)?,
        })
    }

    #[getter]
    fn stock_names(&self) -> Vec<String> {
        self.inner.stock_names().to_vec()
    }

    #[getter]
    fn consumer_names(&self) -> Vec<String> {
        self.inner.consumer_names().to_vec()
    }

    fn start(&self, initial: BTreeMap<String, f64>) -> PyResult<PyDenseTrophicNetwork> {
        Ok(PyDenseTrophicNetwork {
            inner: self.inner.start(initial).map_err(invalid_trophic_network)?,
        })
    }

    #[pyo3(signature = (
        initial_states,
        steps,
        elapsed,
        nutrient_inputs=None,
        harvests=None,
    ))]
    fn simulate<'py>(
        &self,
        py: Python<'py>,
        initial_states: PyReadonlyArray2<'py, f64>,
        steps: usize,
        elapsed: f64,
        nutrient_inputs: Option<PyReadonlyArray2<'py, f64>>,
        harvests: Option<PyReadonlyArray3<'py, f64>>,
    ) -> PyResult<Bound<'py, PyArray3<f64>>> {
        if !elapsed.is_finite() || elapsed < 0.0 {
            return Err(PyValueError::new_err(
                "elapsed must be finite and nonnegative",
            ));
        }
        let stock_count = self.inner.stock_names().len();
        let consumer_count = self.inner.consumer_names().len();
        let initial_shape = initial_states.shape();
        let batch = initial_shape[0];
        if initial_shape[1] != stock_count {
            return Err(PyValueError::new_err(format!(
                "initial_states must have shape (batch, {stock_count}), got ({batch}, {})",
                initial_shape[1]
            )));
        }
        validate_forcing_shape("nutrient_inputs", nutrient_inputs.as_ref(), batch, steps)?;
        if let Some(values) = harvests.as_ref() {
            let shape = values.shape();
            if shape != [batch, steps, consumer_count] {
                return Err(PyValueError::new_err(format!(
                    "harvests must have shape ({batch}, {steps}, {consumer_count}), got ({}, {}, {})",
                    shape[0], shape[1], shape[2]
                )));
            }
            validate_dense_array3("harvests", values)?;
        }
        validate_dense_array("initial_states", &initial_states)?;
        let time_points = steps
            .checked_add(1)
            .ok_or_else(|| PyValueError::new_err("steps is too large"))?;
        let elements_per_batch = time_points
            .checked_mul(stock_count)
            .ok_or_else(|| PyValueError::new_err("trajectory shape is too large"))?;
        let output_elements = batch
            .checked_mul(elements_per_batch)
            .ok_or_else(|| PyValueError::new_err("trajectory shape is too large"))?;
        let output_bytes = output_elements
            .checked_mul(std::mem::size_of::<f64>())
            .ok_or_else(|| PyValueError::new_err("trajectory allocation is too large"))?;
        if output_bytes > isize::MAX as usize {
            return Err(PyValueError::new_err(
                "trajectory allocation exceeds the platform limit",
            ));
        }

        let initial = initial_states
            .as_array()
            .iter()
            .copied()
            .collect::<Vec<_>>();
        let nutrient_inputs =
            nutrient_inputs.map(|values| values.as_array().iter().copied().collect::<Vec<_>>());
        let harvests = harvests.map(|values| values.as_array().iter().copied().collect::<Vec<_>>());
        let plan = self.inner.clone();
        let output = py
            .detach(move || {
                compute_trophic_network_trajectory(
                    &plan,
                    &initial,
                    batch,
                    steps,
                    time_points,
                    stock_count,
                    consumer_count,
                    elapsed,
                    nutrient_inputs.as_deref(),
                    harvests.as_deref(),
                    output_elements,
                )
            })
            .map_err(PyValueError::new_err)?;
        let trajectory = Array3::from_shape_vec((batch, time_points, stock_count), output)
            .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
        Ok(PyArray3::from_owned_array(py, trajectory))
    }
}

#[pymethods]
impl PyDenseTrophicNetwork {
    #[new]
    fn new(
        producers: Vec<ProducerTuple>,
        consumers: Vec<ConsumerTuple>,
        feedings: Vec<FeedingTuple>,
        decomposition: f64,
        initial: BTreeMap<String, f64>,
    ) -> PyResult<Self> {
        Ok(Self {
            inner: DenseTrophicNetwork::new(
                dense_trophic_spec(producers, consumers, feedings, decomposition),
                initial,
            )
            .map_err(invalid_trophic_network)?,
        })
    }

    #[getter]
    fn time(&self) -> f64 {
        self.inner.time()
    }

    #[getter]
    fn stock_names(&self) -> Vec<String> {
        self.inner.stock_names().to_vec()
    }

    fn stock(&self, name: &str) -> PyResult<f64> {
        self.inner
            .stock(name)
            .ok_or_else(|| PyValueError::new_err(format!("unknown trophic stock: {name}")))
    }

    #[getter]
    fn inputs(&self) -> f64 {
        self.inner.inputs()
    }

    #[getter]
    fn outputs(&self) -> f64 {
        self.inner.outputs()
    }

    #[getter]
    fn balance_residual(&self) -> f64 {
        self.inner.balance_residual()
    }

    #[getter]
    fn balanced(&self) -> bool {
        self.inner.is_balanced()
    }

    #[pyo3(signature = (elapsed, nutrient_input=0.0, harvests=None))]
    fn step(
        &mut self,
        elapsed: f64,
        nutrient_input: f64,
        harvests: Option<BTreeMap<String, f64>>,
    ) -> PyResult<PyDenseTrophicNetworkStep> {
        Ok(PyDenseTrophicNetworkStep {
            inner: self
                .inner
                .step(elapsed, nutrient_input, harvests.unwrap_or_default())
                .map_err(invalid_trophic_network)?,
        })
    }
}

impl PyDenseFoodWeb {
    fn stock(&self, name: &str) -> f64 {
        self.inner
            .stock(name)
            .expect("Python facade uses fixed food-web stock names")
    }
}

#[pyfunction]
#[pyo3(signature = (
    initial_states,
    steps,
    elapsed,
    nutrient_inputs=None,
    harvests=None,
    parameters=None,
))]
fn simulate_food_web<'py>(
    py: Python<'py>,
    initial_states: PyReadonlyArray2<'py, f64>,
    steps: usize,
    elapsed: f64,
    nutrient_inputs: Option<PyReadonlyArray2<'py, f64>>,
    harvests: Option<PyReadonlyArray2<'py, f64>>,
    parameters: Option<PyRef<'py, PyFoodWebParameters>>,
) -> PyResult<Bound<'py, PyArray3<f64>>> {
    if !elapsed.is_finite() || elapsed < 0.0 {
        return Err(PyValueError::new_err(
            "elapsed must be finite and nonnegative",
        ));
    }
    let initial_shape = initial_states.shape();
    if initial_shape[1] != 4 {
        return Err(PyValueError::new_err(format!(
            "initial_states must have shape (batch, 4), got ({}, {})",
            initial_shape[0], initial_shape[1]
        )));
    }
    let batch = initial_shape[0];
    let time_points = steps
        .checked_add(1)
        .ok_or_else(|| PyValueError::new_err("steps is too large"))?;
    let elements_per_batch = time_points
        .checked_mul(4)
        .ok_or_else(|| PyValueError::new_err("trajectory shape is too large"))?;
    let output_elements = batch
        .checked_mul(elements_per_batch)
        .ok_or_else(|| PyValueError::new_err("trajectory shape is too large"))?;
    let bytes_per_batch = elements_per_batch
        .checked_mul(std::mem::size_of::<f64>())
        .ok_or_else(|| PyValueError::new_err("trajectory allocation is too large"))?;
    let output_bytes = output_elements
        .checked_mul(std::mem::size_of::<f64>())
        .ok_or_else(|| PyValueError::new_err("trajectory allocation is too large"))?;
    if bytes_per_batch > isize::MAX as usize || output_bytes > isize::MAX as usize {
        return Err(PyValueError::new_err(
            "trajectory allocation exceeds the platform limit",
        ));
    }
    validate_dense_array("initial_states", &initial_states)?;
    validate_forcing_shape("nutrient_inputs", nutrient_inputs.as_ref(), batch, steps)?;
    validate_forcing_shape("harvests", harvests.as_ref(), batch, steps)?;

    // Copy in logical index order before releasing the GIL. This supports
    // non-contiguous views without retaining Python-owned memory.
    let initial = initial_states
        .as_array()
        .iter()
        .copied()
        .collect::<Vec<_>>();
    let nutrient_inputs =
        nutrient_inputs.map(|values| values.as_array().iter().copied().collect::<Vec<_>>());
    let harvests = harvests.map(|values| values.as_array().iter().copied().collect::<Vec<_>>());
    let parameters = parameters
        .map(|parameters| parameters.dense)
        .unwrap_or_default();
    let output = py
        .detach(move || {
            compute_food_web_trajectory(
                &initial,
                batch,
                steps,
                time_points,
                elapsed,
                nutrient_inputs.as_deref(),
                harvests.as_deref(),
                output_elements,
                parameters,
            )
        })
        .map_err(PyValueError::new_err)?;
    let trajectory = Array3::from_shape_vec((batch, time_points, 4), output)
        .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
    Ok(PyArray3::from_owned_array(py, trajectory))
}

#[allow(clippy::too_many_arguments)]
fn compute_food_web_trajectory(
    initial: &[f64],
    batch: usize,
    steps: usize,
    time_points: usize,
    elapsed: f64,
    nutrient_inputs: Option<&[f64]>,
    harvests: Option<&[f64]>,
    output_elements: usize,
    parameters: DenseFoodWebParameters,
) -> Result<Vec<f64>, String> {
    let mut trajectory = Vec::new();
    trajectory
        .try_reserve_exact(output_elements)
        .map_err(|error| format!("trajectory allocation failed: {error}"))?;
    trajectory.resize(output_elements, 0.0);

    for batch_index in 0..batch {
        let initial_offset = batch_index * 4;
        let mut web = DenseFoodWeb::new(
            initial[initial_offset],
            initial[initial_offset + 1],
            initial[initial_offset + 2],
            initial[initial_offset + 3],
            parameters,
        )
        .map_err(|error| error.to_string())?;
        let trajectory_offset = batch_index * time_points * 4;
        trajectory[trajectory_offset..trajectory_offset + 4].copy_from_slice(web.amounts());

        for step_index in 0..steps {
            let forcing_index = batch_index * steps + step_index;
            let nutrient_input = nutrient_inputs.map_or(0.0, |values| values[forcing_index]);
            let harvest = harvests.map_or(0.0, |values| values[forcing_index]);
            web.step_discard(elapsed, nutrient_input, harvest)
                .map_err(|error| error.to_string())?;
            let state_offset = trajectory_offset + (step_index + 1) * 4;
            trajectory[state_offset..state_offset + 4].copy_from_slice(web.amounts());
        }
    }

    Ok(trajectory)
}

#[allow(clippy::too_many_arguments)]
fn compute_trophic_network_trajectory(
    plan: &DenseTrophicNetworkPlan,
    initial: &[f64],
    batch: usize,
    steps: usize,
    time_points: usize,
    stock_count: usize,
    consumer_count: usize,
    elapsed: f64,
    nutrient_inputs: Option<&[f64]>,
    harvests: Option<&[f64]>,
    output_elements: usize,
) -> Result<Vec<f64>, String> {
    let mut trajectory = Vec::new();
    trajectory
        .try_reserve_exact(output_elements)
        .map_err(|error| format!("trajectory allocation failed: {error}"))?;
    trajectory.resize(output_elements, 0.0);
    let zero_harvests = vec![0.0; consumer_count];

    for batch_index in 0..batch {
        let initial_offset = batch_index * stock_count;
        let mut network = plan
            .start_ordered(initial[initial_offset..initial_offset + stock_count].to_vec())
            .map_err(|error| error.to_string())?;
        let trajectory_offset = batch_index * time_points * stock_count;
        trajectory[trajectory_offset..trajectory_offset + stock_count]
            .copy_from_slice(network.amounts());

        for step_index in 0..steps {
            let forcing_index = batch_index * steps + step_index;
            let nutrient_input = nutrient_inputs.map_or(0.0, |values| values[forcing_index]);
            let harvest_offset = forcing_index * consumer_count;
            let harvest = harvests.map_or(zero_harvests.as_slice(), |values| {
                &values[harvest_offset..harvest_offset + consumer_count]
            });
            network
                .step_discard_ordered(elapsed, nutrient_input, harvest)
                .map_err(|error| error.to_string())?;
            let state_offset = trajectory_offset + (step_index + 1) * stock_count;
            trajectory[state_offset..state_offset + stock_count].copy_from_slice(network.amounts());
        }
    }

    Ok(trajectory)
}

fn validate_forcing_shape(
    name: &str,
    values: Option<&PyReadonlyArray2<'_, f64>>,
    batch: usize,
    steps: usize,
) -> PyResult<()> {
    if let Some(values) = values {
        let shape = values.shape();
        if shape != [batch, steps] {
            return Err(PyValueError::new_err(format!(
                "{name} must have shape ({batch}, {steps}), got ({}, {})",
                shape[0], shape[1]
            )));
        }
        validate_dense_array(name, values)?;
    }
    Ok(())
}

fn validate_dense_array(name: &str, values: &PyReadonlyArray2<'_, f64>) -> PyResult<()> {
    if values
        .as_array()
        .iter()
        .all(|value| value.is_finite() && *value >= 0.0)
    {
        Ok(())
    } else {
        Err(PyValueError::new_err(format!(
            "{name} values must be finite and nonnegative"
        )))
    }
}

fn validate_dense_array3(name: &str, values: &PyReadonlyArray3<'_, f64>) -> PyResult<()> {
    if values
        .as_array()
        .iter()
        .all(|value| value.is_finite() && *value >= 0.0)
    {
        Ok(())
    } else {
        Err(PyValueError::new_err(format!(
            "{name} values must be finite and nonnegative"
        )))
    }
}

impl PyFoodWeb {
    fn stock(&self, name: &str) -> PyResult<f64> {
        approximate(
            self.inner
                .stock(name)
                .expect("Python facade uses fixed food-web stock names"),
        )
    }
}

#[pyfunction]
fn energy_law_coefficients(py: Python<'_>) -> PyResult<Py<PyDict>> {
    let result = PyDict::new(py);
    let law = energy_law().map_err(invalid)?;
    for (axis, coefficient) in law.coefficients() {
        result.set_item(axis.as_str(), exact_integer(coefficient)?)?;
    }
    Ok(result.unbind())
}

#[pymodule]
fn _core(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyBalanceReport>()?;
    module.add_class::<PyWorld>()?;
    module.add_class::<PyFoodWebStep>()?;
    module.add_class::<PyFoodWebParameters>()?;
    module.add_class::<PyFoodWeb>()?;
    module.add_class::<PyDenseFoodWebStep>()?;
    module.add_class::<PyDenseFoodWeb>()?;
    module.add_class::<PyTrophicNetworkStep>()?;
    module.add_class::<PyTrophicNetworkPlan>()?;
    module.add_class::<PyTrophicNetwork>()?;
    module.add_class::<PyDenseTrophicNetworkStep>()?;
    module.add_class::<PyDenseTrophicNetworkPlan>()?;
    module.add_class::<PyDenseTrophicNetwork>()?;
    module.add_function(wrap_pyfunction!(energy_law_coefficients, module)?)?;
    module.add_function(wrap_pyfunction!(simulate_food_web, module)?)?;
    Ok(())
}
