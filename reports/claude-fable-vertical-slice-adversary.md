# Claude Fable Adversarial Review: Institutional Conservation Vertical Slice

## Verdict

Not complete at the reviewed revision. The bridge mechanics were internally sound, with no reachable public panic, membership leak, or confusion between semantic violation and malformedness. The tests did not meet the stated architectural claim because they constructed two already-domain-specific source signatures and laws rather than translating one shared neutral mathematical object.

## Central finding

The ecological fixture began with ecological axes and a biomass kind. The economic fixture began with economic axes and a money kind. Each independently derived a fresh law. This demonstrated two analogous intra-domain renamings, not one domain-neutral signature and law interpreted into two domain vocabularies.

The original `AxisRenaming` required exact kind identity and copied the sentence kind unchanged. That was correct for synonym renamings but made a genuine neutral-to-domain interpretation impossible. A signature morphism for the claimed slice must explicitly map both axis symbols and kind symbols.

## Required repairs

1. Add a validated total and injective kind-symbol map to `AxisRenaming`.
2. Validate an axis mapping against the mapped target kind rather than requiring kind identity.
3. Translate `BalanceLaw.kind` through the kind-symbol map.
4. Build literally one neutral signature and one exact incidence-derived law.
5. Use that same source signature and law for both ecological and economic morphisms.
6. Assert translated laws carry the expected domain kinds.
7. Preserve the existing true/true ecological and false/false economic semantic squares.

## Confirmed behavior

- Sentence translation was covariant and model reduct contravariant.
- Model and sentence membership checks were structurally sound.
- False/false represented a genuine balance violation on both sides, not malformed input.
- The non-vacuity helper used the actual target-side cases meaningfully, although all four square corners would be stronger.
- The path-plus-version Institution dependency was appropriate for local development and correctly prevented registry publication before release ordering was resolved.
- Existing internal `expect` calls were unreachable through the validated public constructors, though replacing them with typed failures was recommended for literal panic freedom.

## Integration judgment

Final verification should begin only after the shared-source and kind-symbol-map repairs. Until then, the slice showed structural analogy across domains rather than one institutionally translated theory.
