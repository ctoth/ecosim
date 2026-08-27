use std::collections::BTreeMap;

use ecosim_core::{
    ComparisonRelation, ExactPairError, ExactPairedModel, ExactRunRecord, PairedComparisonSentence,
    PairedComparisonVerdict,
};
use num_rational::BigRational;

fn q(value: i64) -> BigRational {
    BigRational::from_integer(value.into())
}

fn run(id: &str, producer: i64, herbivore: i64) -> ExactRunRecord {
    ExactRunRecord::new(
        id,
        "topology:v1",
        "parameters:v1",
        "terminal-stock",
        "frozen-cascade:predator-presence",
        12,
        q(1),
        BTreeMap::from([
            ("producer".to_owned(), q(producer)),
            ("herbivore".to_owned(), q(herbivore)),
        ]),
    )
    .unwrap()
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
    let perturbed = ExactRunRecord::new(
        "treatment",
        "different-topology",
        "parameters:v1",
        "terminal-stock",
        "frozen-cascade:predator-presence",
        12,
        q(1),
        BTreeMap::from([
            ("producer".to_owned(), q(13)),
            ("herbivore".to_owned(), q(5)),
        ]),
    )
    .unwrap();
    assert_eq!(
        ExactPairedModel::new(baseline, perturbed),
        Err(ExactPairError::Mismatch("topology"))
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
        ("producer".to_owned(), "primary-producer".to_owned()),
        ("herbivore".to_owned(), "consumer".to_owned()),
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
