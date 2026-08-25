---
title: "A structure-preserving numerical approach for simulating algae blooms in marine water bodies of western Patagonia"
authors: "Pablo Mata Almonacid and Carolina Medel"
year: 2020
venue: "arXiv:2007.11815; submitted to Ecological Modelling"
doi_url: "https://arxiv.org/abs/2007.11815"
pages: 34
---

# A structure-preserving numerical approach for simulating algae blooms in marine water bodies of western Patagonia

## One-Sentence Summary

The paper specifies a positive two-layer NPZD bloom model, a first-order splitting-composition integrator that combines a conservative second-order modified Patankar–Runge–Kutta (MPRK) step with exact nutrient-pulse and detritus-sinking flows, and a genetic-algorithm calibration against a 2015 Puyuhuapi Fjord winter bloom. *(pp.5, 12–16)*

## Problem Addressed

Western Patagonia has sparse field data and locally unique hydrodynamics, so a deliberately medium-complexity box model is proposed for short, intense algal blooms rather than a data-hungry coupled hydrodynamic model. The implementation must nevertheless retain nonnegative biomass trajectories and account exactly for biomass gained from vertical nutrient entrainment and lost through detritus sinking. *(pp.4–7, 12–15)*

## Key Contributions

- A two-layer conceptual model whose euphotic layer contains nutrient, phytoplankton, zooplankton, and detritus, while wind-driven mixing injects nutrients from a deeper constant reservoir and sinking exports detritus. *(pp.5–8)*
- A production–destruction NPZD formulation that is positive and exactly mass-conserving before external forcing is applied. *(pp.8–10)*
- A three-map split integrator: second-order conservative MPRK for autonomous NPZD transfer, exact integration of a Gaussian nutrient pulse, and exact integration of thresholded detritus decay. *(pp.12–15)*
- An exact discrete biomass ledger and genetic-algorithm parameter-fitting formulation. *(pp.15–17)*

## Study Design

- **Type:** mathematical model, numerical-method construction, synthetic structure-preservation experiment, and empirical case-study calibration. *(pp.12–17)*
- **Population/system:** a representative 1 m² sea-surface column split into an approximately 5 m euphotic layer and a much deeper (>100 m) layer; the realistic case concerns a 2015 winter dinoflagellate bloom in an austral fjord. *(pp.6, 17)*
- **Primary endpoints:** strict positivity, exact biomass balance, calibrated NPZD trajectories, peak timing, primary-production rate, zooplankton grazing rate, and photosynthesized organic matter. *(pp.10–17)*

## Methodology

The euphotic layer is a nitrogen-based four-state box model, with all states expressed in mmol N m\(^{-3}\). Internal trophic transfers are written in production–destruction form. A Gaussian external flux augments nutrient; a piecewise-linear sinking flux removes only detritus above a nonzero floor. Operator splitting separates the conservative autonomous trophic web from those two nonconservative processes. The autonomous subflow is advanced by a two-stage MPRK solve; the forcing and sinking subflows have closed forms. A genetic algorithm maximizes a negative weighted squared-error fitness over the admissible parameter space. *(pp.8–17)*

## Model Scope and Assumptions

- The upper box represents the average behavior of a much larger marine area; detailed water-column hydrodynamics are intentionally omitted and their bloom-scale effect is summarized by an upward nutrient pulse. *(pp.6–7)*
- Initial stratification is represented without explicit halocline or thermocline depths; temperature and salinity dependence are omitted. *(pp.6–7)*
- Blooms are triggered only by nitrate increments; possible silicate and phosphorus limitation is omitted. *(p.7)*
- Each of \(P\) and \(Z\) aggregates many species and predator–prey interactions into homogeneous functional-group biomass fluxes. *(p.7)*
- The lower layer has constant physical and biological properties over the bloom time scale and no biological activity. *(pp.6–7)*

## Key Equations / Statistical Models

### State and autonomous production–destruction system

$$
\mathbf z:[0,T]\to\mathbb R_{+}^{4}
$$

where \(\mathbf z(t)=(N(t),P(t),Z(t),D(t))\), respectively nutrient, phytoplankton, zooplankton, and detritus concentrations, all in mmol N m\(^{-3}\). *(p.8, Eq.1)*

$$
\frac{d\mathbf z(t)}{dt}=\mathbf f_a(\mathbf z(t);\boldsymbol\theta)
$$

with \(\mathbf z(0)=(N(0),P(0),Z(0),D(0))\). *(p.9, Eq.2a)*

$$
f_{a_i}(\mathbf z;\boldsymbol\theta)=\sum_{j=1}^{4}\left(P_{ij}(\mathbf z;\boldsymbol\theta)-D_{ij}(\mathbf z;\boldsymbol\theta)\right),\qquad i=1,\ldots,4
$$

where \(\mathbf P\) and \(\mathbf D=\mathbf P^{\mathsf T}\) are production and destruction matrices. *(p.9, Eq.2b)*

$$
\mathbf P(\mathbf z;\boldsymbol\theta)=
\begin{bmatrix}
0 & 0 & \phi_z Z & \gamma_m D\\
J(N,I)P & 0 & 0 & 0\\
0 & G(\epsilon,g,P)Z & 0 & 0\\
0 & \phi_p P & (1-\beta)G(\epsilon,g,P)Z+\phi_z^{*}Z^2 & 0
\end{bmatrix}
$$

The nonzero entries encode nutrient uptake, grazing, zooplankton excretion, phytoplankton loss, unassimilated grazing, quadratic zooplankton loss, detritus remineralization, and their paired source/sink transfers. *(p.9, Eq.2c; p.8, Fig.2)*

$$
J(N,I)=\mu_m\frac{N}{k_N+N}\frac{I}{k_I+I}
$$

This phytoplankton growth rate is bounded above by \(\mu_m\); it approaches zero at night when \(I\approx0\). *(p.9)*

$$
G(\epsilon,g,P)=\frac{g\epsilon P^2}{g+\epsilon P^2}
$$

This Holling type III grazing-ratio function approaches the upper bound \(g\) as \(P\to\infty\). *(p.9, Eq.2d)*

$$
\boldsymbol\theta=(k_N,k_I,\mu_m,\phi_z,\phi_z^{*},\phi_p,\gamma_m,\beta,\epsilon,g,\kappa)
$$

*(p.9, Eq.2e)*

$$
0=\frac{d\mathbf z}{dt}\cdot\mathbf 1
=\frac{dN}{dt}+\frac{dP}{dt}+\frac{dZ}{dt}+\frac{dD}{dt}
$$

where \(\mathbf1=(1,1,1,1)\); hence \(N+P+Z+D\) is invariant in the autonomous model and the trajectory stays on the initial-mass manifold. *(p.10, Eq.2f)*

### Gaussian nutrient forcing and detritus sinking

$$
\frac{dN}{dt}=-J(N,I)P+\phi_z Z+\gamma_m D+I_N(t)
$$

*(p.10, Eq.3a)*

$$
I_N(t)=a\exp\!\left(-\frac{(t-b)^2}{2c^2}\right)
$$

Here \(a>0\), \(b>0\), and \(c>0\) control pulse amplitude, temporal position, and width. *(pp.10–11, Eq.3b)*

$$
\frac{dD}{dt}=\phi_pP+(1-\beta)G(\epsilon,g,P)Z+\phi_z^{*}Z^2-\gamma_mD+I_D(\mathbf z(t);\boldsymbol\theta)
$$

*(p.11, Eq.4a)*

$$
I_D(\mathbf z(t);\boldsymbol\theta)=
\begin{cases}
-\kappa\left(D(t)-D^{*}\right),&D(t)\ge D^{*},\\
0,&\text{otherwise.}
\end{cases}
$$

Here \(D^{*}>0\) is a problem-dependent detritus floor and \(\kappa>0\) is an exponential sinking/decay constant. *(p.11, Eq.4b)*

$$
\frac{d\mathbf z(t)}{dt}=\mathbf f_a(\mathbf z(t);\boldsymbol\theta)+\mathbf f_b(t)+\mathbf f_c(\mathbf z(t);\boldsymbol\theta)
$$

with positive initial states, \(\mathbf f_b(t)=(I_N(t),0,0,0)\), and \(\mathbf f_c=(0,0,0,I_D)\). *(p.11, Eqs.5a–5c)*

$$
\frac{d\mathbf z}{dt}\cdot\mathbf1=
\begin{cases}
I_N(t),&D(t)<D^{*},\\
I_N(t)-\kappa(D(t)-D^{*}),&D(t)\ge D^{*}.
\end{cases}
$$

Thus total biomass is not invariant under forcing, but its rate of change is exactly the external nutrient input minus any sinking export. Both added fluxes preserve nonnegativity. *(p.12, Eq.5d)*

### Splitting-composition numerical method

$$
\frac{d\mathbf z}{dt}=\mathbf f_a(\mathbf z;\boldsymbol\theta),\qquad
\frac{d\mathbf z}{dt}=\mathbf f_b(t),\qquad
\frac{d\mathbf z}{dt}=\mathbf f_c(\mathbf z;\boldsymbol\theta)
$$

These three subproblems separately represent conservative NPZD transfers, nutrient enrichment, and detritus sinking. *(p.12, Eq.6a)*

$$
\Phi_h(\mathbf z^0)=\left(\Phi_h^c\circ\Phi_h^b\circ\Phi_h^a\right)(\mathbf z^0)=\mathbf z(h)+\mathcal O(h^2)
$$

This is the paper's first-order one-step map. *(pp.12–13, Eq.6c)*

$$
\Phi_h(\mathbf z^0)=\left(\Phi_{h/2}^a\circ\Phi_h^b\circ\Phi_h^c\circ\Phi_{h/2}^a\right)(\mathbf z^0)
$$

The paper notes this symmetric composition as a second-order alternative for long simulations, but its numerical studies use the three-map first-order construction. *(p.13, Remark 3)*

### Autonomous MPRK substep

$$
\mathbf z^{P}=\Omega(\mathbf z^n)^{-1}\mathbf z^n
$$

*(p.13, Eq.7a)*

$$
\Omega_{ii}=1+\frac{h}{z_i^n}\sum_{j=1}^{4}D_{ij}(\mathbf z^n),\qquad i=1,\ldots,4
$$

*(p.13)*

$$
\Omega_{ij}=-\frac{h}{z_j^n}P_{ij}(\mathbf z^n),\qquad i,j=1,\ldots,4,\quad i\ne j
$$

*(p.13)*

$$
\mathbf z^{n+1}=M(\mathbf z^n,\mathbf z^P)^{-1}\mathbf z^n
$$

*(p.13, Eq.7b)*

$$
M_{ii}=1+\frac{h}{2z_i^P}\sum_{j=1}^{4}\left(D_{ij}(\mathbf z^P)+D_{ij}(\mathbf z^n)\right),\qquad i=1,\ldots,4
$$

*(p.13)*

$$
M_{ij}=-\frac{h}{2z_j^P}\left(P_{ij}(\mathbf z^P)+P_{ij}(\mathbf z^n)\right),\qquad i,j=1,\ldots,4,\quad i\ne j
$$

*(p.13)*

For every \(h>0\), positive input produces positive output, and the substep satisfies the exact conservative increment \(\sum_{i=1}^{4}(z_i^{n+1}-z_i^n)=0\). Both stages require linear solves rather than nonlinear iteration. *(p.13, Eq.7c)*

### Exact forcing and sinking substeps

$$
N^{k+1}=N^k+\int_{t^k}^{t^{k+1}}I_N(s)\,ds,\qquad P^{k+1}=P^k,\quad Z^{k+1}=Z^k,\quad D^{k+1}=D^k
$$

*(p.14, Eq.8a)*

$$
N^{k+1}=N^k+ac\sqrt{\frac{\pi}{2}}\left[
\operatorname{erf}\!\left(\frac{t^{k+1}-b}{\sqrt2c}\right)
-\operatorname{erf}\!\left(\frac{t^k-b}{\sqrt2c}\right)
\right]
$$

*(p.14, Eq.8b)*

$$
N^{k+1}=N^k,\qquad P^{k+1}=P^k,\qquad Z^{k+1}=Z^k
$$

*(p.14, Eq.9a)*

$$
D^{k+1}=
\begin{cases}
D^{*}+e^{-\kappa h}(D^k-D^{*}),&D^k\ge D^{*},\\
D^k,&\text{otherwise.}
\end{cases}
$$

The exact sinking export in one step is \(D^k-D^{k+1}\ge0\), and \(D^k\to D^{*}\) as \(k\to\infty\) when sinking remains active. *(p.14, Eq.9b)*

$$
\sum_{i=1}^{4}(z_i^{n+1}-z_i^n)=
\begin{cases}
(D^k-D^{*})(e^{-\kappa h}-1)+f(t^k,t^{k+1}),&D^k\ge D^{*},\\
f(t^k,t^{k+1}),&\text{otherwise,}
\end{cases}
$$

where \(f(t^k,t^{k+1})>0\) is the exact Gaussian increment in Eq. 8b. This is the executable discrete biomass ledger. *(p.15, Eq.10a)*

### Calibration objective

$$
\underset{\boldsymbol\theta\in\mathcal S}{\operatorname{arg\,max}}\;\Gamma(\boldsymbol\theta)
=-\sum_{i=1}^{4}w_i\left[\sum_{l=1}^{n}\left(z_i(t^l)-z_i^l(\boldsymbol\theta)\right)^2\right]
$$

where \(w_i>0\), \(\sum_iw_i=1\), observations are \(z_i(t^l)\), and numerical predictions are \(z_i^l(\boldsymbol\theta)\). The admissible space is \(\mathcal S=\mathbb R_+^7\times[0,1]\times\mathbb R_+^3\). *(p.15, Eq.11a)*

## Parameters

### General model and forcing parameters

| Name | Symbol | Units | Default | Range | Page | Notes |
|---|---:|---:|---:|---:|---:|---|
| Nutrient half-saturation factor | \(k_N\) | mmol N m\(^{-3}\) | - | \(>0\) | 10 | Nutrient-uptake saturation in \(J\). |
| Light half-saturation factor | \(k_I\) | W m\(^{-3}\) | - | \(>0\) | 10 | Paper labels it a light-absorption factor. |
| Maximum phytoplankton growth factor | \(\mu_m\) | d\(^{-1}\) | - | \(>0\) | 10 | Upper bound of \(J\). |
| Zooplankton linear loss rate | \(\phi_z\) | d\(^{-1}\) | - | \(>0\) | 10 | Transfers \(Z\) to \(N\). |
| Zooplankton quadratic loss rate | \(\phi_z^{*}\) | m³ (mmol N d)\(^{-1}\) | - | \(>0\) | 10 | Transfers \(Z^2\) to \(D\). |
| Phytoplankton linear loss rate | \(\phi_p\) | d\(^{-1}\) | - | \(>0\) | 10 | Transfers \(P\) to \(D\). |
| Detritus remineralization rate | \(\gamma_m\) | d\(^{-1}\) | - | \(>0\) | 10 | Transfers \(D\) to \(N\). |
| Zooplankton assimilation efficiency | \(\beta\) | - | - | \([0,1]\) | 10 | Fraction of grazed biomass routed to \(Z\); \(1-\beta\) goes to \(D\). |
| Grazing encounter rate | \(\epsilon\) | m⁶ (mmol N)\(^{-2}\) d\(^{-1}\) | - | \(>0\) | 10 | Holling III numerator coefficient. |
| Maximum grazing rate | \(g\) | d\(^{-1}\) | - | \(>0\) | 10 | Upper bound of \(G\). |
| Detritus sinking rate | \(\kappa\) | d\(^{-1}\) | - | \(>0\) (or 0 when disabled) | 10, 17 | Exponential decay above \(D^{*}\). |
| Nutrient-pulse amplitude | \(a\) | not stated in Table 1 | - | \(>0\) | 10–11 | Gaussian amplitude. |
| Nutrient-pulse position | \(b\) | not stated in Table 1 | - | \(>0\) | 10–11 | Time of Gaussian center. |
| Nutrient-pulse width | \(c\) | not stated in Table 1 | - | \(>0\) | 10–11 | Gaussian standard-deviation scale. |
| Detritus floor | \(D^{*}\) | mmol N m\(^{-3}\) | problem-dependent | \(>0\) | 11 | Sinking stops at this value. |
| Time step | \(h\) | d | - | \(>0\) | 12–15 | Constant on the conforming time grid. |

### Structure-preservation numerical experiment inputs (first half)

| Name | Symbol | Units | Default | Range | Page | Notes |
|---|---:|---:|---:|---:|---:|---|
| Upper-layer depth | - | m | 5 | - | 17 | Used with initial conditions from Table 2 and model parameters from Tables 3–4. |
| Pulse center | \(b\) | d | 0.5 | - | 17 | Both experiments. |
| Pulse width | \(c\) | d | 0.424 | - | 17, 20 | Fixed by a 1 d full width at half maximum; the paper's printed Eq. 12a total is inconsistent with this value. |
| Multi-pulse amplitudes | \(a_i\) | mmol N m\(^{-3}\) d\(^{-1}\) | - | 15, 18, 21, 24 | 17 | First experiment, with \(\kappa=0\). |
| Single-pulse amplitude | \(a\) | mmol N m\(^{-3}\) d\(^{-1}\) | 21 | - | 17 | Second experiment. |
| Sinking-rate sweep | \(\kappa_i\) | d\(^{-1}\) | - | 0, 0.025, 0.050, 0.100 | 17–18 | Second experiment; values are explicit in Figure 4(b). |

For adjacent amplitudes separated by 3, the injected biomass difference over the full Gaussian is

$$
\Delta N=\int_{-\infty}^{\infty}\left(I_N^{i+1}(s)-I_N^i(s)\right)ds=(a^{i+1}-a^i)c\sqrt{2\pi}=3.64\ \mathrm{mmol\ N\ m^{-3}}.
$$

*(p.17, Eq.12a)*

**Source inconsistency:** substituting the paper's repeated \(c=0.424\) and \(a^{i+1}-a^i=3\) gives \(3(0.424)\sqrt{2\pi}\approx3.188\), not the printed 3.64. A benchmark must either assert the analytic value from the stated inputs or encode 3.64 as a known source discrepancy, never use 3.64 to infer a different \(c\). *(pp.17–18, 20, Eq.12a and Fig.4)*

## Methods & Implementation Details

1. Represent \(N,P,Z,D\) as an ordered, strictly positive four-vector in common nitrogen units. *(pp.8–10)*
2. At each step, evaluate the production matrix and solve \(\Omega\mathbf z^P=\mathbf z^n\). *(p.13)*
3. Re-evaluate production/destruction at \(\mathbf z^P\), form \(M\), and solve \(M\widehat{\mathbf z}=\mathbf z^n\); this is \(\Phi_h^a\). *(p.13)*
4. Apply the exact Gaussian increment only to \(N\); this is \(\Phi_h^b\). *(p.14)*
5. Apply exact exponential sinking only to \(D\) when \(D\ge D^{*}\); this is \(\Phi_h^c\). *(p.14)*
6. Compose in the paper's written order \(\Phi_h^c\circ\Phi_h^b\circ\Phi_h^a\). *(pp.12–14)*
7. Record internal transfer conservation, injected nutrient, exported detritus, and total-state change separately; Eq. 10a must close to numerical roundoff. *(p.15)*
8. For calibration, encode each parameter vector as a GA chromosome; repeatedly evaluate fitness, test convergence, select by ranked fitness, crossover, mutate, and decode until tolerance is attained. *(p.16, Fig.3)*

## Arguments Against Prior Work

- More sophisticated marine models are discouraged by the region's data sparsity, while a simple integrator chosen without geometric care can violate the continuous system's positivity and balance. *(pp.4–5)*
- Generic parameter fitting is difficult because the parameter space can be large, topologically complex, multimodal, and sensitive to small perturbations. *(pp.4, 15)*
- Genetic algorithms avoid derivatives and work for continuous or discrete objectives, but repeated fitness evaluation may be expensive, stochastic operators do not guarantee a global optimum, and the search can linger around local optima. *(p.17)*

## Design Rationale

- Production–destruction form is chosen both to cover many conservative positive biogeochemical systems and to permit structure-preserving Patankar integration. *(p.9, Remark 1)*
- Splitting isolates conservative trophic transfers from nonconservative environmental forcing, allowing exact subflows for the latter and exact accounting of their mass effects. *(pp.12–15)*
- A nonzero \(D^{*}\) preserves a physically meaningful detritus floor; the exponential sink approaches it asymptotically without overshoot. *(pp.11, 14)*
- The first-order three-map method is adequate for short bloom simulations; the paper explicitly recommends higher-order composition for long-term studies where accumulated error matters. *(pp.12–13, Remark 3)*

## Testable Properties (from pp.6–17)

- With \(I_N=I_D=0\), \(N+P+Z+D\) remains exactly constant in continuous time. *(p.10)*
- Positive initial states remain nonnegative for the continuous production–destruction system; the MPRK substep is strictly positive for every \(h>0\). *(pp.10, 13)*
- The autonomous discrete step changes total mass by exactly zero. *(p.13)*
- The exact nutrient substep changes only \(N\), by the analytic Gaussian integral. *(p.14)*
- The sinking substep changes only \(D\), never increases it, never crosses below \(D^{*}\), and tends to \(D^{*}\) while active. *(p.14)*
- The composed step's state-sum increment equals injected nutrient minus exported detritus as Eq. 10a specifies. *(p.15)*
- \(0\le J(N,I)\le\mu_m\), \(J(N,0)=0\), and \(J\to\mu_m\) for saturating nutrient and light. *(p.9)*
- \(0\le G(\epsilon,g,P)<g\) for finite \(P\ge0\), and \(G\to g\) as \(P\to\infty\). *(p.9)*
- For the multi-amplitude experiment, each 3-unit increase in \(a\) produces the analytic Gaussian increment \(3c\sqrt{2\pi}\); the source prints 3.64 even though its stated \(c=0.424\) yields about 3.188. *(pp.17–18, 20)*

## Realistic Case-Study Geometry, Forcing, and Observations

### Euphotic-layer depth

The mean vertical irradiance profile fitted from five PAR profiles (10, 12, 14, and 16 July 2015 at 11 a.m.; the text describes five profiles despite listing four dates) is

$$
\Upsilon(d)=87.56e^{-4.881|d|}+19.31e^{-0.3952|d|},\qquad d\in[-16,0]\ \mathrm m.
$$

*(p.19, Eq.13a)*

$$
\frac{\Upsilon(d^{*})}{\Upsilon(0)}=0.025.
$$

This gives \(d^{*}\approx-5\) m, defining the euphotic layer as the depth over which at least 2.5% of surface PAR remains. *(p.19, Eq.13b)*

### Daily light forcing

At \(t=0.46\) d (11 a.m.), the depth-mean PAR is

$$
A=\frac{1}{|d^{*}|}\int_{d^{*}}^{0}\Upsilon(s)\,ds\approx12.0\ \mu\mathrm E\,\mathrm m^{-2}\,\mathrm s^{-1}.
$$

*(p.20, Eq.14a)*

$$
I(t)=
\begin{cases}
\dfrac{S}{2}\left[\sin\!\left(\dfrac{100\pi t}{21}-2\pi\right)+1\right],&0.31\le t\le0.73\ \mathrm d,\\
0,&\text{otherwise,}
\end{cases}
$$

where the surface-scale estimate \(S\approx100\ \mu\mathrm E\,\mathrm m^{-2}\,\mathrm s^{-1}\) is replaced for the upper-layer mean by \(S=15.5586=12.0/I(0.4600)\). The function is applied as a repeating daily cycle in the simulations. *(p.20, Eq.14b and Fig.6b)*

The fixed-mean-light alternatives use

$$
\int_0^1 I(s)\,ds=3.27\ \mu\mathrm E\,\mathrm m^{-2}\,\mathrm s^{-1}.
$$

*(p.23)*

### One-day nutrient mixing pulse

The idealized event reaches maximum intensity halfway through the first day, so \(b=0.5\) d. A 1 d full width at half maximum gives

$$
F_c=2\sqrt{2\ln2}\,c=1\ \mathrm d,
$$

hence \(c=0.424\) d. Requiring total entrainment \(\widehat N\approx16.00\) mmol N m\(^{-3}\) gives

$$
ac\sqrt{2\pi}=\int_{-\infty}^{\infty}I_N(s)\,ds=\widehat N,
$$

and therefore \(a\approx15.00\) mmol N m\(^{-3}\) d\(^{-1}\). *(pp.20–21, Eq.15a and Fig.6c)*

$$
\widehat N(t)=\int_{-\infty}^{t}I_N(s)\,ds.
$$

*(p.21, Eq.15b)*

### Measurement conversions

- Nitrate is the sole modeled nutrient; discrete observations were available at 2, 5, and 15 m. *(p.21)*
- Chlorophyll-a is the phytoplankton proxy. Surface records and profiles at 2, 5, and 15 m are converted with C:Chl-a = 50 and C:N = 7.6. *(p.21)*
- Zooplankton observations at 2, 5, and 15 m are converted using 56 \(\mu\)g C per individual and C:N = 8.6. *(p.21)*
- Detritus represents POM plus DOM. DOC is the available proxy for autochthonous DOM, converted with DOC:DON = 10.4; no POM data were available. *(p.21)*
- July 8 phytoplankton is extrapolated from July 10–16; July 8 nutrient comes from prior winters; July 8 zooplankton and DOC are unavailable. Detritus is held at \(D=20.631\) mmol N m\(^{-3}\) for interpolation/calibration. *(pp.21–22)*

### Table 2 observations and initial conditions

| Date | Time (d) | N (mmol N m\(^{-3}\)) | P (mmol N m\(^{-3}\)) | Z (mmol N m\(^{-3}\)) | D (mmol N m\(^{-3}\)) | Page |
|---|---:|---:|---:|---:|---:|---:|
| July 6 | 0.0 | 1.000 | 1.500 | 0.100 | 20.631 | 22 |
| July 8 | 1.5 | 11.200 | 2.642 | - | - | 22 |
| July 10 | 3.5 | 5.827 | 5.908 | 0.788 | 15.899 | 22 |
| July 12 | 5.5 | 2.181 | 3.439 | 4.871 | 28.314 | 22 |
| July 14 | 7.5 | 1.831 | 3.135 | 1.484 | 12.882 | 22 |
| July 16 | 9.5 | 3.752 | 5.191 | 0.469 | 25.429 | 22 |

The executable initial state is therefore \(\mathbf z(0)=(1.000,1.500,0.100,20.631)\) mmol N m\(^{-3}\). *(p.22, Table 2)*

The interpolation polynomials plotted against the data are

$$
N(t)=1.0+16.9453t-9.28t^2+1.93t^3-0.18t^4+0.0063t^5,
$$

$$
P(t)=1.5-3.34t+4.50t^2-1.42t^3+0.17t^4-0.007t^5,
$$

$$
Z(t)=-53.93+28.23t-4.32t^2+0.205t^3.
$$

The \(Z\) polynomial is a local interpolation over its observed support and must not be evaluated at \(t=0\), where it is negative and conflicts with the explicit initial condition. *(p.22, Fig.7)*

## Genetic-Algorithm Calibration Configuration

The fitness weights for \((N,P,Z,D)\) are

$$
(w_1,w_2,w_3,w_4)=(0.10,0.40,0.49,0.01).
$$

*(p.21, Eq.17a)*

- Simulation interval: stated as \([0,9]\) d with 100 time steps, so \(h=0.09\) d if 100 equal steps are intended. *(p.22)*
- Code: object-oriented Fortran 2003 using the open-source Pikaia GA routine. *(p.22)*
- Initial population: 1000 parameter chromosomes, randomly sampled from \(\mathcal S\). *(p.22)*
- Stop condition: fitness convergence tolerance \(10^{-6}\) or 10,000 generations. *(p.22)*
- Crossover probability: \(p_{\mathrm{cross}}=0.95\). *(p.22)*
- Initial mutation rate: \(p_{\mathrm{mut}}=0.005\), adjusted according to fitness. *(p.22)*
- Reproduction: always replace the worst individual. *(p.22)*

The four cases are: TLLZ (time-varying light, linear \(Z\) loss), TLQZ (time-varying light, quadratic \(Z\) loss), MLLZ (mean light, linear \(Z\) loss), and MLQZ (mean light, quadratic \(Z\) loss). *(pp.22–23)*

### Calibrated parameter sets (Tables 3–4)

| Case | \(k_N\) | \(k_I\) | \(\mu_m\) | \(\phi_z\) | \(\gamma_m\) | \(\phi_p\) | \(\epsilon\) | Page |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| TLLZ | 0.00459 | 0.04515 | 1.07718 | 0.33840 | 0.00003 | 0.17056 | 0.03995 | 24 |
| TLQZ | 0.86336 | 0.05112 | 0.94848 | 0.10830 | 0.00005 | 0.08091 | 0.02791 | 24 |
| MLLZ | 0.00138 | 9.19370 | 1.93210 | 0.46930 | 0.01245 | 0.25602 | 0.04509 | 24 |
| MLQZ | 0.12540 | 11.1625 | 2.62672 | 0.24589 | 0.03842 | 0.31866 | 0.03423 | 24 |

| Case | \(g\) | \(\beta\) | \(\phi_z^{*}\) | \(\kappa\) | \(\Gamma(\theta)\) | Page |
|---|---:|---:|---:|---:|---:|---:|
| TLLZ | 17.8036 | 0.97842 | - | - | -62.92405 | 24 |
| TLQZ | 26.8129 | 0.99702 | 0.05820 | - | -45.69100 | 24 |
| MLLZ | 17.7468 | 0.99007 | - | - | -105.6393 | 24 |
| MLQZ | 29.7383 | 0.99671 | 0.07773 | - | -95.78919 | 24 |

Quadratic-loss cases fit better than corresponding linear-loss cases, and detailed time-varying light fits better than daily mean light. The four optima differ substantially, demonstrating nonuniqueness/local maxima; \(\mu_m,\epsilon,g,\beta\), and \(\phi_z^{*}\) are described as relatively stable compared with other parameters. *(pp.23–25, Figs.8–9 and Tables 3–4)*

## Sequence-of-Pulses Benchmark

Use the TLQZ parameter set except set \(\gamma_m=10^{-4}\) d\(^{-1}\) and \(\beta=0.75\). The three Gaussian pulses are: \((a,b,c)=(15,0.5,0.4)\), \((18,9.0,0.3)\), and \((8,17.0,0.4)\), with time in days and the amplitude units implied by Eq. 3b. *(pp.25–26, Fig.10)*

Each pulse produces the qualitative sequence \(N\to P\to Z\): the phytoplankton peak follows the nutrient peak by about three days and the zooplankton peak follows roughly two days later. Detritus accumulates as the long-term organic-matter reservoir. *(pp.25–26, Fig.10)*

$$
PP(t)=\int_0^t J(N(s),I(s))P(s)\,ds,
$$

*(p.25, Eq.18a)*

$$
PP^k=\frac h2\sum_{i=2}^{k}\left[J(N^{i-1},I(t^{i-1}))P^{i-1}+J(N^i,I(t^i))P^i\right]\approx PP(t^k).
$$

The identical trapezoidal accumulation with \(G(\epsilon,g,P)Z\) gives total grazing. Any production-matrix flux can be integrated similarly to estimate transfer between groups. *(p.25, Eq.18b)*

The paper reports \(PP(25)=24.38\) mmol N m\(^{-3}\). For a 1 km² area and 5 m euphotic depth, this is \(24.38\times5\times10^6=121.9\times10^6\) mmol N. *(pp.26–27, Fig.11)*

## Figure 4 Acceptance Evidence

- **Figure 4(a): Gaussian-only mass increment.** With \(\kappa=0\), each total-biomass curve rises during the centered pulse and then becomes exactly horizontal. Runs with \(a=15,18,21,24\), common \(b=0.5\) and \(c=0.424\), retain constant pairwise plateau separations after forcing ends. This tests the exact Gaussian update plus zero autonomous mass drift. *(pp.17–18, Fig.4a)*
- **Figure 4(b): sinking monotonicity.** With a single \(a=21\) pulse, \(\kappa=0\) reaches a constant plateau; \(\kappa=0.025,0.050,0.100\) share the same initial rise and then decay increasingly rapidly in that order. At any common late time the expected ordering is \(M_{0}\ge M_{0.025}\ge M_{0.050}\ge M_{0.100}\). *(pp.17–18, Fig.4b)*
- **Ledger acceptance:** for every step, compare \(\Delta(N+P+Z+D)\) with the analytic Gaussian increment plus \((D^k-D^{*})(e^{-\kappa h}-1)\) when sinking is active. The residual should be at linear-solver/roundoff tolerance, not time-discretization tolerance. *(pp.14–15, Eq.10a)*
- **Do not digitize a false target:** Figure 4 is qualitative/structural evidence. Its source text contains the Eq. 12a 3.64 inconsistency, so analytic increments from \((a,b,c)\) are the authoritative executable target. *(pp.17–18)*

## Figures of Interest

- **Fig.1 (p.6):** two-layer bloom sequence: initial stratification, wind-driven mixing and trophic transfer, then freshwater-driven restratification.
- **Fig.2 (p.8):** complete directed NPZD flux graph, including \(I_N\) and \(I_D\).
- **Fig.3 (p.16):** GA flowchart and probabilities used by the calibration protocol.
- **Fig.4 (p.18):** structure-preserving total-biomass acceptance evidence.
- **Fig.6 (p.20):** fitted light profile, scaled daylight curve, Gaussian pulse and cumulative nutrient input.
- **Fig.7 (p.22):** observations and interpolation polynomials used for calibration.
- **Figs.8–9 (pp.23–24):** fit comparisons showing benefits of quadratic zooplankton loss and time-varying light.
- **Fig.10 (p.26):** three-pulse state response and detritus accumulation.
- **Fig.11 (p.27):** instantaneous and cumulative primary-production/grazing fluxes.

## Results Summary

- The discrete method is unconditionally positive for the autonomous transfer and exactly accounts for external mass gain/loss under composition. *(pp.13–15)*
- TLQZ is the best of the four calibrated cases (\(\Gamma=-45.69100\)); MLQZ improves over MLLZ but remains worse than either detailed-light fit. *(pp.23–24)*
- Parameter calibration is nonunique: several distinct parameter combinations fit the observations reasonably, and a 1000-member initial population sparsely samples a roughly 10-parameter space. *(pp.24–25)*
- Three forcing pulses produce repeated nutrient, phytoplankton, and zooplankton peaks plus progressive detritus storage and 24.38 mmol N m\(^{-3}\) cumulative primary production by day 25. *(pp.25–27)*

## Limitations

- Field-data sparsity motivates the reduced model but also limits reliable calibration; some observations are extrapolated, borrowed from prior winters, missing, or represented by constant detritus. *(pp.5, 21–22, 28)*
- The upper-layer box omits explicit hydrodynamics, temperature, salinity, other limiting nutrients, species structure, and lower-layer biology. *(p.7)*
- The GA does not identify a unique optimum and may be trapped near local maxima; parameter values depend strongly on the forcing representation. *(pp.17, 23–25, 28)*
- The paper does not state calibrated \(\kappa\) or \(D^{*}\) for the realistic case, so sinking cannot be independently reproduced there without an explicit benchmark choice. *(pp.11, 24)*
- The stated calibration interval \([0,9]\) d does not cover the final Table 2 time at 9.5 d, and the exact treatment of that point is unspecified. *(p.22)*
- Eq. 12a's printed 3.64 increment conflicts with the stated \(c=0.424\). *(pp.17–18, 20)*

## Independent Executable Benchmark Specification

1. **Core flux benchmark:** implement Eqs. 2b–2e exactly, with \(\mathbf D=\mathbf P^{\mathsf T}\). For randomized positive states/parameters, assert internal derivative sum is zero and each paired transfer is equal/opposite. *(pp.9–10)*
2. **MPRK benchmark:** run the two linear solves in Eqs. 7a–7b for randomized positive states and multiple \(h\), asserting strict positivity and zero state-sum drift to solver tolerance. *(p.13)*
3. **Analytic subflow benchmark:** compare nutrient and detritus updates against Eqs. 8b and 9b, including \(D<D^{*}\), \(D=D^{*}\), and \(D>D^{*}\). *(p.14)*
4. **Composition ledger benchmark:** verify Eq. 10a step by step and cumulatively over Figure 4 scenarios. *(pp.15, 17–18)*
5. **Figure 4 scenarios:** initial state and model parameters come from Tables 2–4 as the paper directs; use \(b=0.5,c=0.424\), amplitudes \(15,18,21,24\) with \(\kappa=0\), then \(a=21\) with \(\kappa=0,0.025,0.05,0.1\). Accept plateau invariance and ordered late-time decay, with the analytic Gaussian integral governing exact differences. *(pp.17–18)*
6. **Case-study forcing benchmark:** assert \(d^{*}\approx-5\) from Eqs. 13a–13b; \(I(0.46)\approx12\) with \(S=15.5586\); and total nutrient input near 16 for \((a,b,c)=(15,0.5,0.424)\). *(pp.19–21)*
7. **TLQZ trajectory benchmark:** use \(\mathbf z_0=(1,1.5,0.1,20.631)\), the Table 3–4 TLQZ parameters, time-varying daily light, prescribed one-day pulse, \(h=0.09\) if interpreting 100 equal steps on \([0,9]\), and no sinking. Compare against the qualitative curves/fitness rather than requiring digit-for-digit calibration, because GA seed and exact observation handling are unspecified. *(pp.21–24)*
8. **Three-pulse benchmark:** use TLQZ with \(\gamma_m=10^{-4}\), \(\beta=0.75\), the three Fig.10 pulses, and assert peak order \(N\to P\to Z\), nondecreasing detritus without sinking, and \(PP(25)\approx24.38\). *(pp.25–27)*

## Relevance to Project

This paper supplies a compact ecosystem-model benchmark whose internal transfers, nonnegative-state behavior, external forcing, exported mass, and cumulative ecological production all have independently testable ledgers. It is especially useful for validating an ecosystem simulator's conservation architecture and forcing semantics before adding spatial hydrodynamics or a larger food web.

## Open Questions

- [ ] Which calibrated parameter set did the authors intend for Figure 4, given that p.17 points generically to Tables 3–4? *(p.17)*
- [ ] What \(D^{*}\) was used in Figure 4(b), and was sinking applied before or after the exact forcing exactly as Eq. 6c states? *(pp.12, 14, 17–18)*
- [ ] Is Eq. 12a's 3.64 a typo, or was a different unreported \(c\) used for Figure 4? *(pp.17–18, 20)*
- [ ] Was the July 16 observation at 9.5 d excluded, clipped, or otherwise transformed for the stated \([0,9]\) d calibration? *(p.22)*
- [ ] Is the daily light function evaluated with time modulo one day? The multi-day figures imply repetition, but the implementation convention is not written explicitly. *(pp.20, 23, 26–27)*

## Related Work Worth Reading

- Burchard, Deleersnijder, and Meister (2003), and Kopecz and Meister (2018), for conservative Patankar and MPRK order conditions. *(pp.28, 33; refs.1, 67)*
- Heinle and Slawig (2013) on NPZD internal dynamics and parameter-choice sensitivity. *(pp.32–33; refs.61, 75)*
- Fasham, Ducklow, and McKelvie (1990), the nitrogen-based plankton-dynamics foundation. *(p.34; ref.82)*
- Montero et al. (2017), the empirical winter-bloom case that supplies the field context and much of the calibration data. *(pp.28; ref.2)*
- Charbonneau and Knapp (1995), the PIKAIA user guide used for the GA implementation. *(p.33; ref.74)*

## Reading Provenance

Extracted from arXiv:2007.11815v1 (`paper.pdf`, 34 pages). All rendered page images `pngs/page-000.png` through `pngs/page-033.png` were visually inspected at the paper-reader house render settings (150 dpi) on 2026-08-25.

## Collection Cross-References

### Already in Collection

- (none found among explicit citations)

### New Leads (Not Yet in Collection)

- Burchard, Deleersnijder, and Meister (2003), "A high-order conservative Patankar-type discretisation for stiff systems of production-destruction equations" - foundational conservative Patankar method used by the autonomous substep.
- Kopecz and Meister (2018), "On order conditions for modified Patankar-Runge-Kutta schemes" - formal MPRK order, positivity, and conservation conditions.
- Montero et al. (2017), "A winter dinoflagellate bloom drives high rates of primary production in a Patagonian fjord ecosystem" - empirical source for the Puyuhuapi bloom and its field observations.
- Heinle and Slawig (2013), "Internal dynamics of NPZD type ecosystem models" - mathematical NPZD behavior underlying the chosen trophic web.
- Fasham, Ducklow, and McKelvie (1990), "A nitrogen-based model of plankton dynamics in the oceanic mixed layer" - nitrogen-based NPZD model lineage.

### Supersedes or Recontextualizes

- (none)

### Conceptual Links (not citation-based)

- [Conservation Laws in Biochemical Reaction Networks](../Mahdi_2017_ConservationLawsBiochemicalReaction/notes.md) - Mahdi's left-nullspace/compatibility-class account gives the general algebraic interpretation of the NPZD total-mass invariant, while this paper supplies a positive integrator that preserves that invariant discretely.
- [Ecopath with Ecosim: Methods, Capabilities and Limitations](../Christensen_2004_EcopathEcosimMethodsCapabilities/notes.md) - both separate internal trophic transfers from boundary forcing and encounter nonunique fitted parameter sets; this paper adds an explicit production-destruction ODE and exact stepwise mass ledger.
- [Testing Ecological Models: The Meaning of Validation](../Rykiel_1996_TestingEcologicalModelsValidation/notes.md) - the paper's conclusion calls the realistic case "validated," but the same observations are used for calibration; under Rykiel's definitions this is calibration/fit evidence, not independent empirical validation.
- [The Trophic-Dynamic Aspect of Ecology](../Lindeman_1942_TrophicDynamicAspectEcology/notes.md) - Lindeman's open trophic-compartment ledger is instantiated here as explicit NPZD transfer, nutrient-input, and sinking-output rates.

### Cited By (in Collection)

- (none found)
