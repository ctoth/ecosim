# Review: Noether Institution Architecture

**Reviewer:** Claude (Sonnet 5), acting as external reviewer
**Subject:** `reports/research-noether-institutions.md`
**Scope:** mathematical soundness of the institution-theoretic apparatus; correctness and principledness of the proposed architecture

---

## Verdict

The core formal apparatus is sound. The category `Sign`, the covariant sentence functor `Sen`, the contravariant model functor `Mod : Sign^op -> Cat`, the satisfaction relation, the satisfaction condition, and the institution-comorphism definition and equation are all stated correctly and match the Goguen–Burstall / Goguen–Rosu literature. I found **no hard category-theoretic error** anywhere in the draft — no variance mistake, no malformed satisfaction condition, no incorrectly-directed comorphism.

The report is also right on the question the user actually cares about: economics and ecology should **not** be split into separate institutions merely to look more sophisticated. But the draft does not make the argument for this as strongly as it can be made, and — more seriously — its own "Approaches Found" taxonomy and its own operative construction ("The Proposed Noether Institution") **do not agree with each other**. The document proposes three competing architectures, then builds something that matches none of them cleanly, and never resolves which one is actually being recommended for `D`, `B`, `V`. That unresolved seam is the real defect the review was commissioned to find, more than any single equation.

A second, independent problem is naming: "Noether institution" is used as the proper name for the *entire* kernel, even though the load-bearing, always-present content of that kernel is linear/incidence balance — which the draft itself insists, correctly, is *not* a consequence of Noether's theorem. The name overclaims what the artifact is.

Neither problem is a reason to prefer the "easier" construction. Both problems, correctly resolved, converge on the *same* architecture as the easy one — but for a principled reason (identical `Sen`/`Mod` shape), not a convenience reason. That distinction is what the corrected draft needs to say explicitly.

**Direct answers to the seven review questions:**

1. Yes — the institution definition and satisfaction condition are mathematically sound as stated.
2. Yes, with a caveat: it is more precise to say a modeling *formalism* is a **theory** inside a shared institution unless its sentence language or model category genuinely diverges, in which case it earns status as its own institution. The draft's Approach 1 gets this right; its summary paragraph and comparison table blur it.
3. **One institution with domain theories** — but for the categorical reason given below (§ Most-Principled Architecture), not for ease. The three-institution multiplex (`D`, `B`, `V`) is *not* independently justified once you check whether their `Sen`/`Mod` actually differ; two of the three (`D`, discrete `V`) do not.
4. Partially. Noether's theorem proper (symmetry ⇒ conserved momentum map) is a genuinely different justificatory structure from linear/stoichiometric/accounting balance (nullspace membership), and that distinction is worth preserving *in provenance and naming*. It does not follow that they need to be separate institutions connected by comorphisms/bridge theorems — a shared `Sen`/`Mod` with the variational vocabulary as an optional conservative extension is sufficient and is in fact what the draft already builds in its operative section.
5. Yes — the comorphism direction and satisfaction equation are correct and match Goguen & Rosu (2002).
6. See Material Issues below.
7. See Most-Principled Architecture below.

---

## Material Issues, Ordered by Severity

### 1. (High) The enumerated architectures and the operative construction contradict each other

"Approaches Found" presents three options as if on one axis (economics/ecology as theories vs. as separate institutions vs. a `D`/`B`/`V` multiplex). But "The Proposed Noether Institution" — the section that actually defines `Sign_N`, `Sen_N`, `Mod_N`, `|=` — does not implement Approach 3. It defines **one** signature category in which dimensional typing is baked into every object (`kinds K, each with a dimension vector`) and variational structure is an *optional* extension of the same signature (`optional configuration, Lagrangian…symbols`). There is no `Sign_D`, `Sen_D`, `Mod_D`; no `Sign_V`; no comorphisms `(Φ_D,α_D,β_D)` or `(Φ_V,α_V,β_V)`; no proof that such comorphisms would satisfy the satisfaction condition. "Bridge entailments…subject to typing in `D`" is asserted in prose but never formalized, and the Recommendations section ("Define `D` and `B` first… Add `V`…") reads as sequential conservative extension of one signature — i.e., Approach 1 applied recursively to `D`+`B`+`V` — which is not what Approach 3 described.

This is the actual architectural question the review was asked to adjudicate (Q3/Q4), and the draft leaves it open rather than deciding it. It should be decided (see below), not left as three co-equal "approaches."

### 2. (Medium-high) The name "Noether institution" overclaims what the kernel is

Everywhere the operative signature/sentence/model apparatus is built, the load-bearing content is typed pools, process/boundary matrices, and nullspace-derived balance sentences — ordinary linear algebra and incidence structure, with no symmetry group or action in sight. Variational/Lagrangian content is explicitly *optional*. Yet the whole thing is named after Noether's theorem, whose actual content — invariance of an action under a continuous symmetry entailing a conserved quantity — is a small, separable slice. The draft itself objects to exactly this conflation when discussing double-entry bookkeeping (`It should not be called a consequence of Noether's variational theorem unless an action and a continuous symmetry have actually been supplied`), but the top-level name commits the same error it warns readers against. This will mislead future contributors into thinking every balance law in the kernel derives from a symmetry argument, and it obscures the genuinely different proof-theoretic origin of the two kinds of law that the draft's own provenance tags (`Noether`, `StoichiometricNullspace`, `IncidenceNullspace`, …) correctly distinguish.

### 3. (Medium) The "Complexity vs Quality Tradeoffs" table mischaracterizes the domain-structure decision as a fidelity tradeoff

The table's first row lists "economics and ecology as theories in one institution" as the *lower-fidelity* option against "separate institutions plus proved comorphisms" as *higher-fidelity*. This framing is the one thing the user explicitly objected to. It is also not correct: a comorphism `(Φ,α,β): E → N` only carries content when `Φ`, `α`, `β` do non-trivial translation work. As the draft's own domain tables show, `Sen_E` and `Mod_E` (double-entry postings, SFC identities, transaction traces) are *literally instances* of `Sen_N`/`Mod_N` under typed signatures — same connectives, same quantifiers, same model category (typed pools + admissible traces). There is nothing for `α_E` to translate that isn't already expressible natively, and `β_E` would be forced to be essentially the identity restricted to a sub-signature. Standing up `E` and `C` as separate institutions in that situation is not "more faithful" — it is inflation: extra structure that buys no additional truth-preservation guarantee, because there is no genuine logical divergence to protect against. This is a yes/no categorical fact (does `Sen`/`Mod` actually diverge?), not a point on a simplicity spectrum, and the table should say so rather than implying the simpler choice sacrifices something.

(The surrounding prose — `avoids pretending the two domains have different logics before a genuine difference has been found`, and Recommendation 2 — already states the correct criterion. The table just undercuts it. Fix the table, not the prose.)

### 4. (Medium) The genuine first use for institution comorphisms is misidentified

The draft flags, correctly, that many-to-one aggregation (firms→sector, species→guild) is *not* an ordinary signature morphism and needs "a separately proved interpretation with explicit lumpability… obligations" — but it doesn't connect this to the comorphism machinery it spent a whole section building for `E`/`C`. This is backwards: aggregation is exactly the situation where `α` must do real work (translating a coarse sentence into a derived combination of fine symbols) and `β` must carry a real proof obligation (a lumpability condition, i.e., that the aggregate's dynamics is well-defined independent of which fine-grained state produced it) — precisely the content a comorphism's satisfaction condition is for. Domain separation, per Issue 3, does not need this machinery. Aggregation does. The comorphism apparatus should be re-targeted at aggregation/hiding as its first real client.

### 5. (Low-medium) Satisfaction-condition proof scope is correct but should be stated as the current definition of `Sign_N`, not a policy note

The proof (`Why the satisfaction condition holds`) is honestly scoped to "the conservative-morphism fragment," and Recommendation 6 separately says to permit only renamings/conservative extensions initially. These two things are the same fact stated twice in different registers. As written, a reader could conclude `Sign_N` already includes aggregation/hiding morphisms (since they're discussed as part of the signature design) with the proof simply "pending." It would be more precise to state plainly: *today*, `Sign_N`'s morphism class **is** {renamings, conservative extensions}; that is the institution that is actually proved; aggregation/hiding are not yet objects of this institution at all, only a specified target for a future comorphism (per Issue 4).

### 6. (Low) Minor semantic ambiguity: is `StateTransition(S,B)` definitional or falsifiable?

`Sen_N` lists "state-transition equations" as sentences (implying a model can fail to satisfy them), while the `Models` section describes models as carrying "a transition relation or evolution law," which could be misread as baking `x_{k+1}-x_k=Sr_k+Bb_k` into the definition of a model (making it trivially true). The intent — confirmed by "*An implementation trace with an unbalanced update is still data that can be interpreted as a model*" — is that raw trace models carry only observed/simulated `(x_k, r_k, b_k)` values with no presupposed law, and `StateTransition(S,B)` is genuinely checked against them. One clarifying sentence in the `Models` subsection would close this ambiguity.

### 7. (Low) Missing citations for the apparatus actually used

The exact comorphism definition and satisfaction equation used (§ "2. Separate institutions connected by comorphisms") is Goguen & Rosu's, not Goguen & Burstall's — the 1992 paper defines institutions but not this named comorphism apparatus with this direction convention. Diaconescu's *Institution-Independent Model Theory* is the standard source for theoroidal/derived signature morphisms, which is exactly the tool needed for aggregation (Issue 4) and for rigorously defining a Grothendieck-style multiplex if Approach 3 is ever pursued formally. Both are absent from the bibliography.

**No hard errors found in:** the institution definition; the satisfaction condition statement and its (scoped) proof; the comorphism direction/equation; the discrete and continuous Noether entailment schemas (these match Marsden & West 2001); the nullspace-balance semantics; the honest caveat about variational integrators not conserving exact energy.

---

## Most-Principled Architecture

Resolve Issue 1 by collapsing to **one institution**, and justify it by a checkable criterion rather than by ease:

> **Two constructs belong in the same institution iff their `Sen` and `Mod` are the same functors (up to signature).** A separate institution, connected by a comorphism, is warranted only when `Φ`, `α`, `β` would have to do genuine, non-trivial translation work — i.e., when the sentence logic or the model category actually differs, not merely when the vocabulary differs.

Applying that test:

- **Economics vs. ecology:** same `Sen` (typed balance/transition formulas), same `Mod` (typed pools + admissible traces). → **theories, not institutions.** (Confirms the draft's Approach 1, for the stated reason rather than convenience.)
- **`D` (dimensional typing) vs. `B` (balance):** dimension/kind metadata is not a peer logic with its own satisfaction relation; it is the sort discipline every `Sign_N` object already carries. → **fold into `Sign_N`, not a separate institution.**
- **Discrete `V` (Lagrangian/momentum) vs. `B`:** a discrete Lagrangian system's reduct along the inclusion `Sig_B ⊆ Sig_V` (forgetting `L_d, G, ξ, J`) is an ordinary state-transition system — exactly the "conservative extension" morphism the draft already permits. → **`V`'s discrete form is an optional conservative extension of the same institution, not a separate one.** The "bridge entailment" from symmetry sentences to balance sentences is then just an ordinary intra-institution entailment (a momentum map is itself a pool subject to `OpenBalance`), requiring no comorphism at all.

So the corrected architecture is:

**`𝒦` (rename from "Noether institution" — see Issue 2), one institution, built by cumulative conservative signature extension:**

- **`Sign_𝒦`:** typed open-process schemas exactly as the draft's `Sign_N` already defines — kinds/dimensions always present, balance structure (`S, B`) present once processes/boundaries are declared, variational structure (`Q, L_d, G, ξ, F, J`) optional. Morphisms: renamings and conservative extensions only, with the draft's existing satisfaction-condition proof (Issue 5: state this as the current definition, not a temporary restriction).
- **`Sen_𝒦(Σ)`, `Mod_𝒦(Σ)`, `⊨_Σ`:** exactly as drafted, with the falsifiability of `StateTransition`/balance sentences made explicit (Issue 6).
- **Theories, not institutions:** `T_E, T_C ⊆ Sen_𝒦(Σ)` for the appropriately typed `Σ`, per the draft's own domain tables. Promotion to a separate institution is gated on the test above (a genuinely different sentence logic — e.g. modal/epistemic content for expectations — or a genuinely different model category — e.g. stochastic ecological birth–death processes), not on complexity or maturity.
- **Noether sub-theory:** `{Invariant(L,G), (Forced)EulerLagrange(L)} ⊨ MomentumConserved(J)` / forced momentum balance, exactly as drafted — kept as a *named, provenance-tagged* fragment of `Sen_𝒦`, not as the name of the whole kernel.
- **Genuine institution comorphisms, reserved for:**
  1. **Aggregation/lumping** (Issue 4) — the first real client: `Φ` maps a coarse signature into a fine one (or vice versa), `α` translates a coarse sentence into a derived combination of fine symbols, `β` carries an explicit lumpability proof obligation. This is where the comorphism satisfaction condition earns its keep in v1, not domain-splitting.
  2. **Future genuinely different logics** — a continuous-time smooth Lagrangian/symplectic institution (if/when needed), stochastic ecological model categories, or interchange with Hets/CASL — each connected to `𝒦` by a comorphism only once its `Sen`/`Mod` is shown to actually diverge.
- **Provenance:** keep and elevate the draft's existing tag set (`Noether`, `StoichiometricNullspace`, `IncidenceNullspace`, `ForcedBalance`, `Declared`) as a first-class field on every derived sentence/theorem, and add `LumpedFrom(σ)` once aggregation comorphisms exist, recording which fine-grained theory and proof a coarse-grained law descends from.

---

## Exact Corrections Recommended for the Draft

1. Reconcile "Approaches Found" with "The Proposed Noether Institution": either drop Approach 3 as a considered-and-rejected alternative (with the reasoning above — `D` and discrete `V` don't diverge from `B` in `Sen`/`Mod`), or, if a future continuous-time `V` is anticipated, say explicitly that *that* is the only case in which a fourth, genuinely separate institution and comorphism would be justified.
2. Rename the top-level construct away from "Noether institution" (e.g. "typed conservation institution" or "conservation kernel institution"), and reserve "Noether" strictly for the symmetry-derived sub-theory/entailment and its provenance tag.
3. Rewrite the "Domain structure" row of the "Complexity vs Quality Tradeoffs" table so it states the categorical criterion (identical `Sen`/`Mod` ⇒ theories, not institutions) rather than presenting it as a simplicity/fidelity tradeoff.
4. Move the aggregation/lumping discussion (currently one sentence under `Signatures`) into a short subsection explicitly framed as "the first target for the comorphism apparatus," referencing the comorphism satisfaction equation already derived for `E`/`C` and repurposing it there instead.
5. In the `Models` subsection, add one sentence clarifying that a raw trace model carries no presupposed evolution law; `StateTransition(S,B)` is genuinely checked, not assumed.
6. Replace "permit only renamings and conservative extensions… initially" (Recommendation 6) with a direct statement that this **is** the current definition of `Sign_N`'s morphism class, not a temporary policy.
7. Add Goguen & Rosu (2002), "Institution Morphisms," and Diaconescu (2008), *Institution-Independent Model Theory*, to the references, since the comorphism apparatus and the aggregation/theoroidal-morphism machinery both come from there.

---

## Unresolved Mathematical Choices

1. Whether discrete-only variational content is sufficient for the foreseeable roadmap, or whether a continuous-time/smooth model category will eventually be needed — which is the one case that would genuinely justify a separate institution and comorphism for `V`.
2. How to formalize aggregation/lumping precisely: as an institution comorphism, as a Diaconescu-style theoroidal signature morphism, or as bespoke abstraction-relation machinery — and what the general (non-Markov, nonlinear) lumpability condition should be.
3. Whether `Mod_𝒦(Σ)` should stay locally discrete indefinitely or gain real model morphisms once behavior-preserving abstraction/refinement is needed (tied to #2).
4. Whether the kernel needs a proof-theoretic entailment system (`⊢`, e.g. a π-institution/entailment-system layer) alongside the purely semantic `⊨` used for the Noether entailment, for mechanized certificate checking — or whether semantic entailment plus finite-trace certificates remains sufficient.
5. Which boundary is authoritative for a given economic or ecological instance (echoes the draft's own Open Questions 4–5) — not a category-theoretic question, but it determines which `Σ` is "the" canonical signature per domain instance, which the architecture above otherwise leaves as a modeling choice per theory.
