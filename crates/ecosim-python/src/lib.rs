use ecosim_core::{BalanceReport, Compartment, World, WorldError, energy_law, integer_amount};
use num_bigint::BigInt;
use num_rational::BigRational;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyModule};

fn invalid(error: WorldError) -> PyErr {
    PyValueError::new_err(error.to_string())
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
    module.add_function(wrap_pyfunction!(energy_law_coefficients, module)?)?;
    Ok(())
}
