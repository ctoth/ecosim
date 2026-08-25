//! Mata Almonacid and Medel's structure-preserving NPZD benchmark.
//!
//! This module implements Eqs. 2, 7, 8, 9, and 10 of the open 2020
//! manuscript (arXiv:2007.11815). It deliberately remains independent of the
//! generic stock-flow settlement engine: the paper's two-stage modified
//! Patankar--Runge--Kutta method is itself part of the benchmark oracle.

use std::error::Error;
use std::fmt;

const STOCKS: usize = 4;
const SQRT_2: f64 = std::f64::consts::SQRT_2;
const SQRT_PI_OVER_2: f64 = 1.253_314_137_315_500_1;

/// Nutrient, phytoplankton, zooplankton, and detritus concentrations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct State {
    /// Nutrient concentration in mmol N m^-3.
    nutrient: f64,
    /// Phytoplankton concentration in mmol N m^-3.
    phytoplankton: f64,
    /// Zooplankton concentration in mmol N m^-3.
    zooplankton: f64,
    /// Detritus concentration in mmol N m^-3.
    detritus: f64,
}

impl State {
    /// Constructs a strictly positive paper state.
    pub fn new(
        nutrient: f64,
        phytoplankton: f64,
        zooplankton: f64,
        detritus: f64,
    ) -> Result<Self, ModelError> {
        let values = [nutrient, phytoplankton, zooplankton, detritus];
        ensure_finite(&values)?;
        if values.iter().any(|value| *value <= 0.0) {
            return Err(ModelError::NonpositiveState);
        }
        Ok(Self {
            nutrient,
            phytoplankton,
            zooplankton,
            detritus,
        })
    }

    /// Nutrient concentration in mmol N m^-3.
    pub fn nutrient(self) -> f64 {
        self.nutrient
    }

    /// Phytoplankton concentration in mmol N m^-3.
    pub fn phytoplankton(self) -> f64 {
        self.phytoplankton
    }

    /// Zooplankton concentration in mmol N m^-3.
    pub fn zooplankton(self) -> f64 {
        self.zooplankton
    }

    /// Detritus concentration in mmol N m^-3.
    pub fn detritus(self) -> f64 {
        self.detritus
    }

    /// Returns state values in the paper's N, P, Z, D order.
    pub fn values(self) -> [f64; STOCKS] {
        [
            self.nutrient,
            self.phytoplankton,
            self.zooplankton,
            self.detritus,
        ]
    }

    /// Total nitrogen represented by the four compartments.
    pub fn total(self) -> f64 {
        compensated_sum(self.values())
    }

    fn from_values(values: [f64; STOCKS]) -> Result<Self, ModelError> {
        Self::new(values[0], values[1], values[2], values[3])
    }
}

/// Parameters of the autonomous production--destruction system.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Parameters {
    pub nutrient_half_saturation: f64,
    pub light_half_saturation: f64,
    pub maximum_growth: f64,
    pub linear_zooplankton_loss: f64,
    pub quadratic_zooplankton_loss: f64,
    pub phytoplankton_loss: f64,
    pub remineralization: f64,
    pub assimilation_efficiency: f64,
    pub grazing_encounter: f64,
    pub maximum_grazing: f64,
}

impl Parameters {
    /// Validates the positive parameters and the assimilation fraction.
    pub fn validate(self) -> Result<Self, ModelError> {
        let positive = [
            self.nutrient_half_saturation,
            self.light_half_saturation,
            self.maximum_growth,
            self.linear_zooplankton_loss,
            self.quadratic_zooplankton_loss,
            self.phytoplankton_loss,
            self.remineralization,
            self.grazing_encounter,
            self.maximum_grazing,
        ];
        ensure_finite(&positive)?;
        if positive.iter().any(|value| *value < 0.0)
            || self.nutrient_half_saturation == 0.0
            || self.light_half_saturation == 0.0
            || self.maximum_grazing == 0.0
            || self.grazing_encounter == 0.0
        {
            return Err(ModelError::InvalidParameter);
        }
        if !self.assimilation_efficiency.is_finite()
            || !(0.0..=1.0).contains(&self.assimilation_efficiency)
        {
            return Err(ModelError::InvalidParameter);
        }
        Ok(self)
    }

    /// The paper's calibrated TLQZ parameter row (Tables 3--4).
    pub fn tlqz() -> Self {
        Self {
            nutrient_half_saturation: 0.863_36,
            light_half_saturation: 0.051_12,
            maximum_growth: 0.948_48,
            linear_zooplankton_loss: 0.108_30,
            quadratic_zooplankton_loss: 0.058_20,
            phytoplankton_loss: 0.080_91,
            remineralization: 0.000_05,
            assimilation_efficiency: 0.997_02,
            grazing_encounter: 0.027_91,
            maximum_grazing: 26.812_9,
        }
    }

    /// TLQZ with the two substitutions stated for the paper's Figure 10 run.
    pub fn three_pulse_tlqz() -> Self {
        Self {
            remineralization: 0.000_1,
            assimilation_efficiency: 0.75,
            ..Self::tlqz()
        }
    }
}

/// One Gaussian nutrient pulse from Eq. 3b.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GaussianPulse {
    amplitude: f64,
    center: f64,
    width: f64,
}

impl GaussianPulse {
    /// Constructs a finite pulse with positive width and nonnegative amplitude.
    pub fn new(amplitude: f64, center: f64, width: f64) -> Result<Self, ModelError> {
        ensure_finite(&[amplitude, center, width])?;
        if amplitude < 0.0 || width <= 0.0 {
            return Err(ModelError::InvalidParameter);
        }
        Ok(Self {
            amplitude,
            center,
            width,
        })
    }

    /// Pulse amplitude.
    pub fn amplitude(self) -> f64 {
        self.amplitude
    }

    /// Pulse center in days.
    pub fn center(self) -> f64 {
        self.center
    }

    /// Gaussian width in days.
    pub fn width(self) -> f64 {
        self.width
    }

    /// Exact analytic nutrient increment over `[start, end]` (Eq. 8b).
    pub fn increment(self, start: f64, end: f64) -> Result<f64, ModelError> {
        ensure_finite(&[start, end])?;
        if end < start {
            return Err(ModelError::InvalidInterval);
        }
        let upper = libm::erf((end - self.center) / (SQRT_2 * self.width));
        let lower = libm::erf((start - self.center) / (SQRT_2 * self.width));
        let increment = self.amplitude * self.width * SQRT_PI_OVER_2 * (upper - lower);
        if increment.is_finite() && increment >= 0.0 {
            Ok(increment)
        } else {
            Err(ModelError::ArithmeticFailure)
        }
    }

    /// Integral over the full real line.
    pub fn total_increment(self) -> f64 {
        self.amplitude * self.width * (2.0 * std::f64::consts::PI).sqrt()
    }
}

/// Exact sinking subflow configuration from Eq. 9b.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sinking {
    rate: f64,
    detritus_floor: f64,
}

impl Sinking {
    /// Validates a nonnegative rate and strictly positive detritus floor.
    pub fn new(rate: f64, detritus_floor: f64) -> Result<Self, ModelError> {
        Self {
            rate,
            detritus_floor,
        }
        .validate()
    }

    /// Detritus sinking rate in inverse days.
    pub fn rate(self) -> f64 {
        self.rate
    }

    /// Strictly positive detritus floor in mmol N m^-3.
    pub fn detritus_floor(self) -> f64 {
        self.detritus_floor
    }

    /// Returns `(remaining_detritus, exported_detritus)` after one exact step.
    pub fn apply(self, detritus: f64, elapsed: f64) -> Result<(f64, f64), ModelError> {
        ensure_finite(&[detritus, elapsed])?;
        if detritus <= 0.0 || elapsed < 0.0 {
            return Err(ModelError::InvalidParameter);
        }
        if detritus < self.detritus_floor || self.rate == 0.0 || elapsed == 0.0 {
            return Ok((detritus, 0.0));
        }
        let remaining = self.detritus_floor
            + libm::exp(-self.rate * elapsed) * (detritus - self.detritus_floor);
        let exported = detritus - remaining;
        if remaining.is_finite() && exported.is_finite() && exported >= 0.0 {
            Ok((remaining, exported))
        } else {
            Err(ModelError::ArithmeticFailure)
        }
    }

    fn validate(self) -> Result<Self, ModelError> {
        ensure_finite(&[self.rate, self.detritus_floor])?;
        if self.rate < 0.0 || self.detritus_floor <= 0.0 {
            return Err(ModelError::InvalidParameter);
        }
        Ok(self)
    }
}

/// Evidence from one paper-composed step.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Step {
    pub before: State,
    pub after_autonomous: State,
    pub after: State,
    pub nutrient_input: f64,
    pub detritus_export: f64,
    pub ledger_residual: f64,
}

/// Source-specific numerical model using the paper's first-order composition.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Model {
    parameters: Parameters,
    sinking: Sinking,
}

impl Model {
    pub fn new(parameters: Parameters, sinking: Sinking) -> Result<Self, ModelError> {
        Ok(Self {
            parameters: parameters.validate()?,
            sinking: sinking.validate()?,
        })
    }

    pub fn parameters(self) -> Parameters {
        self.parameters
    }

    /// Production matrix from Eq. 2c.
    pub fn production_matrix(
        self,
        state: State,
        irradiance: f64,
    ) -> Result<[[f64; STOCKS]; STOCKS], ModelError> {
        ensure_finite(&[irradiance])?;
        if irradiance < 0.0 {
            return Err(ModelError::InvalidParameter);
        }
        let p = self.parameters;
        let growth = p.maximum_growth * state.nutrient
            / (p.nutrient_half_saturation + state.nutrient)
            * irradiance
            / (p.light_half_saturation + irradiance);
        let grazing = p.maximum_grazing * p.grazing_encounter * state.phytoplankton.powi(2)
            / (p.maximum_grazing + p.grazing_encounter * state.phytoplankton.powi(2));
        let mut matrix = [[0.0; STOCKS]; STOCKS];
        matrix[0][2] = p.linear_zooplankton_loss * state.zooplankton;
        matrix[0][3] = p.remineralization * state.detritus;
        matrix[1][0] = growth * state.phytoplankton;
        matrix[2][1] = grazing * state.zooplankton;
        matrix[3][1] = p.phytoplankton_loss * state.phytoplankton;
        matrix[3][2] = (1.0 - p.assimilation_efficiency) * grazing * state.zooplankton
            + p.quadratic_zooplankton_loss * state.zooplankton.powi(2);
        if matrix.iter().flatten().all(|value| value.is_finite()) {
            Ok(matrix)
        } else {
            Err(ModelError::ArithmeticFailure)
        }
    }

    /// Autonomous derivative `sum_j(P_ij - P_ji)` from Eq. 2b.
    pub fn autonomous_derivative(
        self,
        state: State,
        irradiance: f64,
    ) -> Result<[f64; STOCKS], ModelError> {
        let production = self.production_matrix(state, irradiance)?;
        Ok(std::array::from_fn(|row| {
            (0..STOCKS)
                .map(|column| production[row][column] - production[column][row])
                .sum()
        }))
    }

    /// Instantaneous phytoplankton production flux `J(N,I)P`.
    pub fn primary_production_flux(self, state: State, irradiance: f64) -> Result<f64, ModelError> {
        Ok(self.production_matrix(state, irradiance)?[1][0])
    }

    /// Instantaneous zooplankton grazing flux `G(epsilon,g,P)Z`.
    pub fn grazing_flux(self, state: State, irradiance: f64) -> Result<f64, ModelError> {
        Ok(self.production_matrix(state, irradiance)?[2][1])
    }

    /// Two-linear-solve modified Patankar--Runge--Kutta substep (Eqs. 7a--7b).
    pub fn autonomous_step(
        self,
        state: State,
        elapsed: f64,
        irradiance: f64,
    ) -> Result<State, ModelError> {
        ensure_finite(&[elapsed])?;
        if elapsed < 0.0 {
            return Err(ModelError::InvalidInterval);
        }
        if elapsed == 0.0 {
            return Ok(state);
        }
        let initial = state.values();
        let production_n = self.production_matrix(state, irradiance)?;
        // Eq. 7a contains one evaluation of P(z^n). The matrix helper sums
        // its two production arguments, so one-half weights the duplicated
        // argument back to that single evaluation.
        let omega = patankar_matrix(initial, production_n, production_n, elapsed, 0.5)?;
        let predictor = solve_4x4(omega, initial)?;
        let predictor_state = State::from_values(predictor)?;
        let production_p = self.production_matrix(predictor_state, irradiance)?;
        let corrector = patankar_matrix(predictor, production_n, production_p, elapsed, 0.5)?;
        State::from_values(solve_4x4(corrector, initial)?)
    }

    /// Applies autonomous MPRK, analytic nutrient forcing, then analytic sinking.
    pub fn step(
        self,
        state: State,
        start: f64,
        elapsed: f64,
        irradiance: f64,
        pulses: &[GaussianPulse],
    ) -> Result<Step, ModelError> {
        ensure_finite(&[start, elapsed])?;
        if elapsed < 0.0 {
            return Err(ModelError::InvalidInterval);
        }
        let end = start + elapsed;
        if !end.is_finite() {
            return Err(ModelError::ArithmeticFailure);
        }
        let after_autonomous = self.autonomous_step(state, elapsed, irradiance)?;
        let mut nutrient_input = 0.0;
        for pulse in pulses {
            nutrient_input += pulse.increment(start, end)?;
        }
        if !nutrient_input.is_finite() {
            return Err(ModelError::ArithmeticFailure);
        }
        let (detritus, detritus_export) = self.sinking.apply(after_autonomous.detritus, elapsed)?;
        let after = State::new(
            after_autonomous.nutrient + nutrient_input,
            after_autonomous.phytoplankton,
            after_autonomous.zooplankton,
            detritus,
        )?;
        let ledger_residual = (after.total() - state.total()) - nutrient_input + detritus_export;
        Ok(Step {
            before: state,
            after_autonomous,
            after,
            nutrient_input,
            detritus_export,
            ledger_residual,
        })
    }
}

/// Daily irradiance inferred from Eq. 14b and the paper's multi-day figures.
pub fn repeating_daily_irradiance(time_days: f64) -> Result<f64, ModelError> {
    ensure_finite(&[time_days])?;
    let day = time_days.rem_euclid(1.0);
    if (0.31..=0.73).contains(&day) {
        Ok(15.558_6 / 2.0
            * (libm::sin(100.0 * std::f64::consts::PI * day / 21.0 - 2.0 * std::f64::consts::PI)
                + 1.0))
    } else {
        Ok(0.0)
    }
}

/// Invalid source-model input or failed finite arithmetic.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ModelError {
    NonFinite,
    NonpositiveState,
    InvalidParameter,
    InvalidInterval,
    SingularSystem,
    ArithmeticFailure,
}

impl fmt::Display for ModelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::NonFinite => "paper-model values must be finite",
            Self::NonpositiveState => "MPRK paper states must be strictly positive",
            Self::InvalidParameter => "invalid paper-model parameter",
            Self::InvalidInterval => "invalid paper-model time interval",
            Self::SingularSystem => "paper MPRK linear system is singular",
            Self::ArithmeticFailure => "paper-model arithmetic produced an invalid value",
        };
        formatter.write_str(message)
    }
}

impl Error for ModelError {}

fn patankar_matrix(
    denominator: [f64; STOCKS],
    first: [[f64; STOCKS]; STOCKS],
    second: [[f64; STOCKS]; STOCKS],
    elapsed: f64,
    weight: f64,
) -> Result<[[f64; STOCKS]; STOCKS], ModelError> {
    let mut matrix = [[0.0; STOCKS]; STOCKS];
    for row in 0..STOCKS {
        if denominator[row] <= 0.0 || !denominator[row].is_finite() {
            return Err(ModelError::NonpositiveState);
        }
        matrix[row][row] = 1.0
            + elapsed * weight / denominator[row]
                * (0..STOCKS)
                    .map(|column| first[column][row] + second[column][row])
                    .sum::<f64>();
        for column in 0..STOCKS {
            if row != column {
                matrix[row][column] = -elapsed * weight / denominator[column]
                    * (first[row][column] + second[row][column]);
            }
        }
    }
    if matrix.iter().flatten().all(|value| value.is_finite()) {
        Ok(matrix)
    } else {
        Err(ModelError::ArithmeticFailure)
    }
}

fn solve_4x4(
    mut matrix: [[f64; STOCKS]; STOCKS],
    mut right: [f64; STOCKS],
) -> Result<[f64; STOCKS], ModelError> {
    for pivot in 0..STOCKS {
        let pivot_row = (pivot..STOCKS)
            .max_by(|left, right_row| {
                matrix[*left][pivot]
                    .abs()
                    .total_cmp(&matrix[*right_row][pivot].abs())
            })
            .expect("nonempty pivot range");
        if matrix[pivot_row][pivot] == 0.0 || !matrix[pivot_row][pivot].is_finite() {
            return Err(ModelError::SingularSystem);
        }
        matrix.swap(pivot, pivot_row);
        right.swap(pivot, pivot_row);
        let pivot_values = matrix[pivot];
        for row in (pivot + 1)..STOCKS {
            let factor = matrix[row][pivot] / matrix[pivot][pivot];
            matrix[row][pivot] = 0.0;
            for (column, value) in matrix[row].iter_mut().enumerate().skip(pivot + 1) {
                *value -= factor * pivot_values[column];
            }
            right[row] -= factor * right[pivot];
        }
    }
    let mut solution = [0.0; STOCKS];
    for row in (0..STOCKS).rev() {
        let tail: f64 = matrix[row]
            .iter()
            .zip(&solution)
            .skip(row + 1)
            .map(|(coefficient, value)| coefficient * value)
            .sum();
        solution[row] = (right[row] - tail) / matrix[row][row];
    }
    if solution.iter().all(|value| value.is_finite()) {
        Ok(solution)
    } else {
        Err(ModelError::ArithmeticFailure)
    }
}

fn compensated_sum(values: [f64; STOCKS]) -> f64 {
    let mut sum = 0.0;
    let mut correction = 0.0;
    for value in values {
        let adjusted = value - correction;
        let next = sum + adjusted;
        correction = (next - sum) - adjusted;
        sum = next;
    }
    sum
}

fn ensure_finite(values: &[f64]) -> Result<(), ModelError> {
    if values.iter().all(|value| value.is_finite()) {
        Ok(())
    } else {
        Err(ModelError::NonFinite)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_revalidates_sinking_at_its_trust_boundary() {
        let invalid = Sinking {
            rate: f64::NAN,
            detritus_floor: 1.0,
        };
        assert_eq!(
            Model::new(Parameters::tlqz(), invalid),
            Err(ModelError::NonFinite)
        );
    }
}
