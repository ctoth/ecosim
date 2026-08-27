//! Domain-owned exact kinetic and source-allocation sentences.

use std::error::Error;
use std::fmt;

use num_rational::BigRational;
use num_traits::{Signed, Zero};

use crate::{ExactTrophicTransition, TrophicFlow};

/// One exact constitutive rule over a pre-transition ecosystem state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KineticLaw {
    /// Nutrient-limited producer growth.
    ProducerGrowth {
        /// Producer stock and growth-flow suffix.
        producer: String,
        /// Exact maximum growth rate.
        maximum: BigRational,
        /// Exact positive nutrient half-saturation.
        half_saturation: BigRational,
    },
    /// Holling-type feeding proposal before assimilation partitioning.
    Feeding {
        /// Consumer stock.
        consumer: String,
        /// Resource stock.
        resource: String,
        /// Exact maximum feeding rate.
        maximum: BigRational,
        /// Exact positive resource half-saturation.
        half_saturation: BigRational,
    },
    /// Biomass-proportional mortality proposal.
    Mortality {
        /// Stock whose mortality flow is proposed.
        stock: String,
        /// Exact nonnegative mortality rate.
        rate: BigRational,
    },
    /// Detritus-proportional decomposition proposal.
    Decomposition {
        /// Exact nonnegative decomposition rate.
        rate: BigRational,
    },
}

/// Named exact kinetic sentence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KineticSentence {
    name: String,
    law: KineticLaw,
}

impl KineticSentence {
    /// Validates a domain-owned kinetic sentence.
    pub fn new(name: impl Into<String>, law: KineticLaw) -> Result<Self, KineticError> {
        let name = nonblank("kinetic sentence", name.into())?;
        match &law {
            KineticLaw::ProducerGrowth {
                producer,
                maximum,
                half_saturation,
            } => {
                nonblank_ref("producer", producer)?;
                valid_saturation(maximum, half_saturation)?;
            }
            KineticLaw::Feeding {
                consumer,
                resource,
                maximum,
                half_saturation,
            } => {
                nonblank_ref("consumer", consumer)?;
                nonblank_ref("resource", resource)?;
                valid_saturation(maximum, half_saturation)?;
            }
            KineticLaw::Mortality { stock, rate } => {
                nonblank_ref("mortality stock", stock)?;
                valid_rate(rate)?;
            }
            KineticLaw::Decomposition { rate } => valid_rate(rate)?,
        }
        Ok(Self { name, law })
    }

    /// Stable law name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Constitutive rule.
    pub fn law(&self) -> &KineticLaw {
        &self.law
    }

    /// Checks every recorded proposal against the pre-transition state and elapsed time.
    pub fn evaluate(
        &self,
        transitions: &[ExactTrophicTransition],
    ) -> Result<KineticVerdict, KineticError> {
        if transitions.is_empty() {
            return Err(KineticError::TooShort);
        }
        for transition in transitions {
            let (flow, observed, expected) = self.observation(transition)?;
            if observed != expected {
                return Ok(KineticVerdict::Violated(KineticViolation {
                    sentence: self.name.clone(),
                    transition: transition.index(),
                    flow,
                    observed,
                    expected,
                }));
            }
        }
        Ok(KineticVerdict::Satisfied(KineticWitness {
            sentence: self.name.clone(),
            transitions_checked: transitions.len(),
        }))
    }

    fn observation(
        &self,
        transition: &ExactTrophicTransition,
    ) -> Result<(TrophicFlow, BigRational, BigRational), KineticError> {
        match &self.law {
            KineticLaw::ProducerGrowth {
                producer,
                maximum,
                half_saturation,
            } => {
                let flow = TrophicFlow::ProducerGrowth(producer.clone());
                let observed = proposal(transition, &flow)?;
                let nutrient = stock(transition, "nutrient")?;
                let actor = stock(transition, producer)?;
                let expected = saturating(
                    maximum,
                    nutrient,
                    half_saturation,
                    actor,
                    transition.elapsed(),
                );
                Ok((flow, observed, expected))
            }
            KineticLaw::Feeding {
                consumer,
                resource,
                maximum,
                half_saturation,
            } => {
                let assimilation = TrophicFlow::FeedingAssimilation {
                    consumer: consumer.clone(),
                    resource: resource.clone(),
                };
                let waste = TrophicFlow::FeedingWaste {
                    consumer: consumer.clone(),
                    resource: resource.clone(),
                };
                let observed = proposal(transition, &assimilation)? + proposal(transition, &waste)?;
                let expected = saturating(
                    maximum,
                    stock(transition, resource)?,
                    half_saturation,
                    stock(transition, consumer)?,
                    transition.elapsed(),
                );
                Ok((assimilation, observed, expected))
            }
            KineticLaw::Mortality { stock: name, rate } => {
                let flow = TrophicFlow::Mortality(name.clone());
                let observed = proposal(transition, &flow)?;
                let expected = rate * stock(transition, name)? * transition.elapsed();
                Ok((flow, observed, expected))
            }
            KineticLaw::Decomposition { rate } => {
                let flow = TrophicFlow::Decomposition;
                let observed = proposal(transition, &flow)?;
                let expected = rate * stock(transition, "detritus")? * transition.elapsed();
                Ok((flow, observed, expected))
            }
        }
    }
}

/// Positive exact kinetic evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KineticWitness {
    /// Sentence name.
    pub sentence: String,
    /// Number of accepted transitions checked.
    pub transitions_checked: usize,
}

/// First exact kinetic mismatch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KineticViolation {
    /// Sentence name.
    pub sentence: String,
    /// First mismatching transition.
    pub transition: usize,
    /// Semantic process flow.
    pub flow: TrophicFlow,
    /// Recorded proposal.
    pub observed: BigRational,
    /// Proposal recomputed from pre-state and elapsed time.
    pub expected: BigRational,
}

/// Typed exact kinetic result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KineticVerdict {
    /// All proposals matched.
    Satisfied(KineticWitness),
    /// First proposal mismatch.
    Violated(KineticViolation),
}

/// Exact proportional-settlement sentence for every withdrawal sharing one source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AllocationSentence {
    name: String,
    source: String,
    withdrawals: Vec<TrophicFlow>,
}

impl AllocationSentence {
    /// Constructs a complete named source-withdrawal group.
    pub fn new(
        name: impl Into<String>,
        source: impl Into<String>,
        withdrawals: impl IntoIterator<Item = TrophicFlow>,
    ) -> Result<Self, KineticError> {
        let name = nonblank("allocation sentence", name.into())?;
        let source = nonblank("allocation source", source.into())?;
        let mut withdrawals = withdrawals.into_iter().collect::<Vec<_>>();
        withdrawals.sort();
        if withdrawals.is_empty() {
            return Err(KineticError::EmptyWithdrawalGroup);
        }
        if withdrawals.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(KineticError::DuplicateWithdrawal);
        }
        Ok(Self {
            name,
            source,
            withdrawals,
        })
    }

    /// Checks bounds, zero-source behavior, and one common exact scale.
    pub fn evaluate(
        &self,
        transitions: &[ExactTrophicTransition],
    ) -> Result<AllocationVerdict, KineticError> {
        if transitions.is_empty() {
            return Err(KineticError::TooShort);
        }
        for transition in transitions {
            let available = stock(transition, &self.source)?;
            let mut scale = None;
            for flow in &self.withdrawals {
                let proposed = proposal(transition, flow)?;
                let settled = settled(transition, flow)?;
                if settled.is_negative() || settled > proposed {
                    return Ok(AllocationVerdict::Violated(AllocationViolation {
                        sentence: self.name.clone(),
                        transition: transition.index(),
                        flow: flow.clone(),
                        proposed,
                        settled,
                        reason: AllocationViolationReason::OutOfBounds,
                    }));
                }
                if available.is_zero() && !settled.is_zero() {
                    return Ok(AllocationVerdict::Violated(AllocationViolation {
                        sentence: self.name.clone(),
                        transition: transition.index(),
                        flow: flow.clone(),
                        proposed,
                        settled,
                        reason: AllocationViolationReason::NonzeroFromEmptySource,
                    }));
                }
                if !proposed.is_zero() {
                    let current = &settled / &proposed;
                    if scale.as_ref().is_some_and(|expected| expected != &current) {
                        return Ok(AllocationVerdict::Violated(AllocationViolation {
                            sentence: self.name.clone(),
                            transition: transition.index(),
                            flow: flow.clone(),
                            proposed,
                            settled,
                            reason: AllocationViolationReason::DifferentScale,
                        }));
                    }
                    scale = Some(current);
                }
            }
        }
        Ok(AllocationVerdict::Satisfied(AllocationWitness {
            sentence: self.name.clone(),
            transitions_checked: transitions.len(),
        }))
    }
}

/// Positive exact allocation evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AllocationWitness {
    /// Sentence name.
    pub sentence: String,
    /// Number of accepted transitions checked.
    pub transitions_checked: usize,
}

/// Reason a source-allocation sentence failed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AllocationViolationReason {
    /// Settled was negative or exceeded proposed.
    OutOfBounds,
    /// A zero source produced a positive settlement.
    NonzeroFromEmptySource,
    /// Competing positive proposals did not receive one common scale.
    DifferentScale,
}

/// First exact allocation failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AllocationViolation {
    /// Sentence name.
    pub sentence: String,
    /// First offending transition.
    pub transition: usize,
    /// First offending flow.
    pub flow: TrophicFlow,
    /// Proposed withdrawal.
    pub proposed: BigRational,
    /// Settled withdrawal.
    pub settled: BigRational,
    /// Failed allocation property.
    pub reason: AllocationViolationReason,
}

/// Typed exact allocation result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AllocationVerdict {
    /// All source groups settled lawfully.
    Satisfied(AllocationWitness),
    /// First allocation failure.
    Violated(AllocationViolation),
}

/// Structural kinetic/allocation error.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum KineticError {
    /// Required name is blank.
    Blank(&'static str),
    /// A rate is negative.
    NegativeRate,
    /// A saturation denominator parameter is not positive.
    NonpositiveHalfSaturation,
    /// No accepted transition can produce evidence.
    TooShort,
    /// A stock is absent from the compiled transition.
    UnknownStock(String),
    /// A semantic flow is absent from the compiled transition.
    UnknownFlow(TrophicFlow),
    /// A source group contains no withdrawals.
    EmptyWithdrawalGroup,
    /// A source group repeats a flow.
    DuplicateWithdrawal,
}

impl fmt::Display for KineticError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Blank(field) => write!(formatter, "{field} must be nonblank"),
            Self::NegativeRate => formatter.write_str("kinetic rates must be nonnegative"),
            Self::NonpositiveHalfSaturation => {
                formatter.write_str("half-saturation must be positive")
            }
            Self::TooShort => formatter.write_str("kinetic evidence requires one transition"),
            Self::UnknownStock(stock) => write!(formatter, "unknown kinetic stock: {stock}"),
            Self::UnknownFlow(flow) => write!(formatter, "unknown kinetic flow: {flow:?}"),
            Self::EmptyWithdrawalGroup => formatter.write_str("withdrawal group must be nonempty"),
            Self::DuplicateWithdrawal => {
                formatter.write_str("withdrawal group contains a duplicate")
            }
        }
    }
}

impl Error for KineticError {}

fn valid_rate(rate: &BigRational) -> Result<(), KineticError> {
    if rate.is_negative() {
        Err(KineticError::NegativeRate)
    } else {
        Ok(())
    }
}

fn valid_saturation(maximum: &BigRational, half: &BigRational) -> Result<(), KineticError> {
    valid_rate(maximum)?;
    if half <= &BigRational::zero() {
        Err(KineticError::NonpositiveHalfSaturation)
    } else {
        Ok(())
    }
}

fn nonblank(field: &'static str, value: String) -> Result<String, KineticError> {
    nonblank_ref(field, &value)?;
    Ok(value)
}

fn nonblank_ref(field: &'static str, value: &str) -> Result<(), KineticError> {
    if value.trim().is_empty() {
        Err(KineticError::Blank(field))
    } else {
        Ok(())
    }
}

fn stock<'a>(
    transition: &'a ExactTrophicTransition,
    name: &str,
) -> Result<&'a BigRational, KineticError> {
    transition
        .stock_before(name)
        .ok_or_else(|| KineticError::UnknownStock(name.to_owned()))
}

fn proposal(
    transition: &ExactTrophicTransition,
    flow: &TrophicFlow,
) -> Result<BigRational, KineticError> {
    transition
        .proposed(flow)
        .cloned()
        .ok_or_else(|| KineticError::UnknownFlow(flow.clone()))
}

fn settled(
    transition: &ExactTrophicTransition,
    flow: &TrophicFlow,
) -> Result<BigRational, KineticError> {
    transition
        .settled(flow)
        .cloned()
        .ok_or_else(|| KineticError::UnknownFlow(flow.clone()))
}

fn saturating(
    maximum: &BigRational,
    resource: &BigRational,
    half: &BigRational,
    actor: &BigRational,
    elapsed: &BigRational,
) -> BigRational {
    if resource.is_zero() || actor.is_zero() || maximum.is_zero() || elapsed.is_zero() {
        return BigRational::zero();
    }
    maximum * resource / (half + resource) * actor * elapsed
}
