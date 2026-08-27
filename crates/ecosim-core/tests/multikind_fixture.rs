use std::sync::Arc;

use conservation_core::KindId;
use conservation_dynamics::{FlowSpec, FlowTopology, ProcessId, StockDefinition, StockId};
use conservation_stock_flow::{
    BoundaryVerdict, ExactAmounts, FlowId, LinearFlowConstraint, OpenBalanceVerdict, SentenceId,
    StockFlowError, TransitionRecord, TransitionTrace, TransitionVerdict,
    check_boundary_correspondence, check_open_balance, check_transition_equation,
};
use ecosim_core::synthetic_multikind_fixture;
use num_rational::BigRational;

#[test]
fn synthetic_cnp_energy_fixture_satisfies_exact_transition_and_boundaries() {
    let fixture = synthetic_multikind_fixture().unwrap();
    assert!(matches!(
        check_transition_equation(fixture.transition_sentence(), fixture.trace()).unwrap(),
        TransitionVerdict::Satisfied(_)
    ));
    for sentence in fixture.boundary_sentences() {
        assert!(matches!(
            check_boundary_correspondence(sentence, fixture.trace()).unwrap(),
            BoundaryVerdict::Satisfied(_)
        ));
    }
    assert_eq!(fixture.open_balances().len(), 4);
    for sentence in fixture.open_balances() {
        assert!(matches!(
            check_open_balance(sentence, fixture.trace()).unwrap(),
            OpenBalanceVerdict::Satisfied(_)
        ));
    }
}

#[test]
fn topology_rejects_a_carbon_flow_effect_on_a_nitrogen_stock() {
    let carbon = KindId::new("carbon").unwrap();
    let nitrogen = KindId::new("nitrogen").unwrap();
    let c = StockId::new("c-stock").unwrap();
    let n = StockId::new("n-stock").unwrap();
    let result = FlowTopology::new(
        [
            StockDefinition {
                id: c.clone(),
                kind: carbon.clone(),
            },
            StockDefinition {
                id: n.clone(),
                kind: nitrogen,
            },
        ],
        [FlowSpec {
            process: ProcessId::new("invalid-conversion").unwrap(),
            kind: carbon,
            source: Some(c),
            target: Some(n),
        }],
    );
    assert!(result.is_err());
}

#[test]
fn checked_sentence_construction_rejects_a_nitrogen_flow_in_a_carbon_law() {
    let fixture = synthetic_multikind_fixture().unwrap();
    let result = LinearFlowConstraint::new(
        fixture.carrier(),
        SentenceId::new("cross-kind-law").unwrap(),
        KindId::new("carbon").unwrap(),
        [(
            FlowId::new("n-incorporation").unwrap(),
            BigRational::from_integer(1.into()),
        )],
        BigRational::from_integer(0.into()),
    );
    assert!(matches!(
        result,
        Err(StockFlowError::SentenceKindMismatch { expected, actual, .. })
            if expected == KindId::new("carbon").unwrap()
                && actual == KindId::new("nitrogen").unwrap()
    ));
}

fn omitted_energy_boundary_trace(
    boundary_name: &str,
) -> (ecosim_core::SyntheticMultikindFixture, TransitionTrace) {
    let fixture = synthetic_multikind_fixture().unwrap();
    let mut data = fixture.trace().records()[0].clone().into_data();
    data.settled_boundary = ExactAmounts::new(data.settled_boundary.iter().map(
        |(boundary, kind, amount)| {
            let amount = if boundary.as_str() == boundary_name {
                BigRational::from_integer(0.into())
            } else {
                amount.clone()
            };
            (boundary.clone(), kind.clone(), amount)
        },
    ))
    .unwrap();
    let record = TransitionRecord::new(fixture.carrier(), data).unwrap();
    let trace = TransitionTrace::new(Arc::clone(fixture.carrier()), vec![record]).unwrap();

    (fixture, trace)
}

fn assert_visible_energy_boundary_failure(boundary_name: &str) {
    let (fixture, trace) = omitted_energy_boundary_trace(boundary_name);

    assert!(matches!(
        check_open_balance(&fixture.open_balances()[3], &trace).unwrap(),
        OpenBalanceVerdict::Violated(violation)
            if violation.residual != BigRational::from_integer(0.into())
    ));
    assert!(matches!(
        check_boundary_correspondence(&fixture.boundary_sentences()[7], &trace).unwrap(),
        BoundaryVerdict::Violated(violation)
            if violation.transition == 0
                && violation.ledger_axis.as_str() == "cumulative-energy-output"
                && violation.boundaries.iter().any(|boundary| boundary.as_str() == boundary_name)
    ));
}

#[test]
fn omitting_heat_is_a_visible_energy_balance_and_ledger_failure() {
    assert_visible_energy_boundary_failure("energy-heat");
}

#[test]
fn omitting_export_is_a_visible_energy_balance_and_ledger_failure() {
    assert_visible_energy_boundary_failure("energy-export");
}
