use std::collections::BTreeMap;

use ecosim_core::{
    ComparisonRelation, ConsumerSpec, ExactPairError, ExactPairedModel, ExactRunRecord,
    ExactTrophicNetwork, PairedComparisonSentence, PairedComparisonVerdict, ProducerSpec,
    TrophicNetworkSpec,
};
use num_rational::BigRational;

fn q(value: i64) -> BigRational {
    BigRational::from_integer(value.into())
}

fn audited_network(
    producer: i64,
    herbivore: i64,
    nutrient_input: i64,
    producer_growth: i64,
) -> ExactTrophicNetwork {
    audited_network_with_elapsed(producer, herbivore, nutrient_input, producer_growth, q(1))
}

fn audited_network_with_elapsed(
    producer: i64,
    herbivore: i64,
    nutrient_input: i64,
    producer_growth: i64,
    elapsed: BigRational,
) -> ExactTrophicNetwork {
    let spec = TrophicNetworkSpec::new(
        vec![ProducerSpec::new(
            "producer",
            q(producer_growth),
            q(1),
            q(0),
        )],
        vec![ConsumerSpec::new("herbivore", q(0))],
        vec![],
        q(0),
    );
    let initial = BTreeMap::from([
        ("nutrient".to_owned(), q(0)),
        ("producer".to_owned(), q(producer)),
        ("herbivore".to_owned(), q(herbivore)),
        ("detritus".to_owned(), q(0)),
    ]);
    let mut network = ExactTrophicNetwork::new(spec, initial).unwrap();
    network
        .step(elapsed, q(nutrient_input), BTreeMap::new())
        .unwrap();
    network
}

fn run(id: &str, producer: i64, herbivore: i64) -> ExactRunRecord {
    ExactRunRecord::from_trophic_network(id, &audited_network(producer, herbivore, 0, 0)).unwrap()
}

#[test]
fn run_records_retain_the_trace_and_complete_positive_audit() {
    let record = run("control", 10, 7);
    assert_eq!(record.transitions().len(), 1);
    assert!(record.law_evidence().is_satisfied());
    assert_eq!(record.terminal("producer"), Some(&q(10)));
}

#[test]
fn exact_pair_returns_named_rational_witnesses_and_first_violation() {
    let model = ExactPairedModel::new(run("control", 10, 7), run("treatment", 13, 5)).unwrap();
    let sentences = vec![
        PairedComparisonSentence::new(
            "producer-increases",
            "producer",
            ComparisonRelation::GreaterThan,
            q(2),
        )
        .unwrap(),
        PairedComparisonSentence::new(
            "herbivore-increases",
            "herbivore",
            ComparisonRelation::GreaterThan,
            q(0),
        )
        .unwrap(),
        PairedComparisonSentence::new(
            "not-evaluated",
            "producer",
            ComparisonRelation::GreaterThan,
            q(0),
        )
        .unwrap(),
    ];

    let evidence = model.evaluate(&sentences).unwrap();
    assert!(!evidence.is_satisfied());
    assert_eq!(evidence.verdicts().len(), 2);
    assert!(matches!(
        &evidence.verdicts()[0],
        PairedComparisonVerdict::Satisfied(witness)
            if witness.sentence == "producer-increases"
                && witness.baseline_run == "control"
                && witness.perturbed_run == "treatment"
                && witness.delta == q(3)
    ));
    assert!(matches!(
        &evidence.verdicts()[1],
        PairedComparisonVerdict::Violated(violation)
            if violation.sentence == "herbivore-increases"
                && violation.delta == q(-2)
    ));
}

#[test]
fn paired_model_rejects_mismatched_experiment_metadata() {
    let baseline = run("control", 10, 7);
    let perturbed =
        ExactRunRecord::from_trophic_network("treatment", &audited_network(13, 5, 0, 1)).unwrap();
    assert_eq!(
        ExactPairedModel::new(baseline, perturbed),
        Err(ExactPairError::Mismatch("compiled plan"))
    );
}

#[test]
fn paired_model_retains_audited_intervention_differences() {
    let baseline = run("control", 10, 7);
    let perturbed =
        ExactRunRecord::from_trophic_network("treatment", &audited_network(13, 5, 1, 0)).unwrap();
    assert!(ExactPairedModel::new(baseline, perturbed).is_ok());
}

#[test]
fn paired_model_rejects_a_different_observation_time_grid() {
    let baseline = run("control", 10, 7);
    let perturbed = ExactRunRecord::from_trophic_network(
        "treatment",
        &audited_network_with_elapsed(13, 5, 0, 0, q(2)),
    )
    .unwrap();
    assert_eq!(
        ExactPairedModel::new(baseline, perturbed),
        Err(ExactPairError::Mismatch("observation time grid"))
    );
}

#[test]
fn conservative_axis_renaming_preserves_comparative_truth() {
    let model = ExactPairedModel::new(run("control", 10, 7), run("treatment", 13, 5)).unwrap();
    let sentence = PairedComparisonSentence::new(
        "producer-increases",
        "producer",
        ComparisonRelation::GreaterThan,
        q(2),
    )
    .unwrap();
    let renaming = BTreeMap::from([
        ("nutrient".to_owned(), "mineral-nutrient".to_owned()),
        ("producer".to_owned(), "primary-producer".to_owned()),
        ("herbivore".to_owned(), "consumer".to_owned()),
        ("detritus".to_owned(), "dead-organic-matter".to_owned()),
    ]);

    let source = sentence.evaluate(&model).unwrap();
    let target = sentence
        .rename(&renaming)
        .unwrap()
        .evaluate(&model.rename_axes(&renaming).unwrap())
        .unwrap();
    assert!(matches!(source, PairedComparisonVerdict::Satisfied(_)));
    assert!(matches!(target, PairedComparisonVerdict::Satisfied(_)));
}

#[test]
fn conservative_axis_renaming_preserves_comparative_falsity() {
    let model = ExactPairedModel::new(run("control", 10, 7), run("treatment", 13, 5)).unwrap();
    let sentence = PairedComparisonSentence::new(
        "producer-does-not-increase-enough",
        "producer",
        ComparisonRelation::GreaterThan,
        q(4),
    )
    .unwrap();
    let renaming = BTreeMap::from([
        ("nutrient".to_owned(), "mineral-nutrient".to_owned()),
        ("producer".to_owned(), "primary-producer".to_owned()),
        ("herbivore".to_owned(), "consumer".to_owned()),
        ("detritus".to_owned(), "dead-organic-matter".to_owned()),
    ]);

    let source = sentence.evaluate(&model).unwrap();
    let target = sentence
        .rename(&renaming)
        .unwrap()
        .evaluate(&model.rename_axes(&renaming).unwrap())
        .unwrap();
    assert!(matches!(source, PairedComparisonVerdict::Violated(_)));
    assert!(matches!(target, PairedComparisonVerdict::Violated(_)));
}
