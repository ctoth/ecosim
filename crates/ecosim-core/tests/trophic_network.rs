use std::collections::BTreeMap;
use std::sync::Arc;

use conservation_core::{AxisId, Grade};
use conservation_stock_flow::{
    BoundaryCorrespondence, BoundaryVerdict, ExactAmounts, FlowConstraintVerdict, FlowId, LedgerId,
    LinearFlowConstraint, SentenceId, TransitionEquation, TransitionRecord, TransitionTrace,
    TransitionVerdict, check_boundary_correspondence, check_linear_flow_constraint,
    check_transition_equation,
};
use conservation_trace::{LawVerdict, LawWitness};
use ecosim_core::{
    ConsumerSpec, DenseTrophicNetwork, DenseTrophicNetworkPlan, ExactTrophicNetwork,
    ExactTrophicNetworkPlan, FeedingSpec, ProducerSpec, TROPHIC_DENSE_BALANCE_ABSOLUTE_TOLERANCE,
    TROPHIC_DENSE_BALANCE_RELATIVE_TOLERANCE, TrophicBoundary, TrophicFlow, TrophicLaw,
    TrophicLawEvidence, TrophicLawVerdict, TrophicNetworkError, TrophicNetworkSpec,
    TrophicSentenceFamily, trophic_dense_balance_tolerance,
};
use num_rational::BigRational;
use num_traits::{Signed, ToPrimitive};
use proptest::prelude::*;

fn ratio(numerator: i64, denominator: i64) -> BigRational {
    BigRational::new(numerator.into(), denominator.into())
}

fn exact_spec(grazing: BigRational) -> TrophicNetworkSpec<BigRational> {
    TrophicNetworkSpec::new(
        vec![
            ProducerSpec::new("grazed", ratio(1, 2), ratio(10, 1), ratio(1, 100)),
            ProducerSpec::new("refuge", ratio(1, 2), ratio(10, 1), ratio(1, 100)),
        ],
        vec![ConsumerSpec::new("grazer", ratio(1, 50))],
        vec![FeedingSpec::new(
            "grazer",
            "grazed",
            grazing,
            ratio(5, 1),
            ratio(3, 4),
        )],
        ratio(1, 10),
    )
}

fn dense_spec(grazing: f64) -> TrophicNetworkSpec<f64> {
    TrophicNetworkSpec::new(
        vec![
            ProducerSpec::new("grazed", 0.5, 10.0, 0.01),
            ProducerSpec::new("refuge", 0.5, 10.0, 0.01),
        ],
        vec![ConsumerSpec::new("grazer", 0.02)],
        vec![FeedingSpec::new("grazer", "grazed", grazing, 5.0, 0.75)],
        0.1,
    )
}

fn exact_initial() -> BTreeMap<String, BigRational> {
    BTreeMap::from([
        ("nutrient".to_owned(), ratio(100, 1)),
        ("grazed".to_owned(), ratio(10, 1)),
        ("refuge".to_owned(), ratio(10, 1)),
        ("grazer".to_owned(), ratio(5, 1)),
        ("detritus".to_owned(), ratio(0, 1)),
    ])
}

fn dense_initial() -> BTreeMap<String, f64> {
    BTreeMap::from([
        ("nutrient".to_owned(), 100.0),
        ("grazed".to_owned(), 10.0),
        ("refuge".to_owned(), 10.0),
        ("grazer".to_owned(), 5.0),
        ("detritus".to_owned(), 0.0),
    ])
}

fn evidence_for<'a>(
    evidence: &'a ecosim_core::ExactTrophicNetworkEvidence,
    law: &TrophicLaw,
) -> &'a TrophicLawEvidence {
    evidence
        .laws()
        .iter()
        .find(|candidate| candidate.law() == law)
        .expect("compiled evidence contains every named law")
}

#[test]
fn selective_grazing_is_observable_and_exactly_conservative() {
    let mut network = ExactTrophicNetwork::new(exact_spec(ratio(4, 5)), exact_initial()).unwrap();
    let step = network
        .step(ratio(1, 1), ratio(0, 1), BTreeMap::new())
        .unwrap();

    assert!(step.growth("grazed").unwrap().is_positive());
    assert!(step.growth("refuge").unwrap().is_positive());
    assert!(step.feeding("grazer", "grazed").unwrap().is_positive());
    assert!(step.feeding("grazer", "refuge").is_none());
    assert!(network.stock("refuge").unwrap() > network.stock("grazed").unwrap());
    assert!(network.stock("grazer").unwrap() > &ratio(5, 1));
    assert!(network.stock("detritus").unwrap().is_positive());
    assert!(network.is_balanced());
}

#[test]
fn exact_and_dense_networks_share_one_step_semantics() {
    let mut exact = ExactTrophicNetwork::new(exact_spec(ratio(4, 5)), exact_initial()).unwrap();
    let mut dense = DenseTrophicNetwork::new(dense_spec(0.8), dense_initial()).unwrap();

    let exact_step = exact
        .step(
            ratio(1, 4),
            ratio(2, 1),
            BTreeMap::from([("grazer".to_owned(), ratio(1, 2))]),
        )
        .unwrap();
    let dense_step = dense
        .step(0.25, 2.0, BTreeMap::from([("grazer".to_owned(), 0.5)]))
        .unwrap();

    for stock in ["nutrient", "grazed", "refuge", "grazer", "detritus"] {
        let exact_value = exact.stock(stock).unwrap().to_f64().unwrap();
        assert!((dense.stock(stock).unwrap() - exact_value).abs() < 1e-12);
    }
    for producer in ["grazed", "refuge"] {
        let exact_value = exact_step.growth(producer).unwrap().to_f64().unwrap();
        assert!((dense_step.growth(producer).unwrap() - exact_value).abs() < 1e-12);
    }
    assert!(dense.is_balanced());
}

#[test]
fn one_compiled_plan_starts_independent_runs() {
    let plan = ExactTrophicNetworkPlan::compile(exact_spec(ratio(4, 5))).unwrap();
    let mut perturbed = plan.start(exact_initial()).unwrap();
    let untouched = plan.start(exact_initial()).unwrap();

    perturbed
        .step(ratio(1, 1), ratio(3, 1), BTreeMap::new())
        .unwrap();

    assert_eq!(plan.stock_names(), untouched.stock_names());
    assert_eq!(untouched.time(), &ratio(0, 1));
    assert_eq!(untouched.stock("nutrient"), Some(&ratio(100, 1)));
    assert_eq!(perturbed.trace().len(), 2);
    assert_eq!(untouched.trace().len(), 1);
    assert_ne!(perturbed.stock("nutrient"), untouched.stock("nutrient"));
    assert!(perturbed.is_balanced());
    assert!(untouched.is_balanced());
}

#[test]
fn exact_plan_compiles_stable_axes_and_graded_sentences() {
    let plan = ExactTrophicNetworkPlan::compile(exact_spec(ratio(4, 5))).unwrap();

    assert_eq!(
        plan.evidence_axis_names().collect::<Vec<_>>(),
        vec![
            "nutrient",
            "grazed",
            "refuge",
            "grazer",
            "detritus",
            "cumulative_input",
            "cumulative_output",
        ]
    );
    assert_eq!(
        plan.flow_symbols(),
        [
            TrophicFlow::NutrientInput,
            TrophicFlow::ProducerGrowth("grazed".to_owned()),
            TrophicFlow::ProducerGrowth("refuge".to_owned()),
            TrophicFlow::FeedingAssimilation {
                consumer: "grazer".to_owned(),
                resource: "grazed".to_owned(),
            },
            TrophicFlow::FeedingWaste {
                consumer: "grazer".to_owned(),
                resource: "grazed".to_owned(),
            },
            TrophicFlow::Mortality("grazed".to_owned()),
            TrophicFlow::Mortality("refuge".to_owned()),
            TrophicFlow::Mortality("grazer".to_owned()),
            TrophicFlow::Decomposition,
            TrophicFlow::Harvest("grazer".to_owned()),
        ]
    );
    assert_eq!(
        plan.boundary_symbols(),
        [
            TrophicBoundary::NutrientInput,
            TrophicBoundary::Harvest("grazer".to_owned()),
        ]
    );
    assert_eq!(
        plan.evidence_laws().cloned().collect::<Vec<_>>(),
        vec![
            TrophicLaw::MaterialInvariant,
            TrophicLaw::StockNonnegative("nutrient".to_owned()),
            TrophicLaw::StockNonnegative("grazed".to_owned()),
            TrophicLaw::StockNonnegative("refuge".to_owned()),
            TrophicLaw::StockNonnegative("grazer".to_owned()),
            TrophicLaw::StockNonnegative("detritus".to_owned()),
            TrophicLaw::CumulativeInputNondecreasing,
            TrophicLaw::CumulativeOutputNondecreasing,
        ]
    );
}

#[test]
fn exact_trace_records_committed_settled_state_and_typed_witnesses() {
    let plan = ExactTrophicNetworkPlan::compile(exact_spec(ratio(4, 5))).unwrap();
    let mut network = plan.start(exact_initial()).unwrap();
    assert_eq!(network.trace().len(), 1);
    assert!(network.transitions().is_empty());
    assert!(network.evidence().is_err());

    network
        .step(
            ratio(1, 4),
            ratio(2, 1),
            BTreeMap::from([("grazer".to_owned(), ratio(1, 2))]),
        )
        .unwrap();

    assert_eq!(network.trace().len(), 2);
    assert_eq!(network.transitions().len(), 1);
    let transition = &network.transitions()[0];
    assert_eq!(transition.index(), 0);
    assert_eq!(transition.time_before(), &ratio(0, 1));
    assert_eq!(transition.time_after(), &ratio(1, 4));
    assert_eq!(
        transition.stocks_before(),
        [
            ratio(100, 1),
            ratio(10, 1),
            ratio(10, 1),
            ratio(5, 1),
            ratio(0, 1),
        ]
    );
    assert_eq!(transition.stocks_after(), network.amounts());
    assert_eq!(transition.inputs_before(), &ratio(0, 1));
    assert_eq!(transition.inputs_after(), &ratio(2, 1));
    assert_eq!(transition.outputs_before(), &ratio(0, 1));
    assert_eq!(transition.outputs_after(), &ratio(1, 2));
    let final_state = network.trace().last().unwrap();
    for name in network.stock_names() {
        assert_eq!(
            final_state.value(&AxisId::new(name.clone()).unwrap()),
            network.stock(name)
        );
    }
    assert_eq!(
        final_state.value(&AxisId::new("cumulative_input").unwrap()),
        Some(&network.inputs())
    );
    assert_eq!(
        final_state.value(&AxisId::new("cumulative_output").unwrap()),
        Some(&network.outputs())
    );

    let evidence = network.evidence().unwrap();
    assert!(evidence.is_satisfied());
    let invariant = evidence_for(&evidence, &TrophicLaw::MaterialInvariant);
    assert_eq!(invariant.grade(), Grade::Invariant);
    assert!(matches!(
        invariant.verdict(),
        LawVerdict::Satisfied(LawWitness::Invariant(witness))
            if witness.states_checked == 2 && witness.conserved_value == ratio(125, 1)
    ));
    for name in network.stock_names() {
        let stock = evidence_for(&evidence, &TrophicLaw::StockNonnegative(name.clone()));
        assert_eq!(stock.grade(), Grade::Nonnegative);
        assert!(matches!(
            stock.verdict(),
            LawVerdict::Satisfied(LawWitness::Nonnegative(witness))
                if witness.states_checked == 2
        ));
    }
    for law in [
        TrophicLaw::CumulativeInputNondecreasing,
        TrophicLaw::CumulativeOutputNondecreasing,
    ] {
        let boundary = evidence_for(&evidence, &law);
        assert_eq!(boundary.grade(), Grade::Nondecreasing);
        assert!(matches!(
            boundary.verdict(),
            LawVerdict::Satisfied(LawWitness::Nondecreasing(witness))
                if witness.states_checked == 2
        ));
    }
}

#[test]
fn complete_exact_law_suite_has_stable_typed_process_and_boundary_evidence() {
    let plan = ExactTrophicNetworkPlan::compile(exact_spec(ratio(4, 5))).unwrap();
    let mut network = plan.start(exact_initial()).unwrap();
    network
        .step(
            ratio(1, 4),
            ratio(2, 1),
            BTreeMap::from([("grazer".to_owned(), ratio(1, 2))]),
        )
        .unwrap();

    let evidence = network.law_evidence().unwrap();
    assert!(evidence.is_satisfied());
    assert_eq!(
        evidence
            .laws()
            .iter()
            .map(|item| item.law().clone())
            .collect::<Vec<_>>(),
        vec![
            TrophicLaw::TransitionEquation,
            TrophicLaw::FeedingPartition {
                consumer: "grazer".to_owned(),
                resource: "grazed".to_owned(),
            },
            TrophicLaw::NutrientInputCorrespondence,
            TrophicLaw::HarvestCorrespondence("grazer".to_owned()),
            TrophicLaw::MaterialInvariant,
            TrophicLaw::StockNonnegative("detritus".to_owned()),
            TrophicLaw::StockNonnegative("grazed".to_owned()),
            TrophicLaw::StockNonnegative("grazer".to_owned()),
            TrophicLaw::StockNonnegative("nutrient".to_owned()),
            TrophicLaw::StockNonnegative("refuge".to_owned()),
            TrophicLaw::CumulativeInputNondecreasing,
            TrophicLaw::CumulativeOutputNondecreasing,
            TrophicLaw::OpenMaterialBalance,
        ]
    );
    assert!(matches!(
        evidence.laws()[0].verdict(),
        TrophicLawVerdict::Transition(verdict) if verdict.is_satisfied()
    ));
    assert_eq!(
        evidence.laws()[1].family(),
        TrophicSentenceFamily::FlowConstraint
    );
    assert_eq!(evidence.laws()[1].grade(), None);
    assert!(matches!(
        evidence.laws().last().unwrap().verdict(),
        TrophicLawVerdict::OpenBalance(verdict) if verdict.is_satisfied()
    ));
}

#[test]
fn balanced_but_misrouted_state_change_fails_the_transition_equation() {
    let plan = ExactTrophicNetworkPlan::compile(exact_spec(ratio(4, 5))).unwrap();
    let mut network = plan.start(exact_initial()).unwrap();
    network
        .step(ratio(1, 4), ratio(0, 1), BTreeMap::new())
        .unwrap();
    let mut data = network.transitions()[0].record().clone().into_data();
    data.after = ExactAmounts::new(data.after.iter().map(|(axis, kind, amount)| {
        let corrupted = match axis.as_str() {
            "grazed" => amount + ratio(1, 1),
            "refuge" => amount - ratio(1, 1),
            _ => amount.clone(),
        };
        (axis.clone(), kind.clone(), corrupted)
    }))
    .unwrap();
    let record = TransitionRecord::new(plan.stock_flow_carrier(), data).unwrap();
    let trace =
        TransitionTrace::new(Arc::new(plan.stock_flow_carrier().clone()), vec![record]).unwrap();

    let verdict = check_transition_equation(
        &TransitionEquation::new(SentenceId::new("misrouting-counterexample").unwrap()),
        &trace,
    )
    .unwrap();
    assert!(matches!(verdict, TransitionVerdict::Violated(_)));
}

#[test]
fn balanced_feeding_with_the_wrong_partition_fails_its_process_constraint() {
    let plan = ExactTrophicNetworkPlan::compile(exact_spec(ratio(4, 5))).unwrap();
    let mut network = plan.start(exact_initial()).unwrap();
    network
        .step(ratio(1, 4), ratio(0, 1), BTreeMap::new())
        .unwrap();
    let assimilation = FlowId::new("feeding-assimilated:grazer:grazed").unwrap();
    let waste = FlowId::new("feeding-waste:grazer:grazed").unwrap();
    let mut data = network.transitions()[0].record().clone().into_data();
    data.settled_internal =
        ExactAmounts::new(data.settled_internal.iter().map(|(flow, kind, amount)| {
            let corrupted = if flow == &assimilation {
                ratio(0, 1)
            } else {
                amount.clone()
            };
            (flow.clone(), kind.clone(), corrupted)
        }))
        .unwrap();
    let record = TransitionRecord::new(plan.stock_flow_carrier(), data).unwrap();
    let trace =
        TransitionTrace::new(Arc::new(plan.stock_flow_carrier().clone()), vec![record]).unwrap();
    let sentence = LinearFlowConstraint::new(
        plan.stock_flow_carrier(),
        SentenceId::new("feeding-counterexample").unwrap(),
        conservation_core::KindId::new("material-equivalent").unwrap(),
        [(assimilation, ratio(1, 4)), (waste, ratio(-3, 4))],
        ratio(0, 1),
    )
    .unwrap();

    assert!(matches!(
        check_linear_flow_constraint(&sentence, &trace).unwrap(),
        FlowConstraintVerdict::Violated(_)
    ));
}

#[test]
fn unchanged_input_ledger_with_settled_input_fails_boundary_correspondence() {
    let plan = ExactTrophicNetworkPlan::compile(exact_spec(ratio(4, 5))).unwrap();
    let mut network = plan.start(exact_initial()).unwrap();
    network
        .step(ratio(1, 4), ratio(2, 1), BTreeMap::new())
        .unwrap();
    let input_ledger = LedgerId::new("cumulative_input").unwrap();
    let mut data = network.transitions()[0].record().clone().into_data();
    let before = data.ledger_before.amount(&input_ledger).unwrap().clone();
    data.ledger_after =
        ExactAmounts::new(data.ledger_after.iter().map(|(ledger, kind, amount)| {
            let corrupted = if ledger == &input_ledger {
                before.clone()
            } else {
                amount.clone()
            };
            (ledger.clone(), kind.clone(), corrupted)
        }))
        .unwrap();
    let record = TransitionRecord::new(plan.stock_flow_carrier(), data).unwrap();
    let trace =
        TransitionTrace::new(Arc::new(plan.stock_flow_carrier().clone()), vec![record]).unwrap();
    let sentence = BoundaryCorrespondence::new(
        SentenceId::new("input-ledger-counterexample").unwrap(),
        input_ledger,
    );

    assert!(matches!(
        check_boundary_correspondence(&sentence, &trace).unwrap(),
        BoundaryVerdict::Violated(_)
    ));
}

#[test]
fn swapping_two_harvest_ledger_attributions_fails_both_port_correspondences() {
    let spec = TrophicNetworkSpec::new(
        vec![ProducerSpec::new(
            "plant",
            ratio(0, 1),
            ratio(1, 1),
            ratio(0, 1),
        )],
        vec![
            ConsumerSpec::new("grazer", ratio(0, 1)),
            ConsumerSpec::new("hunter", ratio(0, 1)),
        ],
        vec![],
        ratio(0, 1),
    );
    let plan = ExactTrophicNetworkPlan::compile(spec).unwrap();
    let mut network = plan
        .start(BTreeMap::from([
            ("nutrient".to_owned(), ratio(0, 1)),
            ("plant".to_owned(), ratio(0, 1)),
            ("grazer".to_owned(), ratio(1, 1)),
            ("hunter".to_owned(), ratio(2, 1)),
            ("detritus".to_owned(), ratio(0, 1)),
        ]))
        .unwrap();
    network
        .step(
            ratio(1, 1),
            ratio(0, 1),
            BTreeMap::from([
                ("grazer".to_owned(), ratio(3, 1)),
                ("hunter".to_owned(), ratio(3, 1)),
            ]),
        )
        .unwrap();

    let grazer = LedgerId::new("cumulative_harvest:grazer").unwrap();
    let hunter = LedgerId::new("cumulative_harvest:hunter").unwrap();
    let mut data = network.transitions()[0].record().clone().into_data();
    let grazer_after = data.ledger_after.amount(&grazer).unwrap().clone();
    let hunter_after = data.ledger_after.amount(&hunter).unwrap().clone();
    data.ledger_after =
        ExactAmounts::new(data.ledger_after.iter().map(|(ledger, kind, amount)| {
            let misattributed = if ledger == &grazer {
                hunter_after.clone()
            } else if ledger == &hunter {
                grazer_after.clone()
            } else {
                amount.clone()
            };
            (ledger.clone(), kind.clone(), misattributed)
        }))
        .unwrap();
    let record = TransitionRecord::new(plan.stock_flow_carrier(), data).unwrap();
    let trace =
        TransitionTrace::new(Arc::new(plan.stock_flow_carrier().clone()), vec![record]).unwrap();

    for (name, ledger) in [("grazer", grazer), ("hunter", hunter)] {
        let sentence = BoundaryCorrespondence::new(
            SentenceId::new(format!("wrong-harvest-port:{name}")).unwrap(),
            ledger,
        );
        assert!(matches!(
            check_boundary_correspondence(&sentence, &trace).unwrap(),
            BoundaryVerdict::Violated(_)
        ));
    }
}

#[test]
fn rejected_exact_step_leaves_trace_and_all_observables_unchanged() {
    let plan = ExactTrophicNetworkPlan::compile(exact_spec(ratio(4, 5))).unwrap();
    let mut network = plan.start(exact_initial()).unwrap();
    network
        .step(ratio(1, 4), ratio(1, 1), BTreeMap::new())
        .unwrap();
    let before = (
        network.time().clone(),
        network.amounts().to_vec(),
        network.inputs(),
        network.outputs(),
        network.trace().to_vec(),
        network.transitions().to_vec(),
    );

    assert_eq!(
        network
            .step(ratio(-1, 1), ratio(0, 1), BTreeMap::new())
            .unwrap_err(),
        TrophicNetworkError::NegativeAmount
    );
    assert_eq!(
        (
            network.time().clone(),
            network.amounts().to_vec(),
            network.inputs(),
            network.outputs(),
            network.trace().to_vec(),
            network.transitions().to_vec(),
        ),
        before
    );
}

#[test]
fn source_limited_harvest_trace_records_settled_output() {
    let spec = TrophicNetworkSpec::new(
        Vec::new(),
        vec![ConsumerSpec::new("grazer", ratio(0, 1))],
        Vec::new(),
        ratio(0, 1),
    );
    let initial = BTreeMap::from([
        ("nutrient".to_owned(), ratio(0, 1)),
        ("grazer".to_owned(), ratio(1, 1)),
        ("detritus".to_owned(), ratio(0, 1)),
    ]);
    let mut network = ExactTrophicNetwork::new(spec, initial).unwrap();

    let step = network
        .step(
            ratio(1, 1),
            ratio(0, 1),
            BTreeMap::from([("grazer".to_owned(), ratio(100, 1))]),
        )
        .unwrap();

    assert_eq!(step.harvest("grazer"), Some(&ratio(1, 1)));
    let transition = step.transition();
    let harvest = TrophicFlow::Harvest("grazer".to_owned());
    assert_eq!(transition.proposed(&harvest), Some(&ratio(100, 1)));
    assert_eq!(transition.settled(&harvest), Some(&ratio(1, 1)));
    assert_eq!(transition, &network.transitions()[0]);
    assert_eq!(network.outputs(), ratio(1, 1));
    assert_eq!(
        network
            .trace()
            .last()
            .unwrap()
            .value(&AxisId::new("cumulative_output").unwrap()),
        Some(&ratio(1, 1))
    );
    assert!(network.evidence().unwrap().is_satisfied());
}

#[test]
fn ordered_discard_step_matches_reporting_step() {
    let plan = DenseTrophicNetworkPlan::compile(dense_spec(0.8)).unwrap();
    let mut reporting = plan.start(dense_initial()).unwrap();
    let mut discard = plan.start(dense_initial()).unwrap();

    reporting
        .step(0.25, 2.0, BTreeMap::from([("grazer".to_owned(), 0.5)]))
        .unwrap();
    discard.step_discard_ordered(0.25, 2.0, &[0.5]).unwrap();

    assert_eq!(plan.consumer_names(), &["grazer"]);
    assert_eq!(reporting.amounts(), discard.amounts());
    assert_eq!(reporting.inputs(), discard.inputs());
    assert_eq!(reporting.outputs(), discard.outputs());
    assert_eq!(reporting.time(), discard.time());
}

#[test]
fn dense_balance_contract_uses_the_declared_tolerance() {
    let tolerance = trophic_dense_balance_tolerance();

    assert_eq!(tolerance.absolute, TROPHIC_DENSE_BALANCE_ABSOLUTE_TOLERANCE);
    assert_eq!(tolerance.relative, TROPHIC_DENSE_BALANCE_RELATIVE_TOLERANCE);
    assert_eq!(tolerance.absolute, 256.0 * f64::EPSILON);
    assert_eq!(tolerance.relative, 256.0 * f64::EPSILON);
}

#[test]
fn invalid_ordered_harvest_count_is_atomic() {
    let plan = DenseTrophicNetworkPlan::compile(dense_spec(0.8)).unwrap();
    let mut network = plan.start(dense_initial()).unwrap();
    let before = (network.time(), network.amounts().to_vec());

    assert_eq!(
        network.step_discard_ordered(0.25, 2.0, &[]).unwrap_err(),
        TrophicNetworkError::HarvestCount {
            expected: 1,
            actual: 0,
        }
    );
    assert_eq!((network.time(), network.amounts().to_vec()), before);
}

#[test]
fn competing_producers_share_a_scarce_nutrient_without_order_effects() {
    let starved = BTreeMap::from([
        ("nutrient".to_owned(), ratio(1, 1)),
        ("grazed".to_owned(), ratio(100, 1)),
        ("refuge".to_owned(), ratio(100, 1)),
        ("grazer".to_owned(), ratio(0, 1)),
        ("detritus".to_owned(), ratio(0, 1)),
    ]);
    let mut forward = ExactTrophicNetwork::new(exact_spec(ratio(0, 1)), starved.clone()).unwrap();
    let base = exact_spec(ratio(0, 1));
    let mut producers = base.producers().to_vec();
    producers.reverse();
    let reversed_spec = TrophicNetworkSpec::new(
        producers,
        base.consumers().to_vec(),
        base.feedings().to_vec(),
        base.decomposition().clone(),
    );
    let mut reversed = ExactTrophicNetwork::new(reversed_spec, starved).unwrap();

    forward
        .step(ratio(1, 1), ratio(0, 1), BTreeMap::new())
        .unwrap();
    reversed
        .step(ratio(1, 1), ratio(0, 1), BTreeMap::new())
        .unwrap();

    assert_eq!(forward.stock("nutrient"), Some(&ratio(0, 1)));
    for stock in ["nutrient", "grazed", "refuge", "grazer", "detritus"] {
        assert_eq!(forward.stock(stock), reversed.stock(stock));
    }
    let forward_evidence = forward
        .evidence()
        .unwrap()
        .laws()
        .iter()
        .map(|law| (law.law().clone(), law.verdict().clone()))
        .collect::<BTreeMap<_, _>>();
    let reversed_evidence = reversed
        .evidence()
        .unwrap()
        .laws()
        .iter()
        .map(|law| (law.law().clone(), law.verdict().clone()))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(forward_evidence, reversed_evidence);
    assert_eq!(
        forward.law_evidence().unwrap(),
        reversed.law_evidence().unwrap()
    );
}

#[test]
fn source_limitation_preserves_the_assimilation_and_waste_partition() {
    let spec = TrophicNetworkSpec::new(
        vec![ProducerSpec::new(
            "prey",
            ratio(0, 1),
            ratio(1, 1),
            ratio(0, 1),
        )],
        vec![ConsumerSpec::new("predator", ratio(0, 1))],
        vec![FeedingSpec::new(
            "predator",
            "prey",
            ratio(100, 1),
            ratio(1, 1),
            ratio(3, 4),
        )],
        ratio(0, 1),
    );
    let initial = BTreeMap::from([
        ("nutrient".to_owned(), ratio(0, 1)),
        ("prey".to_owned(), ratio(1, 1)),
        ("predator".to_owned(), ratio(10, 1)),
        ("detritus".to_owned(), ratio(0, 1)),
    ]);
    let mut network = ExactTrophicNetwork::new(spec, initial).unwrap();

    let step = network
        .step(ratio(1, 1), ratio(0, 1), BTreeMap::new())
        .unwrap();

    assert_eq!(step.feeding("predator", "prey"), Some(ratio(1, 1)));
    assert_eq!(network.stock("prey"), Some(&ratio(0, 1)));
    assert_eq!(network.stock("predator"), Some(&ratio(43, 4)));
    assert_eq!(network.stock("detritus"), Some(&ratio(1, 4)));
    assert!(network.is_balanced());
}

#[test]
fn overflowing_dense_process_proposal_is_atomic() {
    let spec = TrophicNetworkSpec::new(
        vec![ProducerSpec::new("plant", f64::MAX, 1.0, 0.0)],
        Vec::new(),
        Vec::new(),
        0.0,
    );
    let initial = BTreeMap::from([
        ("nutrient".to_owned(), 1.0),
        ("plant".to_owned(), 3.0),
        ("detritus".to_owned(), 0.0),
    ]);
    let mut network = DenseTrophicNetwork::new(spec, initial).unwrap();
    let before = (
        network.time(),
        network.stock("nutrient"),
        network.stock("plant"),
        network.stock("detritus"),
    );

    assert!(network.step(1.0, 0.0, BTreeMap::new()).is_err());
    assert_eq!(
        (
            network.time(),
            network.stock("nutrient"),
            network.stock("plant"),
            network.stock("detritus"),
        ),
        before
    );
}

#[test]
fn competing_feeding_edges_are_independent_of_declaration_order() {
    let producer = ProducerSpec::new("prey", ratio(0, 1), ratio(1, 1), ratio(0, 1));
    let consumers = vec![
        ConsumerSpec::new("first", ratio(0, 1)),
        ConsumerSpec::new("second", ratio(0, 1)),
    ];
    let feedings = vec![
        FeedingSpec::new("first", "prey", ratio(10, 1), ratio(1, 1), ratio(1, 2)),
        FeedingSpec::new("second", "prey", ratio(10, 1), ratio(1, 1), ratio(1, 2)),
    ];
    let forward_spec = TrophicNetworkSpec::new(
        vec![producer.clone()],
        consumers.clone(),
        feedings.clone(),
        ratio(0, 1),
    );
    let reverse_spec = TrophicNetworkSpec::new(
        vec![producer],
        consumers,
        feedings.into_iter().rev().collect(),
        ratio(0, 1),
    );
    let initial = BTreeMap::from([
        ("nutrient".to_owned(), ratio(0, 1)),
        ("prey".to_owned(), ratio(1, 1)),
        ("first".to_owned(), ratio(10, 1)),
        ("second".to_owned(), ratio(10, 1)),
        ("detritus".to_owned(), ratio(0, 1)),
    ]);
    let forward_plan = ExactTrophicNetworkPlan::compile(forward_spec).unwrap();
    let reverse_plan = ExactTrophicNetworkPlan::compile(reverse_spec).unwrap();

    assert_eq!(forward_plan.law_suite_laws(), reverse_plan.law_suite_laws());
    let mut forward = forward_plan.start(initial.clone()).unwrap();
    let mut reverse = reverse_plan.start(initial).unwrap();

    forward
        .step(ratio(1, 1), ratio(0, 1), BTreeMap::new())
        .unwrap();
    reverse
        .step(ratio(1, 1), ratio(0, 1), BTreeMap::new())
        .unwrap();

    for stock in ["nutrient", "prey", "first", "second", "detritus"] {
        assert_eq!(forward.stock(stock), reverse.stock(stock));
    }
}

#[test]
fn malformed_networks_fail_before_state_exists() {
    let unknown_resource = TrophicNetworkSpec::new(
        vec![ProducerSpec::new("plant", 1.0, 1.0, 0.0)],
        vec![ConsumerSpec::new("grazer", 0.0)],
        vec![FeedingSpec::new("grazer", "missing", 1.0, 1.0, 0.5)],
        0.0,
    );
    let initial = BTreeMap::from([
        ("nutrient".to_owned(), 1.0),
        ("plant".to_owned(), 1.0),
        ("grazer".to_owned(), 1.0),
        ("detritus".to_owned(), 0.0),
    ]);
    assert_eq!(
        DenseTrophicNetwork::new(unknown_resource, initial).unwrap_err(),
        TrophicNetworkError::UnknownResource("missing".to_owned())
    );

    let invalid_efficiency = TrophicNetworkSpec::new(
        vec![ProducerSpec::new("plant", 1.0, 1.0, 0.0)],
        vec![ConsumerSpec::new("grazer", 0.0)],
        vec![FeedingSpec::new("grazer", "plant", 1.0, 1.0, 1.1)],
        0.0,
    );
    let initial = BTreeMap::from([
        ("nutrient".to_owned(), 1.0),
        ("plant".to_owned(), 1.0),
        ("grazer".to_owned(), 1.0),
        ("detritus".to_owned(), 0.0),
    ]);
    assert_eq!(
        DenseTrophicNetwork::new(invalid_efficiency, initial).unwrap_err(),
        TrophicNetworkError::InvalidAssimilationEfficiency
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn arbitrary_forcing_preserves_dense_nonnegativity_and_open_balance(
        nutrient in 0_u16..1_000,
        grazed in 0_u16..1_000,
        refuge in 0_u16..1_000,
        grazer in 0_u16..1_000,
        detritus in 0_u16..1_000,
        steps in prop::collection::vec((0_u8..5, 0_u16..100, 0_u16..100), 1..8),
    ) {
        let initial = BTreeMap::from([
            ("nutrient".to_owned(), f64::from(nutrient)),
            ("grazed".to_owned(), f64::from(grazed)),
            ("refuge".to_owned(), f64::from(refuge)),
            ("grazer".to_owned(), f64::from(grazer)),
            ("detritus".to_owned(), f64::from(detritus)),
        ]);
        let mut network = DenseTrophicNetwork::new(dense_spec(0.8), initial).unwrap();
        for (quarters, input, harvest) in steps {
            network.step(
                f64::from(quarters) / 4.0,
                f64::from(input),
                BTreeMap::from([("grazer".to_owned(), f64::from(harvest))]),
            ).unwrap();
            for stock in ["nutrient", "grazed", "refuge", "grazer", "detritus"] {
                prop_assert!(network.stock(stock).unwrap().is_finite());
                prop_assert!(network.stock(stock).unwrap() >= 0.0);
            }
            prop_assert!(network.is_balanced());
        }
    }

    #[test]
    fn arbitrary_exact_runs_satisfy_every_compiled_sentence(
        nutrient in 0_u16..1_000,
        grazed in 0_u16..1_000,
        refuge in 0_u16..1_000,
        grazer in 0_u16..1_000,
        detritus in 0_u16..1_000,
        steps in prop::collection::vec((0_u8..5, 0_u16..100, 0_u16..100), 1..8),
    ) {
        let initial = BTreeMap::from([
            ("nutrient".to_owned(), ratio(i64::from(nutrient), 1)),
            ("grazed".to_owned(), ratio(i64::from(grazed), 1)),
            ("refuge".to_owned(), ratio(i64::from(refuge), 1)),
            ("grazer".to_owned(), ratio(i64::from(grazer), 1)),
            ("detritus".to_owned(), ratio(i64::from(detritus), 1)),
        ]);
        let mut network = ExactTrophicNetwork::new(exact_spec(ratio(4, 5)), initial).unwrap();
        let successful_steps = steps.len();
        for (quarters, input, harvest) in steps {
            network.step(
                ratio(i64::from(quarters), 4),
                ratio(i64::from(input), 1),
                BTreeMap::from([("grazer".to_owned(), ratio(i64::from(harvest), 1))]),
            ).unwrap();
        }

        let evidence = network.evidence().unwrap();
        let law_evidence = network.law_evidence().unwrap();
        prop_assert_eq!(network.trace().len(), successful_steps + 1);
        prop_assert_eq!(network.transitions().len(), successful_steps);
        prop_assert!(evidence.is_satisfied());
        prop_assert!(evidence.laws().iter().all(TrophicLawEvidence::is_satisfied));
        prop_assert!(law_evidence.is_satisfied());
    }

    #[test]
    fn arbitrary_rejected_exact_steps_preserve_trace_and_state(
        nutrient in 0_u16..1_000,
        grazed in 0_u16..1_000,
        refuge in 0_u16..1_000,
        grazer in 0_u16..1_000,
        detritus in 0_u16..1_000,
        negative in 1_u16..1_000,
    ) {
        let initial = BTreeMap::from([
            ("nutrient".to_owned(), ratio(i64::from(nutrient), 1)),
            ("grazed".to_owned(), ratio(i64::from(grazed), 1)),
            ("refuge".to_owned(), ratio(i64::from(refuge), 1)),
            ("grazer".to_owned(), ratio(i64::from(grazer), 1)),
            ("detritus".to_owned(), ratio(i64::from(detritus), 1)),
        ]);
        let mut network = ExactTrophicNetwork::new(exact_spec(ratio(4, 5)), initial).unwrap();
        let before = (
            network.time().clone(),
            network.amounts().to_vec(),
            network.inputs(),
            network.outputs(),
            network.trace().to_vec(),
            network.transitions().to_vec(),
        );

        prop_assert!(matches!(
            network.step(
                ratio(-i64::from(negative), 1),
                ratio(0, 1),
                BTreeMap::new(),
            ),
            Err(TrophicNetworkError::NegativeAmount),
        ));
        prop_assert_eq!(
            (
                network.time().clone(),
                network.amounts().to_vec(),
                network.inputs(),
                network.outputs(),
                network.trace().to_vec(),
                network.transitions().to_vec(),
            ),
            before,
        );
    }
}
