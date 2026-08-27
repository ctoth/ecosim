# Stock–Flow Source Verification

## Method

The research-paper citation verifier expects `@cite_key` syntax and a
`papers/keymap.tsv` mapping. The existing research report uses ordinary
Markdown links, so automated extraction resolves no claim-to-paper pairs.
Claims were therefore mapped manually, then graded one paper at a time using
the plugin's required verdicts:

- `SUPPORTED`: the source directly states the claim;
- `PARTIAL`: a material qualifier or project extension was omitted;
- `UNSUPPORTED`: the source does not establish the claim;
- `MISATTRIBUTED`: the cited source concerns something materially different.

Each paper was assigned to a separate read-only grader. The main agent also
read the local notes for Goguen–Burstall and Mahdi et al. and reconciled every
reported correction into `PLAN.md`,
`reports/research-noether-institutions.md`, or the formal specification.

## Goguen and Burstall (1992)

Sources:

- local `papers/Goguen_1992_InstitutionsAbstractModelTheory/notes.md`;
- local abstract derived from the paper;
- paper Definition 1, PDF pp. 7–8, journal pp. 101–102.

Short source passages:

```text
"a category Sign"
"Sen : Sign -> Set"
"a contravariant functor Mod"
"sentences translate covariantly ... models reduce contravariantly"
"translations/reducts must preserve identity and composition"
```

| Claim | Verdict | Resolution |
|---|---|---|
| An Institution has signatures, sentences, models, and satisfaction satisfying the square. | PARTIAL | Corrected to require a lawful signature category plus functorial sentence translation and model reduct. |
| Sentence translation is covariant and model reduct contravariant. | SUPPORTED | Retained. |
| Truth is preserved under a supported change of notation. | SUPPORTED | Retained with identity/composition obligations. |
| Any concrete economy, ecosystem, or trace simply is a model. | PARTIAL | Corrected to “can serve as a model once its signature-indexed interpretation and reduct behavior are defined.” |
| Economics and ecology can initially be theories described by arbitrary sentence sets. | PARTIAL | Corrected to presentations/profiles; “theory” is reserved for semantic closure. |
| The report's variance convention is a Goguen–Burstall institution comorphism. | MISATTRIBUTED | Replaced with “candidate cross-institution adapter”; variance, naturality, and satisfaction remain proof obligations. |
| A D/B/V multiplex is the uniquely most faithful design. | PARTIAL | Relabeled as one proposed multiplex design. |
| The project-specific typed open-process language comes from Goguen–Burstall. | UNSUPPORTED | Labeled explicitly as a project-specific proposed instance. |
| A set-valued or locally discrete model specialization is legitimate. | PARTIAL | Retained only as an explicitly documented discrete-category specialization with reduct functoriality. |
| Finite-trace witnesses are part of the paper's definition. | PARTIAL | Relabeled as additional project evidence for a project-defined satisfaction relation. |
| The coordinate-renaming sketch by itself proves a complete Institution. | PARTIAL | Relabeled as an atomic proof sketch and supplemented with category/functor obligations. |

## Mahdi et al. (2017)

Sources:

- local `papers/Mahdi_2017_ConservationLawsBiochemicalReaction/notes.md`;
- local abstract derived from the paper;
- Proposition 7, PDF pp. 6–7.

Short source passages:

```text
"if a row vector omega satisfies omega Gamma = 0, then H(x) = omega x is conserved"
"The left kernel has dimension n - rank(Gamma)"
"structural linear conservation laws, not all conservation laws"
"Keep the result independent of rates and numerical integration"
```

| Claim | Verdict | Resolution |
|---|---|---|
| A left-null vector of the stoichiometric matrix gives a structural linear conservation law. | SUPPORTED | Retained. |
| The left-kernel basis has size `rows(S) - rank(S)`. | SUPPORTED | Added to the plan's certificate acceptance criteria. |
| Mahdi directly proves the project's discrete open equation with `B b[k]`. | PARTIAL | Corrected to an algebraic project extension of Mahdi's closed continuous structural premise. |
| Exact nullspace annihilation may be checked to construct a certificate. | PARTIAL | Exact annihilation is supported; sealed certificate mechanics are labeled project evidence policy. |
| Stoichiometric, incidence, Noether, and declared origins are all Mahdi's taxonomy. | UNSUPPORTED | Relabeled as a project provenance taxonomy with separate sources. |
| Exact rationals and finite-trace tolerances are prescribed by the paper. | PARTIAL | Relabeled as project representation and evidence policies. |
| A boundary residual is incomplete-model evidence under Mahdi's semantics. | UNSUPPORTED | Relabeled as this project's explicit open-system semantics. |
| The nullspace method derives all conservation laws. | UNSUPPORTED | The implementation and documents consistently say “structural linear conservation laws”; kinetic and nonlinear first integrals remain separate. |

## Marsden and West (2001)

Primary source:

- [CaltechAUTHORS record](https://authors.library.caltech.edu/records/1h96d-ymc40);
- official 158-page PDF, MD5 `339B9A3F1902C70F1E942672F2DF26D0`.

Short source passages and equations:

```text
Theorem 1.3.3, p. 375: J_Ld o F_Ld = J_Ld
Eq. 3.1.4, p. 423: forced momentum change equals the projected force integral
p. 373: (F_Ld)^* Omega_Ld = Omega_Ld
p. 479: extended time translation recovers a discrete energy statement
p. 500: recommendation not to enforce exact energy conservation
```

| Claim | Verdict | Resolution |
|---|---|---|
| Invariance plus discrete Euler–Lagrange evolution preserves the discrete momentum map. | SUPPORTED | Retained. |
| Discrete forcing changes momentum by a projected-force source term. | SUPPORTED | Retained. |
| Marsden–West formulate this as an Institution entailment and runtime ledger witness. | PARTIAL | Institutional framing and evidence are labeled as this project's Goguen-style reformulation. |
| Force is required in every discrete variational system. | PARTIAL | Corrected: unforced ingredients first; the forced extension adds `f_d^-` and `f_d^+`. |
| Fixed-step variational integrators preserve symplectic form and momentum but not exact energy. | PARTIAL | Qualified to unforced systems and to the true Hamiltonian; extended formulations conserve a discrete energy under stated conditions. |
| Marsden–West survey implementations lacking Institution adapters. | UNSUPPORTED | Replaced with the narrower statement that the paper is a mathematical review, not this project's implementation specification. |
| A genuine discrete Noether slice needs configuration, discrete Lagrangian, symmetry, evolution, and momentum map. | SUPPORTED | Retained. |
| A process-matrix nullspace identity alone is Noether's theorem. | UNSUPPORTED | The project explicitly rejects this classification. |

## Godley and Lavoie (2007), Chapter 2

Primary source: the Levy Economics Institute chapter PDF, pp. 25–44.

Short source passages:

```text
"Open economies will not be examined"
"every row and every column sums to zero"
"no characterization of behaviour"
"change arising from revaluations"
```

| Claim | Verdict | Resolution |
|---|---|---|
| The chapter supplies sectors, accounts, instruments, inventories, and rest-of-world ports. | PARTIAL | Retained the domestic accounting vocabulary; labeled rest-of-world ports a project open-economy extension. |
| The project's typed pools, kinds, process columns, boundary columns, and traces are source terminology. | UNSUPPORTED | The table is explicitly introduced as a project encoding. |
| Double entry means merely one equal-and-opposite pair. | PARTIAL | Replaced with counterpart source/use entries and the chapter's economy-wide row/column balancing discipline. |
| Stock change is exhausted by transaction flows. | PARTIAL | Added revaluation change, with the simpler equation restricted to fixed-price assets without capital gains. |
| Every row and column of the transactions-flow matrix sums to zero. | SUPPORTED | Retained as `1^T T = 0` and `T 1 = 0` for the chapter's rows-by-sectors matrix. |
| Inventories and financial positions link opening stocks, transactions, revaluations, and closing stocks. | SUPPORTED | Retained. |
| The chapter covers issuance, buy-backs, imports, exports, losses, and depreciation as one general rule. | PARTIAL | Retained issuance, buy-backs, and revaluations; labeled remaining explicit symbols as project requirements when in scope. |
| The chapter derives an arbitrary signed-vector nullspace law with an open boundary matrix. | PARTIAL | Separated the source's ones-vector matrix identities from the project's stock-change and boundary-operator mapping. |
| Outside issuers are boundary ports under the chapter's model. | PARTIAL | Kept the internal counterpart; labeled the outside-issuer case a project open-system extension. |
| Accounting identities are distinct from behavioral determination. | SUPPORTED | Retained, while clarifying that prices may enter valuation accounting. |
| The chapter defines a Goguen Institution or surveys SFC software model categories. | UNSUPPORTED | No such claim remains. |

## Caiani et al. (2016)

Primary source: the author-hosted published article and journal DOI
`10.1016/j.jedc.2016.06.001`.

| Claim | Verdict | Resolution |
|---|---|---|
| The model combines individual balance sheets, a transaction-flow matrix, heterogeneous agents, markets, and behavioral rules. | SUPPORTED | Retained as the paper's concrete AB-SFC contribution. |
| It supplies the general zero-row/column theorem and the project's full list of boundary processes. | PARTIAL | General matrix discipline remains attributed to Godley–Lavoie; the project process list is labeled separately. |
| It derives the project's nullspace/open-boundary equation and issuer-inside/outside rule. | UNSUPPORTED | Those are explicitly project derivations and boundary semantics. |
| It demonstrates separation of accounting consistency from pricing, credit, investment, expectations, and behavioral heuristics. | SUPPORTED | Retained. |
| It surveys SFC tools or supplies an Institution/reusable conservation kernel. | UNSUPPORTED | The software-generalization was removed; the institutional layer remains explicitly project-specific. |

## Fath (2012)

Primary source: official IIASA PDF, MD5
`16A0665F1735DE3182346F7270465DC6`.

| Claim | Verdict | Resolution |
|---|---|---|
| Ecological network analysis uses compartmental storages, internal flows, and boundary inputs and outputs. | SUPPORTED | Retained as the source-backed ecological carrier. |
| Fath supplies the project's typed C/N/P/energy vocabulary, discrete `S`/`B` equation, nullspace proof, or variational laws. | UNSUPPORTED | The richer table and laws are labeled as the project's proposed encoding with separate sources where applicable. |
| Fath establishes a general limitation of ecosystem software model categories. | UNSUPPORTED | Replaced with the narrow statement that Fath's Network Environ Analysis embodies domain-specific compartment and flow semantics. |

## Ellerman (1985)

Primary source: author-hosted scan, pp. 226–233.

Short source passages:

```text
"equal debits and credits"
"not in matrix algebra"
"the Pacioli group"
"transactional zero-terms"
"balance sheet equation"
```

| Claim | Verdict | Resolution |
|---|---|---|
| Double entry directly means equal-and-opposite signed postings. | PARTIAL | The source supports equal debit and credit totals; signed-vector summation is labeled a project encoding. |
| Ellerman derives the project's matrix-nullspace/open-boundary equation. | UNSUPPORTED | The source's group construction is kept separate from the project's matrix translation and boundary semantics. |
| Valid double-entry transactions preserve an encoded accounting equation. | SUPPORTED | Retained as the source-backed algebraic claim. |
| Ellerman supplies signature-indexed models, reducts, or behavioral economics. | UNSUPPORTED | Those layers remain separate project or Caiani constructions. |
| The paper supports a Pacioli-group or incidence invariant equally. | PARTIAL | Retained only the Pacioli-group zero-term characterization; incidence is identified as a separate encoding. |
| Ellerman establishes that accounting laws are not Noether laws. | UNSUPPORTED | The classification is a cross-source conclusion using Marsden–West's variational premises. |

## Maarleveld et al. (2013)

Primary source: full text at PubMed Central, article `PMC4671265`.

Short source passages:

```text
"derived from the left null-space"
"boundary metabolites do not enter the stoichiometry matrix"
"number of atoms" ... "should balance"
"net stoichiometric coefficient"
```

| Claim | Verdict | Resolution |
|---|---|---|
| Conserved moieties are derived from the stoichiometric matrix's left nullspace. | SUPPORTED | Retained, including the `m - rank(N)` count. |
| The source defines semantic quantity kinds and a one-source/one-target unit-incidence carrier. | UNSUPPORTED | Labeled these as the project's first transfer-only typing restrictions. |
| Maarleveld's boundary metabolites are the project's open-system `B` ports. | MISATTRIBUTED | The project now explicitly distinguishes external-flow ports from fixed-concentration boundary metabolites excluded from `N`. |
| Stoichiometric coefficients must use exact rational arithmetic. | PARTIAL | Signed and fractional coefficients are source-backed; exact rational representation is labeled project policy. |
| The source supplies the discrete open-balance equation and sealed certificates. | PARTIAL | The closed left-nullspace relation is source-backed; the discrete open carrier and certificate API are labeled project extensions. |
| Stoichiometric analysis alone covers kinetic or nonlinear first integrals. | UNSUPPORTED | The specification retains Mahdi as authority for the structural-linear boundary. |
| Atom-wise C, N, and P balancing is meaningful. | SUPPORTED | Retained; separate axes, energy treatment, and explicit open ports are labeled project policies. |
| Cross-kind conversions and same-kind incidence are source requirements. | UNSUPPORTED | Labeled as project type-safety policies. |

## Completion

All eight papers cited as mathematical or domain foundations in the research
report have been graded individually and their corrections reconciled. The
remaining Hets link is an implementation reference, not a paper-derived
mathematical claim; it requires repository/API inspection before Hets informs
code, but it is outside this paper-verification pass.
