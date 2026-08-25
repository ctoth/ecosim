use ecosim_core::{
    DenseFoodWeb, DenseFoodWebParameters, FoodWeb, FoodWebError, FoodWebParameters, integer_amount,
};
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{Signed, ToPrimitive, Zero};
use proptest::prelude::*;

fn integer(value: i64) -> BigRational {
    integer_amount(BigInt::from(value))
}

#[test]
fn one_step_exhibits_growth_grazing_mortality_and_exact_accounting() {
    let mut web =
        FoodWeb::with_defaults(integer(100), integer(20), integer(5), integer(0)).unwrap();
    let step = web.step(integer(1), integer(0), integer(0)).unwrap();

    assert!(step.applied("producer-growth").is_positive());
    assert!(step.applied("grazing").is_positive());
    assert!(step.applied("producer-mortality").is_positive());
    assert!(step.applied("consumer-mortality").is_positive());
    assert!(web.stock("nutrient").unwrap() < &integer(100));
    assert!(web.stock("producer").unwrap() > &integer(20));
    assert!(web.stock("consumer").unwrap() > &integer(5));
    assert!(web.stock("detritus").unwrap().is_positive());
    assert!(web.is_balanced());
}

#[test]
fn detritus_recycles_to_nutrient() {
    let mut web = FoodWeb::with_defaults(integer(0), integer(0), integer(0), integer(10)).unwrap();
    let step = web.step(integer(1), integer(0), integer(0)).unwrap();
    assert_eq!(step.applied("decomposition"), integer(1));
    assert_eq!(web.stock("nutrient"), Some(&integer(1)));
    assert_eq!(web.stock("detritus"), Some(&integer(9)));
    assert!(web.is_balanced());
}

#[test]
fn absent_resources_do_not_create_producers_or_consumers() {
    let mut web = FoodWeb::with_defaults(integer(0), integer(0), integer(0), integer(0)).unwrap();
    let step = web.step(integer(10), integer(0), integer(0)).unwrap();
    assert!(step.applied("producer-growth").is_zero());
    assert!(step.applied("grazing").is_zero());
    assert_eq!(web.stock("producer"), Some(&integer(0)));
    assert_eq!(web.stock("consumer"), Some(&integer(0)));
}

#[test]
fn invalid_parameters_and_steps_are_rejected_atomically() {
    let invalid = FoodWebParameters::new(
        integer(1),
        integer(0),
        integer(1),
        integer(1),
        integer(0),
        integer(0),
        integer(0),
    );
    assert_eq!(invalid, Err(FoodWebError::NonpositiveHalfSaturation));

    let mut web = FoodWeb::with_defaults(integer(10), integer(2), integer(1), integer(0)).unwrap();
    let before = web.clone();
    assert_eq!(
        web.step(integer(-1), integer(0), integer(0)),
        Err(FoodWebError::NegativeAmount)
    );
    assert_eq!(web, before);
}

#[test]
fn dense_and_exact_default_models_agree_for_one_step() {
    let mut exact =
        FoodWeb::with_defaults(integer(100), integer(20), integer(5), integer(3)).unwrap();
    let mut dense = DenseFoodWeb::with_defaults(100.0, 20.0, 5.0, 3.0).unwrap();

    let exact_step = exact.step(integer(1), integer(2), integer(1)).unwrap();
    let dense_step = dense.step(1.0, 2.0, 1.0).unwrap();

    for stock in ["nutrient", "producer", "consumer", "detritus"] {
        let exact_amount = exact.stock(stock).unwrap().to_f64().unwrap();
        assert!((dense.stock(stock).unwrap() - exact_amount).abs() < 1e-12);
    }
    for process in [
        "nutrient-input",
        "producer-growth",
        "grazing",
        "producer-mortality",
        "consumer-mortality",
        "decomposition",
        "harvest",
    ] {
        let exact_amount = exact_step.applied(process).to_f64().unwrap();
        assert!((dense_step.applied(process) - exact_amount).abs() < 1e-12);
    }
    assert!(dense.is_balanced());
}

#[test]
fn dense_model_supports_long_finite_nonnegative_trajectories() {
    let mut web = DenseFoodWeb::with_defaults(100.0, 20.0, 5.0, 0.0).unwrap();
    for _ in 0..10_000 {
        web.step(0.01, 0.002, 0.001).unwrap();
        assert!(
            web.amounts()
                .iter()
                .all(|amount| amount.is_finite() && *amount >= 0.0)
        );
        assert!(web.is_balanced());
    }
    assert!((web.time() - 100.0).abs() < 2e-11);
}

#[test]
fn invalid_dense_construction_and_steps_are_atomic() {
    assert!(DenseFoodWebParameters::new(0.5, 0.0, 0.4, 10.0, 0.05, 0.04, 0.1).is_err());
    assert!(DenseFoodWeb::with_defaults(f64::NAN, 1.0, 0.0, 0.0).is_err());

    let mut web = DenseFoodWeb::with_defaults(10.0, 2.0, 1.0, 0.0).unwrap();
    let before = web.clone();
    assert!(web.step(f64::INFINITY, 0.0, 0.0).is_err());
    assert_eq!(web, before);
    assert!(web.step(1.0, -1.0, 0.0).is_err());
    assert_eq!(web, before);

    let explosive = DenseFoodWebParameters::new(f64::MAX, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0).unwrap();
    let mut web = DenseFoodWeb::new(1.0, f64::MAX, 0.0, 0.0, explosive).unwrap();
    let before = web.clone();
    assert!(web.step(1.0, 0.0, 0.0).is_err());
    assert_eq!(web, before);
}

#[test]
fn dense_saturation_matches_exact_at_extreme_finite_magnitudes() {
    let maximum = BigRational::from_float(f64::MAX).unwrap();
    let exact_parameters = FoodWebParameters::new(
        maximum.clone(),
        maximum.clone(),
        integer(0),
        integer(1),
        integer(0),
        integer(0),
        integer(0),
    )
    .unwrap();
    let dense_parameters =
        DenseFoodWebParameters::new(f64::MAX, f64::MAX, 0.0, 1.0, 0.0, 0.0, 0.0).unwrap();

    let mut exact = FoodWeb::new(
        maximum.clone(),
        integer(1),
        integer(0),
        integer(0),
        exact_parameters,
    )
    .unwrap();
    let mut dense = DenseFoodWeb::new(f64::MAX, 1.0, 0.0, 0.0, dense_parameters).unwrap();

    let exact_growth = exact
        .step(integer(1), integer(0), integer(0))
        .unwrap()
        .applied("producer-growth")
        .to_f64()
        .unwrap();
    let dense_growth = dense
        .step(1.0, 0.0, 0.0)
        .unwrap()
        .applied("producer-growth");
    assert!(dense_growth.is_finite());
    assert_eq!(dense_growth, exact_growth);
    assert_eq!(dense_growth, f64::MAX / 2.0);
}

#[test]
fn zero_actor_prevents_extreme_dense_rates_from_creating_nan() {
    let maximum = BigRational::from_float(f64::MAX).unwrap();
    let exact_parameters = FoodWebParameters::new(
        maximum,
        integer(1),
        integer(0),
        integer(1),
        integer(0),
        integer(0),
        integer(0),
    )
    .unwrap();
    let dense_parameters =
        DenseFoodWebParameters::new(f64::MAX, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0).unwrap();
    let mut exact = FoodWeb::new(
        BigRational::from_float(f64::MAX).unwrap(),
        integer(0),
        integer(0),
        integer(0),
        exact_parameters,
    )
    .unwrap();
    let mut dense = DenseFoodWeb::new(f64::MAX, 0.0, 0.0, 0.0, dense_parameters).unwrap();

    assert_eq!(
        exact
            .step(integer(1), integer(0), integer(0))
            .unwrap()
            .applied("producer-growth"),
        integer(0)
    );
    assert_eq!(
        dense
            .step(1.0, 0.0, 0.0)
            .unwrap()
            .applied("producer-growth"),
        0.0
    );
    assert!(dense.amounts().iter().all(|value| value.is_finite()));
}

#[test]
fn dense_discard_path_is_state_equivalent_and_atomic() {
    let mut reporting = DenseFoodWeb::with_defaults(100.0, 20.0, 5.0, 3.0).unwrap();
    let mut discard = reporting.clone();
    for step in 0..500 {
        let nutrient_input = f64::from(step % 7) * 0.001;
        let harvest = f64::from(step % 3) * 0.0005;
        reporting.step(0.01, nutrient_input, harvest).unwrap();
        discard.step_discard(0.01, nutrient_input, harvest).unwrap();
        assert_eq!(discard.amounts(), reporting.amounts());
        assert_eq!(discard.inputs(), reporting.inputs());
        assert_eq!(discard.outputs(), reporting.outputs());
        assert_eq!(discard.time(), reporting.time());
    }

    let before = discard.clone();
    assert!(discard.step_discard(f64::NAN, 0.0, 0.0).is_err());
    assert_eq!(discard, before);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn arbitrary_forcing_preserves_nonnegative_stocks_and_exact_open_balance(
        initial in prop::array::uniform4(0_i64..10_000),
        steps in prop::collection::vec((0_i64..4, 0_i64..10_000, 0_i64..10_000), 1..7),
    ) {
        let mut web = FoodWeb::with_defaults(
            integer(initial[0]), integer(initial[1]), integer(initial[2]), integer(initial[3]),
        ).unwrap();
        for (elapsed, input, harvest) in steps {
            web.step(integer(elapsed), integer(input), integer(harvest)).unwrap();
            for stock in ["nutrient", "producer", "consumer", "detritus"] {
                prop_assert!(!web.stock(stock).unwrap().is_negative());
            }
            prop_assert!(web.balance_residual().is_zero());
        }
    }
}
