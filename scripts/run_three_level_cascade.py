"""Run the frozen dense confirmation and short exact evidence audits."""

from __future__ import annotations

import json

from ecosim import load_frozen_cascade, run_frozen_cascade


def main() -> None:
    scenario = load_frozen_cascade()
    result = run_frozen_cascade(scenario)
    audits = {
        "predator_presence": result.exact_predator_presence,
        "nutrient_pulse": result.exact_nutrient_pulse,
        "predator_harvest_press": result.exact_predator_harvest_press,
    }
    output = {
        "scenario_status": "frozen-confirmatory",
        "model_scope": (
            "abstract material-equivalent trophic model; "
            "not an empirical ecosystem validation"
        ),
        "dense_confirmation_passed": result.confirmation_passed,
        "dense_observations": [
            {
                "comparison": item.comparison,
                "stock": item.stock,
                "observed": item.observed,
                "relation": item.relation,
                "threshold": item.threshold,
                "passed": item.passed,
            }
            for item in result.observations
        ],
        "exact_response_signs_passed": result.exact_signs_passed,
        "exact_sign_observations": [
            {
                "comparison": item.comparison,
                "stock": item.stock,
                "observed": item.observed,
                "sign": item.sign,
                "epsilon": item.epsilon,
                "passed": item.passed,
            }
            for item in result.exact_sign_observations
        ],
        "exact_evidence_satisfied": result.exact_evidence_satisfied,
        "exact_audits": {
            comparison: {
                side: {
                    "trace_length": audit.trajectory.values.shape[1],
                    "cumulative_input": audit.cumulative_input,
                    "cumulative_output": audit.cumulative_output,
                    "evidence_satisfied": audit.evidence.satisfied,
                    "laws": [
                        {
                            "name": law.law.name,
                            "axis": law.law.axis_name,
                            "grade": law.law.grade,
                            "satisfied": law.satisfied,
                        }
                        for law in audit.evidence.laws
                    ],
                }
                for side, audit in (
                    ("baseline", pair.baseline),
                    ("perturbed", pair.perturbed),
                )
            }
            for comparison, pair in audits.items()
        },
        "dense_balance_tolerance": {
            "absolute": scenario.dense_balance_absolute,
            "relative": scenario.dense_balance_relative,
        },
    }
    print(json.dumps(output, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
