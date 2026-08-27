use std::collections::BTreeMap;

use ecosim_core::{
    AllocationSentence, AllocationVerdict, ConsumerSpec, ExactTrophicNetworkPlan, FeedingSpec,
    KineticLaw, KineticSentence, KineticVerdict, ProducerSpec, TrophicFlow, TrophicNetworkSpec,
};
use num_rational::BigRational;
use proptest::prelude::*;

fn q(numerator: i64, denominator: i64) -> BigRational {
    BigRational::new(numerator.into(), denominator.into())
}

fn plan() -> ExactTrophicNetworkPlan {
    ExactTrophicNetworkPlan::compile(TrophicNetworkSpec::new(
        vec![ProducerSpec::new("plant", q(1, 2), q(10, 1), q(1, 100))],
        vec![ConsumerSpec::new("grazer", q(1, 50))],
        vec![FeedingSpec::new(
            "grazer",
            "plant",
            q(1, 4),
            q(5, 1),
            q(3, 4),
        )],
        q(1, 10),
    ))
    .unwrap()
}

fn initial(plant: i64, grazer: i64) -> BTreeMap<String, BigRational> {
    BTreeMap::from([
        ("nutrient".to_owned(), q(100, 1)),
        ("plant".to_owned(), q(plant, 1)),
        ("grazer".to_owned(), q(grazer, 1)),
        ("detritus".to_owned(), q(20, 1)),
    ])
}

#[test]
fn recorded_proposals_satisfy_all_first_slice_kinetic_sentences() {
    let mut run = plan().start(initial(50, 10)).unwrap();
    run.step(q(1, 1), q(0, 1), BTreeMap::new()).unwrap();
    let sentences = [
        KineticSentence::new(
            "plant-growth",
            KineticLaw::ProducerGrowth {
                producer: "plant".to_owned(),
                maximum: q(1, 2),
                half_saturation: q(10, 1),
            },
        )
        .unwrap(),
        KineticSentence::new(
            "grazer-feeding",
            KineticLaw::Feeding {
                consumer: "grazer".to_owned(),
                resource: "plant".to_owned(),
                maximum: q(1, 4),
                half_saturation: q(5, 1),
            },
        )
        .unwrap(),
        KineticSentence::new(
            "plant-mortality",
            KineticLaw::Mortality {
                stock: "plant".to_owned(),
                rate: q(1, 100),
            },
        )
        .unwrap(),
        KineticSentence::new(
            "grazer-mortality",
            KineticLaw::Mortality {
                stock: "grazer".to_owned(),
                rate: q(1, 50),
            },
        )
        .unwrap(),
        KineticSentence::new(
            "decomposition",
            KineticLaw::Decomposition { rate: q(1, 10) },
        )
        .unwrap(),
    ];
    for sentence in sentences {
        assert!(matches!(
            sentence.evaluate(run.transitions()).unwrap(),
            KineticVerdict::Satisfied(_)
        ));
    }
}

#[test]
fn wrong_rate_reports_the_first_exact_proposal_mismatch() {
    let mut run = plan().start(initial(50, 10)).unwrap();
    run.step(q(1, 1), q(0, 1), BTreeMap::new()).unwrap();
    let sentence = KineticSentence::new(
        "wrong-mortality",
        KineticLaw::Mortality {
            stock: "plant".to_owned(),
            rate: q(1, 10),
        },
    )
    .unwrap();
    assert!(matches!(
        sentence.evaluate(run.transitions()).unwrap(),
        KineticVerdict::Violated(violation)
            if violation.transition == 0
                && violation.flow == TrophicFlow::Mortality("plant".to_owned())
                && violation.observed != violation.expected
    ));
}

#[test]
fn source_limited_withdrawals_share_the_kernel_scale() {
    let mut run = plan().start(initial(50, 10)).unwrap();
    run.step(
        q(1, 1),
        q(0, 1),
        BTreeMap::from([("grazer".to_owned(), q(100, 1))]),
    )
    .unwrap();
    let sentence = AllocationSentence::new(
        "grazer-withdrawals",
        "grazer",
        [
            TrophicFlow::Mortality("grazer".to_owned()),
            TrophicFlow::Harvest("grazer".to_owned()),
        ],
    )
    .unwrap();
    assert!(matches!(
        sentence.evaluate(run.transitions()).unwrap(),
        AllocationVerdict::Satisfied(_)
    ));
    for flow in [
        TrophicFlow::Mortality("grazer".to_owned()),
        TrophicFlow::Harvest("grazer".to_owned()),
    ] {
        let transition = &run.transitions()[0];
        assert!(transition.settled(&flow).unwrap() <= transition.proposed(&flow).unwrap());
    }
}

#[test]
fn zero_actor_and_zero_resource_produce_zero_proposals() {
    let mut run = plan().start(initial(0, 0)).unwrap();
    run.step(q(1, 1), q(0, 1), BTreeMap::new()).unwrap();
    let transition = &run.transitions()[0];
    for flow in [
        TrophicFlow::ProducerGrowth("plant".to_owned()),
        TrophicFlow::FeedingAssimilation {
            consumer: "grazer".to_owned(),
            resource: "plant".to_owned(),
        },
        TrophicFlow::FeedingWaste {
            consumer: "grazer".to_owned(),
            resource: "plant".to_owned(),
        },
        TrophicFlow::Mortality("plant".to_owned()),
        TrophicFlow::Mortality("grazer".to_owned()),
    ] {
        assert_eq!(transition.proposed(&flow).unwrap(), &q(0, 1));
    }
}

proptest! {
    #[test]
    fn generated_invalid_kinetic_and_allocation_sentences_fail_structurally(
        magnitude in 1_i64..10_000,
        duplicates in 2_usize..12,
    ) {
        let kinetic = KineticSentence::new(
            "negative-mortality",
            KineticLaw::Mortality {
                stock: "plant".to_owned(),
                rate: q(-magnitude, 1),
            },
        );
        prop_assert!(kinetic.is_err());

        let allocation = AllocationSentence::new(
            "duplicate-withdrawals",
            "grazer",
            std::iter::repeat_n(TrophicFlow::Harvest("grazer".to_owned()), duplicates),
        );
        prop_assert!(allocation.is_err());
    }
}
