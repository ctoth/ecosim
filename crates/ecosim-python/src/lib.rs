use ecosim_core::{
    BalanceReport, Compartment, DenseFoodWeb, DenseFoodWebParameters, DenseFoodWebStep, FoodWeb,
    FoodWebError, FoodWebParameters, FoodWebStep, World, WorldError, energy_law, integer_amount,
};
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::ToPrimitive;
use numpy::ndarray::Array3;
use numpy::{PyArray3, PyReadonlyArray2, PyUntypedArrayMethods};
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyModule};

fn invalid(error: WorldError) -> PyErr {
    PyValueError::new_err(error.to_string())
}

fn invalid_food_web(error: FoodWebError) -> PyErr {
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
    module.add_function(wrap_pyfunction!(energy_law_coefficients, module)?)?;
    module.add_function(wrap_pyfunction!(simulate_food_web, module)?)?;
    Ok(())
}
