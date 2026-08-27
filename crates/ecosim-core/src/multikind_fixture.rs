//! Synthetic C/N/P/energy fixture for reviewing typed open balances.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

use conservation_core::{AxisId, KindId};
use conservation_dynamics::{FlowSpec, FlowTopology, ProcessId, StockDefinition, StockId};
use conservation_stock_flow::{
    BoundaryCorrespondence, BoundaryId, ChannelId, ExactAmounts, FlowId, LedgerDefinition,
    LedgerId, OpenBalance, SentenceId, StockAxisDefinition, StockFlowCarrier, StockFlowError,
    TransitionEquation, TransitionRecord, TransitionRecordData, TransitionTrace, certify_nullspace,
};
use num_rational::BigRational;

/// A mathematical fixture, not an empirically calibrated ecosystem.
#[derive(Debug)]
pub struct SyntheticMultikindFixture {
    carrier: Arc<StockFlowCarrier>,
    trace: TransitionTrace,
    transition_sentence: TransitionEquation,
    boundary_sentences: Vec<BoundaryCorrespondence>,
    open_balances: Vec<OpenBalance>,
}

impl SyntheticMultikindFixture {
    /// Exact typed carrier defining the fixture's system boundary.
    pub fn carrier(&self) -> &Arc<StockFlowCarrier> {
        &self.carrier
    }

    /// One exact transition with explicit inputs, heat, work, and exports.
    pub fn trace(&self) -> &TransitionTrace {
        &self.trace
    }

    /// Complete state-transition sentence.
    pub fn transition_sentence(&self) -> &TransitionEquation {
        &self.transition_sentence
    }

    /// One ledger-correspondence sentence per quantity-kind boundary.
    pub fn boundary_sentences(&self) -> &[BoundaryCorrespondence] {
        &self.boundary_sentences
    }

    /// Separate checked C, N, P, and stored-energy open balances.
    pub fn open_balances(&self) -> &[OpenBalance] {
        &self.open_balances
    }
}

/// Constructs the exact synthetic C/N/P/stored-energy fixture.
///
/// C, N, and P enter an available pool, may transfer internally into biomass,
/// and may leave through an explicit export port. Energy enters one stored
/// pool and leaves through explicit heat, work, and export ports. No coefficient
/// or flow crosses quantity kinds.
pub fn synthetic_multikind_fixture() -> Result<SyntheticMultikindFixture, SyntheticFixtureError> {
    let kinds = BTreeMap::from([
        ("c", kind("carbon")?),
        ("n", kind("nitrogen")?),
        ("p", kind("phosphorus")?),
        ("e", kind("stored-energy")?),
    ]);
    let stock_specs = [
        ("c-available", "c"),
        ("c-biomass", "c"),
        ("n-available", "n"),
        ("n-biomass", "n"),
        ("p-available", "p"),
        ("p-biomass", "p"),
        ("energy-stored", "e"),
    ];
    let stocks = stock_specs
        .iter()
        .map(|(name, key)| {
            Ok(StockDefinition {
                id: stock(name)?,
                kind: kinds[*key].clone(),
            })
        })
        .collect::<Result<Vec<_>, SyntheticFixtureError>>()?;

    let flows = vec![
        input("c-input", "c-available", &kinds["c"])?,
        transfer("c-incorporation", "c-available", "c-biomass", &kinds["c"])?,
        output("c-export", "c-biomass", &kinds["c"])?,
        input("n-input", "n-available", &kinds["n"])?,
        transfer("n-incorporation", "n-available", "n-biomass", &kinds["n"])?,
        output("n-export", "n-biomass", &kinds["n"])?,
        input("p-input", "p-available", &kinds["p"])?,
        transfer("p-incorporation", "p-available", "p-biomass", &kinds["p"])?,
        output("p-export", "p-biomass", &kinds["p"])?,
        input("energy-input", "energy-stored", &kinds["e"])?,
        output("energy-heat", "energy-stored", &kinds["e"])?,
        output("energy-work", "energy-stored", &kinds["e"])?,
        output("energy-export", "energy-stored", &kinds["e"])?,
    ];
    let topology = Arc::new(FlowTopology::new(stocks, flows)?);
    let stock_axes = stock_specs
        .iter()
        .map(|(name, _)| {
            Ok(StockAxisDefinition {
                stock: stock(name)?,
                axis: axis(name)?,
            })
        })
        .collect::<Result<Vec<_>, SyntheticFixtureError>>()?;
    let channels = vec![
        boundary("c-input")?,
        internal("c-incorporation")?,
        boundary("c-export")?,
        boundary("n-input")?,
        internal("n-incorporation")?,
        boundary("n-export")?,
        boundary("p-input")?,
        internal("p-incorporation")?,
        boundary("p-export")?,
        boundary("energy-input")?,
        boundary("energy-heat")?,
        boundary("energy-work")?,
        boundary("energy-export")?,
    ];
    let ledger_specs = [
        ("carbon-input", "c", &["c-input"][..]),
        ("carbon-output", "c", &["c-export"][..]),
        ("nitrogen-input", "n", &["n-input"][..]),
        ("nitrogen-output", "n", &["n-export"][..]),
        ("phosphorus-input", "p", &["p-input"][..]),
        ("phosphorus-output", "p", &["p-export"][..]),
        ("energy-input", "e", &["energy-input"][..]),
        (
            "energy-output",
            "e",
            &["energy-heat", "energy-work", "energy-export"][..],
        ),
    ];
    let ledgers = ledger_specs
        .iter()
        .map(|(name, key, ports)| {
            Ok(LedgerDefinition {
                id: ledger(name)?,
                axis: axis(&format!("cumulative-{name}"))?,
                kind: kinds[*key].clone(),
                boundaries: ports
                    .iter()
                    .map(|port| boundary_id(port))
                    .collect::<Result<Vec<_>, _>>()?,
            })
        })
        .collect::<Result<Vec<_>, SyntheticFixtureError>>()?;
    let carrier = Arc::new(StockFlowCarrier::new(
        topology, stock_axes, channels, ledgers,
    )?);

    let before = amounts_axis(
        &kinds,
        &[
            ("c-available", "c", 0),
            ("c-biomass", "c", 0),
            ("n-available", "n", 0),
            ("n-biomass", "n", 0),
            ("p-available", "p", 0),
            ("p-biomass", "p", 0),
            ("energy-stored", "e", 0),
        ],
    )?;
    let after = amounts_axis(
        &kinds,
        &[
            ("c-available", "c", 1),
            ("c-biomass", "c", 1),
            ("n-available", "n", 1),
            ("n-biomass", "n", 1),
            ("p-available", "p", 0),
            ("p-biomass", "p", 1),
            ("energy-stored", "e", 3),
        ],
    )?;
    let internal_values = amounts_flow(
        &kinds,
        &[
            ("c-incorporation", "c", 2),
            ("n-incorporation", "n", 1),
            ("p-incorporation", "p", 1),
        ],
    )?;
    let boundary_values = amounts_boundary(
        &kinds,
        &[
            ("c-input", "c", 3),
            ("c-export", "c", 1),
            ("n-input", "n", 2),
            ("n-export", "n", 0),
            ("p-input", "p", 1),
            ("p-export", "p", 0),
            ("energy-input", "e", 10),
            ("energy-heat", "e", 4),
            ("energy-work", "e", 2),
            ("energy-export", "e", 1),
        ],
    )?;
    let ledger_before = amounts_ledger(
        &kinds,
        &[
            ("carbon-input", "c", 0),
            ("carbon-output", "c", 0),
            ("nitrogen-input", "n", 0),
            ("nitrogen-output", "n", 0),
            ("phosphorus-input", "p", 0),
            ("phosphorus-output", "p", 0),
            ("energy-input", "e", 0),
            ("energy-output", "e", 0),
        ],
    )?;
    let ledger_after = amounts_ledger(
        &kinds,
        &[
            ("carbon-input", "c", 3),
            ("carbon-output", "c", 1),
            ("nitrogen-input", "n", 2),
            ("nitrogen-output", "n", 0),
            ("phosphorus-input", "p", 1),
            ("phosphorus-output", "p", 0),
            ("energy-input", "e", 10),
            ("energy-output", "e", 7),
        ],
    )?;
    let record = TransitionRecord::new(
        &carrier,
        TransitionRecordData {
            before,
            after,
            requested_internal: internal_values.clone(),
            settled_internal: internal_values,
            requested_boundary: boundary_values.clone(),
            settled_boundary: boundary_values,
            ledger_before,
            ledger_after,
        },
    )?;
    let trace = TransitionTrace::new(Arc::clone(&carrier), vec![record])?;
    let transition_sentence = TransitionEquation::new(sentence("multikind-transition")?);
    let boundary_sentences = ledger_specs
        .iter()
        .map(|(name, _, _)| {
            Ok(BoundaryCorrespondence::new(
                sentence(&format!("{name}-correspondence"))?,
                ledger(name)?,
            ))
        })
        .collect::<Result<Vec<_>, SyntheticFixtureError>>()?;
    let balance_specs = [
        (
            "carbon-open-balance",
            "c",
            &["c-available", "c-biomass"][..],
        ),
        (
            "nitrogen-open-balance",
            "n",
            &["n-available", "n-biomass"][..],
        ),
        (
            "phosphorus-open-balance",
            "p",
            &["p-available", "p-biomass"][..],
        ),
        ("stored-energy-open-balance", "e", &["energy-stored"][..]),
    ];
    let open_balances = balance_specs
        .iter()
        .map(|(name, key, axes)| {
            let certificate = certify_nullspace(
                &carrier,
                kinds[*key].clone(),
                axes.iter()
                    .map(|name| Ok((axis(name)?, q(1))))
                    .collect::<Result<Vec<_>, SyntheticFixtureError>>()?,
            )?;
            Ok(certificate.open_balance(sentence(name)?))
        })
        .collect::<Result<Vec<_>, SyntheticFixtureError>>()?;

    Ok(SyntheticMultikindFixture {
        carrier,
        trace,
        transition_sentence,
        boundary_sentences,
        open_balances,
    })
}

/// Construction failure for the checked synthetic fixture.
#[derive(Debug)]
pub enum SyntheticFixtureError {
    /// A stable identifier was rejected.
    Identifier(String),
    /// The settlement topology rejected an incompatible stock/flow declaration.
    Topology(conservation_dynamics::StockFlowError),
    /// The exact carrier or record was structurally invalid.
    Carrier(StockFlowError),
}

impl fmt::Display for SyntheticFixtureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Identifier(message) => write!(formatter, "invalid fixture identifier: {message}"),
            Self::Topology(error) => error.fmt(formatter),
            Self::Carrier(error) => error.fmt(formatter),
        }
    }
}

impl Error for SyntheticFixtureError {}

impl From<conservation_dynamics::StockFlowError> for SyntheticFixtureError {
    fn from(value: conservation_dynamics::StockFlowError) -> Self {
        Self::Topology(value)
    }
}

impl From<StockFlowError> for SyntheticFixtureError {
    fn from(value: StockFlowError) -> Self {
        Self::Carrier(value)
    }
}

fn identifier<T>(result: Result<T, impl fmt::Display>) -> Result<T, SyntheticFixtureError> {
    result.map_err(|error| SyntheticFixtureError::Identifier(error.to_string()))
}

fn kind(name: &str) -> Result<KindId, SyntheticFixtureError> {
    identifier(KindId::new(name))
}
fn stock(name: &str) -> Result<StockId, SyntheticFixtureError> {
    identifier(StockId::new(name))
}
fn axis(name: &str) -> Result<AxisId, SyntheticFixtureError> {
    identifier(AxisId::new(name))
}
fn process(name: &str) -> Result<ProcessId, SyntheticFixtureError> {
    identifier(ProcessId::new(name))
}
fn internal_id(name: &str) -> Result<FlowId, SyntheticFixtureError> {
    identifier(FlowId::new(name))
}
fn boundary_id(name: &str) -> Result<BoundaryId, SyntheticFixtureError> {
    identifier(BoundaryId::new(name))
}
fn ledger(name: &str) -> Result<LedgerId, SyntheticFixtureError> {
    identifier(LedgerId::new(name))
}
fn sentence(name: &str) -> Result<SentenceId, SyntheticFixtureError> {
    identifier(SentenceId::new(name))
}

fn input(name: &str, target: &str, kind: &KindId) -> Result<FlowSpec, SyntheticFixtureError> {
    Ok(FlowSpec {
        process: process(name)?,
        kind: kind.clone(),
        source: None,
        target: Some(stock(target)?),
    })
}
fn output(name: &str, source: &str, kind: &KindId) -> Result<FlowSpec, SyntheticFixtureError> {
    Ok(FlowSpec {
        process: process(name)?,
        kind: kind.clone(),
        source: Some(stock(source)?),
        target: None,
    })
}
fn transfer(
    name: &str,
    source: &str,
    target: &str,
    kind: &KindId,
) -> Result<FlowSpec, SyntheticFixtureError> {
    Ok(FlowSpec {
        process: process(name)?,
        kind: kind.clone(),
        source: Some(stock(source)?),
        target: Some(stock(target)?),
    })
}
fn internal(name: &str) -> Result<ChannelId, SyntheticFixtureError> {
    Ok(ChannelId::Internal(internal_id(name)?))
}
fn boundary(name: &str) -> Result<ChannelId, SyntheticFixtureError> {
    Ok(ChannelId::Boundary(boundary_id(name)?))
}

fn q(value: i64) -> BigRational {
    BigRational::from_integer(value.into())
}

fn amounts_axis(
    kinds: &BTreeMap<&str, KindId>,
    values: &[(&str, &str, i64)],
) -> Result<ExactAmounts<AxisId>, SyntheticFixtureError> {
    Ok(ExactAmounts::new(
        values
            .iter()
            .map(|(name, key, value)| Ok((axis(name)?, kinds[key].clone(), q(*value))))
            .collect::<Result<Vec<_>, SyntheticFixtureError>>()?,
    )?)
}
fn amounts_flow(
    kinds: &BTreeMap<&str, KindId>,
    values: &[(&str, &str, i64)],
) -> Result<ExactAmounts<FlowId>, SyntheticFixtureError> {
    Ok(ExactAmounts::new(
        values
            .iter()
            .map(|(name, key, value)| Ok((internal_id(name)?, kinds[key].clone(), q(*value))))
            .collect::<Result<Vec<_>, SyntheticFixtureError>>()?,
    )?)
}
fn amounts_boundary(
    kinds: &BTreeMap<&str, KindId>,
    values: &[(&str, &str, i64)],
) -> Result<ExactAmounts<BoundaryId>, SyntheticFixtureError> {
    Ok(ExactAmounts::new(
        values
            .iter()
            .map(|(name, key, value)| Ok((boundary_id(name)?, kinds[key].clone(), q(*value))))
            .collect::<Result<Vec<_>, SyntheticFixtureError>>()?,
    )?)
}
fn amounts_ledger(
    kinds: &BTreeMap<&str, KindId>,
    values: &[(&str, &str, i64)],
) -> Result<ExactAmounts<LedgerId>, SyntheticFixtureError> {
    Ok(ExactAmounts::new(
        values
            .iter()
            .map(|(name, key, value)| Ok((ledger(name)?, kinds[key].clone(), q(*value))))
            .collect::<Result<Vec<_>, SyntheticFixtureError>>()?,
    )?)
}
