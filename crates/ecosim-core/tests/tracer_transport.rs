use std::collections::BTreeMap;

use ecosim_core::{
    ConsumerSpec, DenseTrophicNetworkPlan, ExactTrophicNetworkPlan, FeedingSpec, ProducerSpec,
    TrophicFlow, TrophicLaw, TrophicNetworkError, TrophicNetworkSpec,
};
use num_rational::BigRational;
use num_traits::Zero;

fn ratio(numerator: i64, denominator: i64) -> BigRational {
    BigRational::new(numerator.into(), denominator.into())
}

fn exact_spec(tracer_kinds: Vec<String>) -> TrophicNetworkSpec<BigRational> {
    TrophicNetworkSpec::with_tracer_kinds(
        vec![ProducerSpec::new(
            "algae",
            ratio(1, 2),
            ratio(10, 1),
            ratio(1, 100),
        )],
        vec![ConsumerSpec::new("grazer", ratio(1, 50))],
        vec![FeedingSpec::new(
            "grazer",
            "algae",
            ratio(4, 5),
            ratio(5, 1),
            ratio(3, 4),
        )],
        ratio(1, 10),
        tracer_kinds,
    )
}

fn exact_initial() -> BTreeMap<String, BigRational> {
    BTreeMap::from([
        ("nutrient".to_owned(), ratio(100, 1)),
        ("algae".to_owned(), ratio(10, 1)),
        ("grazer".to_owned(), ratio(5, 1)),
        ("detritus".to_owned(), ratio(4, 1)),
    ])
}

fn nitrogen_initial() -> BTreeMap<String, BigRational> {
    BTreeMap::from([
        ("nutrient".to_owned(), ratio(7, 1)),
        ("algae".to_owned(), ratio(3, 2)),
        ("grazer".to_owned(), ratio(1, 3)),
        ("detritus".to_owned(), ratio(2, 5)),
    ])
}

/// The material source stock of one semantic flow, if it has one.
fn flow_source(flow: &TrophicFlow) -> Option<&str> {
    match flow {
        TrophicFlow::NutrientInput => None,
        TrophicFlow::ProducerGrowth(_) => Some("nutrient"),
        TrophicFlow::FeedingAssimilation { resource, .. }
        | TrophicFlow::FeedingWaste { resource, .. } => Some(resource),
        TrophicFlow::Mortality(stock) => Some(stock),
        TrophicFlow::Decomposition => Some("detritus"),
        TrophicFlow::Harvest(consumer) => Some(consumer),
    }
}

#[test]
fn empty_tracer_declaration_compiles_the_single_kind_network_unchanged() {
    let bare = ExactTrophicNetworkPlan::compile(TrophicNetworkSpec::new(
        exact_spec(Vec::new()).producers().to_vec(),
        exact_spec(Vec::new()).consumers().to_vec(),
        exact_spec(Vec::new()).feedings().to_vec(),
        ratio(1, 10),
    ))
    .unwrap();
    let declared = ExactTrophicNetworkPlan::compile(exact_spec(Vec::new())).unwrap();

    assert_eq!(
        bare.evidence_axis_names().collect::<Vec<_>>(),
        declared.evidence_axis_names().collect::<Vec<_>>()
    );
    assert_eq!(bare.law_suite_laws(), declared.law_suite_laws());
    assert_eq!(bare.stock_names(), declared.stock_names());
    assert_eq!(bare.flow_symbols(), declared.flow_symbols());
    assert!(declared.tracer_kinds().is_empty());

    let mut bare_run = bare.start(exact_initial()).unwrap();
    let mut declared_run = declared.start(exact_initial()).unwrap();
    let bare_step = bare_run
        .step(ratio(1, 1), ratio(2, 1), BTreeMap::new())
        .unwrap();
    let declared_step = declared_run
        .step(ratio(1, 1), ratio(2, 1), BTreeMap::new())
        .unwrap();
    assert_eq!(bare_step.transition(), declared_step.transition());
}

#[test]
fn tracer_flows_follow_settled_material_in_exact_source_proportion() {
    let plan = ExactTrophicNetworkPlan::compile(exact_spec(vec!["nitrogen".to_owned()])).unwrap();
    let mut network = plan
        .start_with_tracers(
            exact_initial(),
            BTreeMap::from([("nitrogen".to_owned(), nitrogen_initial())]),
        )
        .unwrap();

    for step in 0..5 {
        let tracer_input = if step % 2 == 0 {
            ratio(1, 4)
        } else {
            ratio(0, 1)
        };
        network
            .step_with_tracer_inputs(
                ratio(1, 2),
                ratio(3, 1),
                BTreeMap::from([("nitrogen".to_owned(), tracer_input)]),
                BTreeMap::from([("grazer".to_owned(), ratio(1, 8))]),
            )
            .unwrap();
    }

    let stock_names = plan.stock_names().to_vec();
    for transition in network.transitions() {
        let material_before = transition.stocks_before().to_vec();
        let nitrogen_before = transition
            .tracer_stocks_before("nitrogen")
            .unwrap()
            .to_vec();
        for flow in transition.flow_symbols().to_vec() {
            let Some(source) = flow_source(&flow) else {
                continue;
            };
            let index = stock_names
                .iter()
                .position(|name| name == source)
                .expect("semantic sources are compiled stocks");
            let material = transition.settled(&flow).unwrap().clone();
            let tracer = transition
                .tracer_settled("nitrogen", &flow)
                .unwrap()
                .clone();
            if material_before[index].is_zero() {
                assert!(tracer.is_zero());
            } else {
                assert_eq!(
                    tracer,
                    &material * &nitrogen_before[index] / &material_before[index],
                    "flow {flow:?} broke tracer proportionality"
                );
            }
        }
    }
}

#[test]
fn rationed_settlement_keeps_tracer_proportionality_and_satisfies_every_law() {
    let plan = ExactTrophicNetworkPlan::compile(exact_spec(vec!["nitrogen".to_owned()])).unwrap();
    let mut network = plan
        .start_with_tracers(
            exact_initial(),
            BTreeMap::from([("nitrogen".to_owned(), nitrogen_initial())]),
        )
        .unwrap();

    // Demand far more harvest than the grazer holds so the kernel rations
    // every outflow of the grazer stock below its proposal.
    let step = network
        .step_with_tracer_inputs(
            ratio(1, 1),
            ratio(0, 1),
            BTreeMap::from([("nitrogen".to_owned(), ratio(1, 2))]),
            BTreeMap::from([("grazer".to_owned(), ratio(1000, 1))]),
        )
        .unwrap();
    let transition = step.transition();
    let harvest = TrophicFlow::Harvest("grazer".to_owned());
    assert!(transition.settled(&harvest).unwrap() < transition.proposed(&harvest).unwrap());

    let grazer = plan
        .stock_names()
        .iter()
        .position(|name| name == "grazer")
        .unwrap();
    let material = transition.settled(&harvest).unwrap();
    let tracer = transition.tracer_settled("nitrogen", &harvest).unwrap();
    assert_eq!(
        tracer.clone(),
        material * &transition.tracer_stocks_before("nitrogen").unwrap()[grazer]
            / &transition.stocks_before()[grazer]
    );

    let suite = network.law_evidence().unwrap();
    assert!(suite.is_satisfied());
    assert!(network.balance_residual().is_zero());
    assert_eq!(
        network.tracer_balance_residual("nitrogen").unwrap(),
        ratio(0, 1)
    );
}

#[test]
fn tracer_books_close_and_the_law_suite_names_tracer_sentences() {
    let plan = ExactTrophicNetworkPlan::compile(exact_spec(vec!["nitrogen".to_owned()])).unwrap();
    let mut network = plan
        .start_with_tracers(
            exact_initial(),
            BTreeMap::from([("nitrogen".to_owned(), nitrogen_initial())]),
        )
        .unwrap();
    network
        .step_with_tracer_inputs(
            ratio(1, 1),
            ratio(2, 1),
            BTreeMap::from([("nitrogen".to_owned(), ratio(3, 7))]),
            BTreeMap::from([("grazer".to_owned(), ratio(1, 2))]),
        )
        .unwrap();

    assert_eq!(network.tracer_inputs("nitrogen").unwrap(), ratio(3, 7));
    assert_eq!(
        network.tracer_balance_residual("nitrogen").unwrap(),
        ratio(0, 1)
    );

    let laws = plan.law_suite_laws();
    for expected in [
        TrophicLaw::Tracer {
            kind: "nitrogen".to_owned(),
            law: Box::new(TrophicLaw::MaterialInvariant),
        },
        TrophicLaw::Tracer {
            kind: "nitrogen".to_owned(),
            law: Box::new(TrophicLaw::OpenMaterialBalance),
        },
        TrophicLaw::Tracer {
            kind: "nitrogen".to_owned(),
            law: Box::new(TrophicLaw::NutrientInputCorrespondence),
        },
        TrophicLaw::Tracer {
            kind: "nitrogen".to_owned(),
            law: Box::new(TrophicLaw::HarvestCorrespondence("grazer".to_owned())),
        },
        TrophicLaw::Tracer {
            kind: "nitrogen".to_owned(),
            law: Box::new(TrophicLaw::FeedingPartition {
                consumer: "grazer".to_owned(),
                resource: "algae".to_owned(),
            }),
        },
    ] {
        assert!(laws.contains(&expected), "law suite lacks {expected:?}");
    }
    assert!(network.law_evidence().unwrap().is_satisfied());
    assert!(network.evidence().unwrap().is_satisfied());
}

#[test]
fn evidence_axes_carry_one_block_per_kind() {
    let plan = ExactTrophicNetworkPlan::compile(exact_spec(vec![
        "carbon".to_owned(),
        "nitrogen".to_owned(),
    ]))
    .unwrap();
    assert_eq!(
        plan.evidence_axis_names().collect::<Vec<_>>(),
        vec![
            "nutrient",
            "algae",
            "grazer",
            "detritus",
            "nutrient:carbon",
            "algae:carbon",
            "grazer:carbon",
            "detritus:carbon",
            "nutrient:nitrogen",
            "algae:nitrogen",
            "grazer:nitrogen",
            "detritus:nitrogen",
            "cumulative_input",
            "cumulative_output",
            "cumulative_input:carbon",
            "cumulative_output:carbon",
            "cumulative_input:nitrogen",
            "cumulative_output:nitrogen",
        ]
    );
    assert_eq!(plan.tracer_kinds(), ["carbon", "nitrogen"]);
}

#[test]
fn a_tracer_stranded_in_an_empty_material_stock_does_not_move() {
    let plan = ExactTrophicNetworkPlan::compile(exact_spec(vec!["nitrogen".to_owned()])).unwrap();
    let mut initial = exact_initial();
    initial.insert("detritus".to_owned(), ratio(0, 1));
    let mut network = plan
        .start_with_tracers(
            initial,
            BTreeMap::from([(
                "nitrogen".to_owned(),
                BTreeMap::from([
                    ("nutrient".to_owned(), ratio(0, 1)),
                    ("algae".to_owned(), ratio(0, 1)),
                    ("grazer".to_owned(), ratio(0, 1)),
                    ("detritus".to_owned(), ratio(5, 1)),
                ]),
            )]),
        )
        .unwrap();

    let step = network
        .step(ratio(1, 1), ratio(0, 1), BTreeMap::new())
        .unwrap();
    assert!(
        step.transition()
            .tracer_settled("nitrogen", &TrophicFlow::Decomposition)
            .unwrap()
            .is_zero()
    );
    assert_eq!(
        network.tracer_stock("nitrogen", "detritus").unwrap(),
        &ratio(5, 1)
    );
    assert!(network.law_evidence().unwrap().is_satisfied());
}

#[test]
fn invalid_tracer_declarations_are_rejected() {
    for kind in ["", "material-equivalent", "with:separator"] {
        assert!(matches!(
            ExactTrophicNetworkPlan::compile(exact_spec(vec![kind.to_owned()])),
            Err(TrophicNetworkError::InvalidKindName(name)) if name == kind
        ));
    }
    assert!(matches!(
        ExactTrophicNetworkPlan::compile(exact_spec(vec![
            "nitrogen".to_owned(),
            "nitrogen".to_owned()
        ])),
        Err(TrophicNetworkError::DuplicateKind(name)) if name == "nitrogen"
    ));
    let colliding = TrophicNetworkSpec::with_tracer_kinds(
        vec![ProducerSpec::new(
            "algae:nitrogen",
            ratio(1, 2),
            ratio(10, 1),
            ratio(0, 1),
        )],
        Vec::new(),
        Vec::new(),
        ratio(0, 1),
        vec!["nitrogen".to_owned()],
    );
    assert!(matches!(
        ExactTrophicNetworkPlan::compile(colliding),
        Err(TrophicNetworkError::InvalidStockName(name)) if name == "algae:nitrogen"
    ));
}

#[test]
fn undeclared_or_incomplete_tracer_state_is_rejected() {
    let plan = ExactTrophicNetworkPlan::compile(exact_spec(vec!["nitrogen".to_owned()])).unwrap();
    assert!(matches!(
        plan.start_with_tracers(
            exact_initial(),
            BTreeMap::from([("phosphorus".to_owned(), nitrogen_initial())]),
        ),
        Err(TrophicNetworkError::UnknownKind(kind)) if kind == "phosphorus"
    ));
    let mut incomplete = nitrogen_initial();
    incomplete.remove("detritus");
    assert!(matches!(
        plan.start_with_tracers(
            exact_initial(),
            BTreeMap::from([("nitrogen".to_owned(), incomplete)]),
        ),
        Err(TrophicNetworkError::MissingInitialStock(name)) if name == "detritus:nitrogen"
    ));
    let mut network = plan.start(exact_initial()).unwrap();
    assert!(matches!(
        network.step_with_tracer_inputs(
            ratio(1, 1),
            ratio(0, 1),
            BTreeMap::from([("phosphorus".to_owned(), ratio(1, 1))]),
            BTreeMap::new(),
        ),
        Err(TrophicNetworkError::UnknownKind(kind)) if kind == "phosphorus"
    ));
}

#[test]
fn dense_tracer_transport_closes_within_kernel_tolerance() {
    let plan = DenseTrophicNetworkPlan::compile(TrophicNetworkSpec::with_tracer_kinds(
        vec![ProducerSpec::new("algae", 0.5, 10.0, 0.01)],
        vec![ConsumerSpec::new("grazer", 0.02)],
        vec![FeedingSpec::new("grazer", "algae", 0.8, 5.0, 0.75)],
        0.1,
        vec!["nitrogen".to_owned()],
    ))
    .unwrap();
    let mut network = plan
        .start_with_tracers(
            BTreeMap::from([
                ("nutrient".to_owned(), 100.0),
                ("algae".to_owned(), 10.0),
                ("grazer".to_owned(), 5.0),
                ("detritus".to_owned(), 4.0),
            ]),
            BTreeMap::from([(
                "nitrogen".to_owned(),
                BTreeMap::from([
                    ("nutrient".to_owned(), 7.0),
                    ("algae".to_owned(), 1.5),
                    ("grazer".to_owned(), 0.4),
                    ("detritus".to_owned(), 0.3),
                ]),
            )]),
        )
        .unwrap();

    for _ in 0..50 {
        network
            .step_with_tracer_inputs(
                0.5,
                3.0,
                BTreeMap::from([("nitrogen".to_owned(), 0.25)]),
                BTreeMap::from([("grazer".to_owned(), 0.125)]),
            )
            .unwrap();
    }
    assert!(network.is_balanced());
    let residual = network.tracer_balance_residual("nitrogen").unwrap();
    assert!(
        residual.abs() <= 1e-9,
        "dense nitrogen residual {residual} exceeded tolerance"
    );
    assert!(network.tracer_stock("nitrogen", "grazer").unwrap() > 0.0);
    assert_eq!(network.tracer_inputs("nitrogen").unwrap(), 0.25 * 50.0);
}
