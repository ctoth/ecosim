use std::collections::BTreeMap;

use ecosim_core::{
    ConsumerSpec, DenseTrophicNetwork, DenseTrophicNetworkPlan, ExactTrophicNetwork,
    ExactTrophicNetworkPlan, FeedingSpec, ProducerSpec, TrophicNetworkError, TrophicNetworkSpec,
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
    assert_ne!(perturbed.stock("nutrient"), untouched.stock("nutrient"));
    assert!(perturbed.is_balanced());
    assert!(untouched.is_balanced());
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
    let mut forward = ExactTrophicNetwork::new(forward_spec, initial.clone()).unwrap();
    let mut reverse = ExactTrophicNetwork::new(reverse_spec, initial).unwrap();

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
}
