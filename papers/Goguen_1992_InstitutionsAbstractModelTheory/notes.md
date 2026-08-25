---
title: "Institutions: Abstract Model Theory for Specification and Programming"
authors: "Joseph Goguen and Rod Burstall"
year: 1992
venue: "Journal of the ACM"
doi_url: "https://doi.org/10.1145/147508.147524"
---

# Reading notes — Goguen and Burstall (1992)

## Pages 1–20 of the PDF (journal pages 95–114)

- The paper introduces an institution to separate the model-theoretic structure shared by logical systems from the syntax peculiar to any one logic. The shared data are signatures, sentences, models, and satisfaction; signature morphisms express changes of notation such as enrichment, binding, and renaming. [PDF pp. 1–5; journal pp. 95–99]
- Definition 1 is load-bearing for this codebase. An institution consists of (1) a **category** `Sign`, (2) a functor `Sen : Sign -> Set`, (3) a contravariant functor `Mod : Sign -> Cat`, and (4) a satisfaction relation for each signature. For every signature morphism `phi : Sigma -> Sigma'`, model `m'`, and source sentence `e`, the satisfaction condition is `m' |= Sen(phi)(e)` iff `Mod(phi)(m') |= e`. [PDF pp. 7–8; journal pp. 101–102]
- The directionality is deliberate: sentences translate covariantly with the signature morphism while models reduce contravariantly. Because `Sen` and `Mod` are functors on a category, translations/reducts must preserve identity and composition in addition to satisfying each individual square. [PDF pp. 7–8; journal pp. 101–102]
- The authors explicitly allow a simpler set-valued model functor for some purposes, but the primary definition is category-valued so that model homomorphisms are retained. A set-valued implementation can be a legitimate specialization; omitting the signature category/functor laws is a weaker executable fragment, not the whole definition. [PDF pp. 7–8, 16; journal pp. 101–102, 110]
- Presentations and theories arise from collections of sentences and their model classes. Semantic consequence and closure form a Galois connection; a theory is a closed presentation. Theory morphisms are signature morphisms whose translated consequences are consequences of the target. [PDF pp. 8–11; journal pp. 102–105]
- The satisfaction condition makes reduct along a theory morphism well-defined: a target model satisfying the target theory reduces to a model satisfying the source theory. It can also be written as an equality between inverse images of model classes and closures of translated sentences. [PDF pp. 10–11; journal pp. 104–105]
- The paper uses colimits of theories to assemble large specifications from smaller theories. This is a later compositional layer, not required merely to witness an institution, but it explains why categorical identity/composition and lawful theory morphisms matter operationally. [PDF pp. 12–15; journal pp. 106–109]
- Definition 12 repackages an institution as a functor from `Sign` into a category of twisted relations; the satisfaction condition is precisely the commuting condition for those relation morphisms. This confirms that a collection of unrelated satisfaction checks is insufficient: the family must itself be functorial. [PDF pp. 15–17; journal pp. 109–111]
- Constraints and free extensions add stronger ways of restricting models beyond satisfaction of ordinary sentences. They are useful future structure for domain theories, but they are not prerequisites for the current conservation slice. [PDF pp. 18–20; journal pp. 112–114]

## Provisional implementation obligations

1. Represent, or provide a law-bearing interface for, identities and composition of signature morphisms.
2. Property-test `Sen(id) = id`, `Sen(g o f) = Sen(g) o Sen(f)`, `Mod(id) = id`, and `Mod(g o f) = Mod(f) o Mod(g)`.
3. Property-test the satisfaction condition for arbitrary supported morphisms and for their compositions.
4. Describe a set-valued model implementation as a specialization of the paper's category-valued definition; do not claim the full category-valued structure until model morphisms are represented.

## Pages 21–40 of the PDF (journal pages 115–134)

- Sections 3.1–3.3 develop free extensions and model constraints. The key implementation lesson is structural: when constraints are translated along a signature morphism, their satisfaction condition follows from **functoriality of `Mod`**. The proof literally reduces the needed equality to composition of reducts. [PDF pp. 21–24; journal pp. 115–118]
- Proposition 23 checks that adding constraints produces another institution by separately establishing the signature category, functoriality of sentence translation, the model functor, and the satisfaction condition. This is a useful checklist for every domain-specific construction we claim is an institution. [PDF p. 25; journal p. 119]
- Model constraints distinguish loose models from intended/free interpretations. For the current conservation system, ordinary invariant sentences may suffice initially; claims about canonical or freely generated ecosystem/economic models would require this richer layer and should not be smuggled into the base satisfaction relation. [PDF pp. 22–30; journal pp. 116–124]
- Institution morphisms connect different institutions. Definition 32 gives concrete data: a functor between signature categories, a natural translation of sentences, and a natural model mapping, all satisfying a cross-institution satisfaction condition. Naturality adds two more commuting diagrams beyond satisfaction. [PDF pp. 31–32; journal pp. 125–126]
- Sound reuse of a theorem prover across institutions needs more than an institution morphism: Definition 34 requires the model component to be surjective on objects. This cautions against claiming that an adapter automatically transfers all consequences merely because pointwise satisfaction is preserved. [PDF p. 32; journal p. 126]
- Duplex and multiplex institutions formalize simultaneous use of more than one logic. This supports keeping conservation mathematics and ecosystem/economic domain languages distinct while connecting them with explicit institution morphisms, rather than forcing every sentence form into one monolithic institution. [PDF pp. 35–38; journal pp. 129–132]
- The summary emphasizes a general-construction discipline: establish the institution laws and satisfaction condition, then obtain theory structuring and cross-logic reuse. It explicitly calls checking the satisfaction condition tedious and suggests constructing institutions from simpler structures that guarantee it. [PDF pp. 37–38; journal pp. 131–132]
- Appendix A makes identities and composition concrete for equational signatures, algebras, homomorphisms, and reduct. This is a model for our API: morphisms carry explicit source/target-compatible maps, and both signature and model levels state their category/functor operations rather than leaving them implicit. [PDF pp. 39–40; journal pp. 133–134]

## Additional implementation obligations

5. Separate “institution instance” laws from stronger cross-institution adapter laws; an adapter should not be called sound for consequence transfer without its extra hypotheses.
6. Make source/target compatibility of morphisms explicit enough that composition cannot silently combine mismatched signatures.
7. Treat the conservation-to-domain bridge as an institution morphism or comorphism only after its naturality and satisfaction laws are represented and tested; until then it is a narrower adapter.

## Pages 41–52 of the PDF (journal pages 135–146)

- The equational-logic example constructs the model reduct as an actual contravariant functor, constructs equation translation as a covariant functor, defines satisfaction, and then proves the satisfaction condition. The example's order reinforces that all four pieces and their laws are part of establishing an institution. [PDF pp. 41–42; journal pp. 135–136]
- The many-sorted first-order example repeats the same construction with richer signatures, model categories, sentence syntax, and Tarskian satisfaction. Restricting an established institution to a sentence class is legitimate only when that class is closed under signature translation. [PDF pp. 43–47; journal pp. 137–141]
- The appendices demonstrate that “models” need not be simulations or physical states in an informal sense: they are interpretations appropriate to the chosen signatures and sentences. For our domain, a ledger state/trace may serve as a model only after its signature-indexed interpretation and reduct behavior are specified. [PDF pp. 41–47; journal pp. 135–141]
- Remaining pages are references. Several cited lines of work concern general systems theory, which is historically suggestive for this project, but the present audit relies on the definitions and constructions proved in this paper rather than importing claims from unread references. [PDF pp. 48–52; journal pp. 142–146]

## Final assessment for this implementation

The existing satisfaction-square API is a valuable nucleus, but by Definition 1 it is not yet a complete Goguen-style Institution interface. The smallest principled repair is to add a lawful signature category (identities and composition), require sentence translation and model reduct to behave functorially, and test those laws. Model morphisms may remain erased in an explicitly documented set-valued specialization. Cross-domain bridges should be named adapters until their naturality and cross-institution satisfaction laws are implemented.

## Collection Cross-References

### Already in Collection

- (none found)

### New Leads (Not Yet in Collection)

- Tarlecki, Burstall, and Goguen (1991), "Some Fundamental Algebraic Tools for the Semantics of Computation" - related institution-independent specification machinery.
- Meseguer (1989), "General Logics" - mappings between logical systems.

### Supersedes or Recontextualizes

- (none)

### Conceptual Links (not citation-based)

- (none among the four ecological and conservation papers processed in this pass)

### Cited By (in Collection)

- (none found)
