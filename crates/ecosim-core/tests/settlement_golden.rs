//! Settlement results recorded before the conservation f10d989 migration.
//!
//! `tests/golden/settlement.txt` was written by this file at the old pins
//! (conservation 50f0364) and is checked, unedited, by every later build:
//! exact values by string equality, dense values within
//! `DenseTolerance::default()`. Each scenario ends with a source-limited step,
//! so rationed settlement is part of the record.
//!
//! Regenerate only on purpose: `ECOSIM_WRITE_GOLDEN=1 cargo test -p ecosim-core
//! --test settlement_golden`.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use conservation_dynamics::DenseTolerance;
use ecosim_core::{
    ConsumerSpec, DenseFoodWeb, DenseTrophicNetwork, DenseTrophicNetworkPlan, ExactTrophicNetwork,
    ExactTrophicNetworkPlan, FeedingSpec, FoodWeb, ProducerSpec, TrophicNetworkSpec,
};
use num_rational::BigRational;

const GOLDEN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden/settlement.txt");

const FOOD_WEB_STOCKS: [&str; 4] = ["nutrient", "producer", "consumer", "detritus"];
const FOOD_WEB_PROCESSES: [&str; 7] = [
    "nutrient-input",
    "producer-growth",
    "grazing",
    "producer-mortality",
    "consumer-mortality",
    "decomposition",
    "harvest",
];
const NITROGEN: &str = "nitrogen";

fn ratio(numerator: i64, denominator: i64) -> BigRational {
    BigRational::new(numerator.into(), denominator.into())
}

fn exact(out: &mut String, scenario: &str, step: usize, field: &str, value: &BigRational) {
    writeln!(out, "{scenario} {step} {field} exact {value}").unwrap();
}

fn dense(out: &mut String, scenario: &str, step: usize, field: &str, value: f64) {
    writeln!(
        out,
        "{scenario} {step} {field} dense {:016x}",
        value.to_bits()
    )
    .unwrap();
}

fn food_web_exact(out: &mut String) {
    const SCENARIO: &str = "food_web_exact";
    let mut web =
        FoodWeb::with_defaults(ratio(100, 1), ratio(20, 1), ratio(5, 1), ratio(0, 1)).unwrap();
    let record_state = |out: &mut String, web: &FoodWeb, step: usize| {
        for name in FOOD_WEB_STOCKS {
            exact(
                out,
                SCENARIO,
                step,
                &format!("stock:{name}"),
                web.stock(name).unwrap(),
            );
        }
        exact(out, SCENARIO, step, "inputs", &web.inputs());
        exact(out, SCENARIO, step, "outputs", &web.outputs());
        exact(
            out,
            SCENARIO,
            step,
            "balance_residual",
            &web.balance_residual(),
        );
    };
    record_state(out, &web, 0);
    let forcings = [
        (ratio(1, 1), ratio(0, 1)),
        (ratio(1, 1), ratio(0, 1)),
        (ratio(1, 1), ratio(0, 1)),
        (ratio(0, 1), ratio(1000, 1)),
    ];
    for (index, (nutrient_input, harvest)) in forcings.into_iter().enumerate() {
        let step = index + 1;
        let settled = web.step(ratio(1, 4), nutrient_input, harvest).unwrap();
        for process in FOOD_WEB_PROCESSES {
            exact(
                out,
                SCENARIO,
                step,
                &format!("applied:{process}"),
                &settled.applied(process),
            );
        }
        record_state(out, &web, step);
    }
}

fn food_web_dense(out: &mut String) {
    const SCENARIO: &str = "food_web_dense";
    let mut web = DenseFoodWeb::with_defaults(100.0, 20.0, 5.0, 0.0).unwrap();
    let record_state = |out: &mut String, web: &DenseFoodWeb, step: usize| {
        for name in FOOD_WEB_STOCKS {
            dense(
                out,
                SCENARIO,
                step,
                &format!("stock:{name}"),
                web.stock(name).unwrap(),
            );
        }
        dense(out, SCENARIO, step, "inputs", web.inputs());
        dense(out, SCENARIO, step, "outputs", web.outputs());
        dense(
            out,
            SCENARIO,
            step,
            "balance_residual",
            web.balance_residual(),
        );
    };
    record_state(out, &web, 0);
    let forcings = std::iter::repeat_n((1.0, 0.0), 40).chain([(0.0, 1000.0)]);
    for (index, (nutrient_input, harvest)) in forcings.enumerate() {
        let step = index + 1;
        let settled = web.step(0.25, nutrient_input, harvest).unwrap();
        for process in FOOD_WEB_PROCESSES {
            dense(
                out,
                SCENARIO,
                step,
                &format!("applied:{process}"),
                settled.applied(process),
            );
        }
        record_state(out, &web, step);
    }
}

fn exact_spec() -> TrophicNetworkSpec<BigRational> {
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
        vec![NITROGEN.to_owned()],
    )
}

fn dense_spec() -> TrophicNetworkSpec<f64> {
    TrophicNetworkSpec::with_tracer_kinds(
        vec![ProducerSpec::new("algae", 0.5, 10.0, 0.01)],
        vec![ConsumerSpec::new("grazer", 0.02)],
        vec![FeedingSpec::new("grazer", "algae", 0.8, 5.0, 0.75)],
        0.1,
        vec![NITROGEN.to_owned()],
    )
}

/// A stock name with its initial material and nitrogen ratios.
type InitialStock = (&'static str, (i64, i64), (i64, i64));

/// Initial material and nitrogen stocks, as in tests/tracer_transport.rs.
fn trophic_initial() -> [InitialStock; 4] {
    [
        ("nutrient", (100, 1), (7, 1)),
        ("algae", (10, 1), (3, 2)),
        ("grazer", (5, 1), (1, 3)),
        ("detritus", (4, 1), (2, 5)),
    ]
}

fn exact_trophic_state(out: &mut String, scenario: &str, step: usize, net: &ExactTrophicNetwork) {
    for (name, amount) in net.stock_names().iter().zip(net.amounts()) {
        exact(out, scenario, step, &format!("stock:{name}"), amount);
    }
    let tracer = net.tracer_amounts(NITROGEN).unwrap();
    for (name, amount) in net.stock_names().iter().zip(tracer) {
        exact(
            out,
            scenario,
            step,
            &format!("tracer:{NITROGEN}:{name}"),
            amount,
        );
    }
    exact(out, scenario, step, "inputs", &net.inputs());
    exact(out, scenario, step, "outputs", &net.outputs());
    exact(
        out,
        scenario,
        step,
        "balance_residual",
        &net.balance_residual(),
    );
    let field = |name: &str| format!("{name}:{NITROGEN}");
    exact(
        out,
        scenario,
        step,
        &field("tracer_inputs"),
        &net.tracer_inputs(NITROGEN).unwrap(),
    );
    exact(
        out,
        scenario,
        step,
        &field("tracer_outputs"),
        &net.tracer_outputs(NITROGEN).unwrap(),
    );
    exact(
        out,
        scenario,
        step,
        &field("tracer_balance_residual"),
        &net.tracer_balance_residual(NITROGEN).unwrap(),
    );
}

fn trophic_exact_nitrogen(out: &mut String) {
    const SCENARIO: &str = "trophic_exact_nitrogen";
    let plan = ExactTrophicNetworkPlan::compile(exact_spec()).unwrap();
    let initial = trophic_initial()
        .map(|(name, (n, d), _)| (name.to_owned(), ratio(n, d)))
        .into();
    let nitrogen = trophic_initial()
        .map(|(name, _, (n, d))| (name.to_owned(), ratio(n, d)))
        .into();
    let mut net = plan
        .start_with_tracers(initial, BTreeMap::from([(NITROGEN.to_owned(), nitrogen)]))
        .unwrap();
    exact_trophic_state(out, SCENARIO, 0, &net);
    let forcings = [
        (ratio(1, 4), ratio(1, 1), BTreeMap::new()),
        (ratio(1, 4), ratio(1, 1), BTreeMap::new()),
        (
            ratio(1, 1),
            ratio(0, 1),
            BTreeMap::from([("grazer".to_owned(), ratio(1000, 1))]),
        ),
    ];
    for (index, (elapsed, nutrient_input, harvests)) in forcings.into_iter().enumerate() {
        let step = index + 1;
        let settled = net
            .step_with_tracer_inputs(
                elapsed,
                nutrient_input,
                BTreeMap::from([(NITROGEN.to_owned(), ratio(1, 2))]),
                harvests,
            )
            .unwrap();
        exact_trophic_state(out, SCENARIO, step, &net);
        for flow in plan.flow_symbols() {
            // Debug text of a struct variant contains spaces; one line keeps
            // one whitespace-free field token.
            let key: String = format!("{flow:?}").split_whitespace().collect();
            exact(
                out,
                SCENARIO,
                step,
                &format!("settled:{key}"),
                settled.transition().settled(flow).unwrap(),
            );
        }
    }
}

fn dense_trophic_state(out: &mut String, scenario: &str, step: usize, net: &DenseTrophicNetwork) {
    for (name, amount) in net.stock_names().iter().zip(net.amounts()) {
        dense(out, scenario, step, &format!("stock:{name}"), *amount);
    }
    let tracer = net.tracer_amounts(NITROGEN).unwrap();
    for (name, amount) in net.stock_names().iter().zip(tracer) {
        dense(
            out,
            scenario,
            step,
            &format!("tracer:{NITROGEN}:{name}"),
            *amount,
        );
    }
    dense(out, scenario, step, "inputs", net.inputs());
    dense(out, scenario, step, "outputs", net.outputs());
    dense(
        out,
        scenario,
        step,
        "balance_residual",
        net.balance_residual(),
    );
    let field = |name: &str| format!("{name}:{NITROGEN}");
    dense(
        out,
        scenario,
        step,
        &field("tracer_inputs"),
        net.tracer_inputs(NITROGEN).unwrap(),
    );
    dense(
        out,
        scenario,
        step,
        &field("tracer_outputs"),
        net.tracer_outputs(NITROGEN).unwrap(),
    );
    dense(
        out,
        scenario,
        step,
        &field("tracer_balance_residual"),
        net.tracer_balance_residual(NITROGEN).unwrap(),
    );
}

fn trophic_dense_nitrogen(out: &mut String) {
    const SCENARIO: &str = "trophic_dense_nitrogen";
    let plan = DenseTrophicNetworkPlan::compile(dense_spec()).unwrap();
    let as_f64 = |(n, d): (i64, i64)| n as f64 / d as f64;
    let initial = trophic_initial()
        .map(|(name, amount, _)| (name.to_owned(), as_f64(amount)))
        .into();
    let nitrogen = trophic_initial()
        .map(|(name, _, amount)| (name.to_owned(), as_f64(amount)))
        .into();
    let mut net = plan
        .start_with_tracers(initial, BTreeMap::from([(NITROGEN.to_owned(), nitrogen)]))
        .unwrap();
    dense_trophic_state(out, SCENARIO, 0, &net);
    let forcings = std::iter::repeat_n((0.25, 1.0, BTreeMap::new()), 40).chain([(
        1.0,
        0.0,
        BTreeMap::from([("grazer".to_owned(), 1000.0)]),
    )]);
    for (index, (elapsed, nutrient_input, harvests)) in forcings.enumerate() {
        let step = index + 1;
        net.step_with_tracer_inputs(
            elapsed,
            nutrient_input,
            BTreeMap::from([(NITROGEN.to_owned(), 0.5)]),
            harvests,
        )
        .unwrap();
        dense_trophic_state(out, SCENARIO, step, &net);
    }
}

fn record() -> String {
    let mut out = String::new();
    food_web_exact(&mut out);
    food_web_dense(&mut out);
    trophic_exact_nitrogen(&mut out);
    trophic_dense_nitrogen(&mut out);
    out
}

#[test]
fn settlement_results_match_the_pre_migration_golden_record() {
    let actual = record();
    if std::env::var_os("ECOSIM_WRITE_GOLDEN").is_some() {
        std::fs::write(GOLDEN, &actual).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(GOLDEN).unwrap();
    let expected: Vec<&str> = expected.lines().collect();
    let actual: Vec<&str> = actual.lines().collect();
    assert_eq!(expected.len(), actual.len(), "golden line count");

    let tolerance = DenseTolerance::default();
    let (mut same, mut total) = (0_usize, 0_usize);
    for (expected, actual) in expected.iter().zip(&actual) {
        let expected_tokens: Vec<&str> = expected.split(' ').collect();
        let actual_tokens: Vec<&str> = actual.split(' ').collect();
        assert_eq!(
            expected_tokens.len(),
            5,
            "malformed golden line: {expected}"
        );
        assert_eq!(actual_tokens.len(), 5, "malformed record line: {actual}");
        assert_eq!(expected_tokens[..4], actual_tokens[..4], "golden key");
        match expected_tokens[3] {
            "exact" => assert_eq!(expected_tokens[4], actual_tokens[4], "{expected}"),
            "dense" => {
                let bits = |text: &str| u64::from_str_radix(text, 16).unwrap();
                let (expected_bits, actual_bits) =
                    (bits(expected_tokens[4]), bits(actual_tokens[4]));
                total += 1;
                if expected_bits == actual_bits {
                    same += 1;
                }
                assert!(
                    tolerance.contains(f64::from_bits(expected_bits), f64::from_bits(actual_bits)),
                    "{expected} vs {actual}"
                );
            }
            other => panic!("unknown golden value mode {other}"),
        }
    }
    eprintln!("dense values bit-identical: {same}/{total}");
}
