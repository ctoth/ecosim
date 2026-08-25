# Adversarial Review: Institution and Conservation Foundations

## Scope

Review the uncommitted implementations in:

- the sibling `institution` repository
- the sibling `conservation` repository

Also consider the intended first integration: a `conservation-institution` crate owned downstream by the Conservation repository. It will interpret exact balance laws as sentences, exact finite traces as models, and trace witnesses/counterexamples as satisfaction evidence. It must demonstrate the Goguen satisfaction condition under two nontrivial bijective renamings of one domain-neutral flow network: ecological trophic pools and economic ledger accounts.

Do not inspect or modify unrelated repositories. Do not edit, commit, or push files.

## Review standard

Be adversarial and mathematical. Passing tests are insufficient. Look for ways the implementation can claim satisfaction, non-vacuity, conservation, exactness, or canonicality without earning the claim.

## Questions

1. Does the Institution trait encode enough of a Goguen institution for the claimed executable slice, without accidentally requiring or omitting essential structure?
2. Can its satisfaction-square or non-vacuity helpers return reassuring results for malformed signatures, morphisms, sentences, models, errors, or carefully chosen examples?
3. Is the exact left-nullspace implementation mathematically correct and deterministic for zero-dimensional, zero-rank, full-rank, rectangular, rational, dependent-row, and sign/scaling cases?
4. Are balance-law and trace constructors capable of silently accepting invalid or ambiguous objects, including empty IDs, duplicate axes, empty laws, missing axes, extra axes, or duplicate trace values?
5. Are derivation certificates being confused with runtime evidence anywhere?
6. Will the proposed conservation-institution bridge be able to implement sentence translation and contravariant model reduct honestly without modifying either core abstraction?
7. Specify adversarial tests that must pass before the dual ecological/economic satisfaction square is accepted.
8. Classify findings as blocking for the first vertical slice, important before publication, or later work.

## Output

Return a concise but substantive report containing:

- verdict;
- findings ordered by severity with file and line references;
- required repairs;
- adversarial test matrix;
- judgment on whether integration may begin.
