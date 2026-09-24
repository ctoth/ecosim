//! Ecosim's conserved currencies and the registry that declares them.
//!
//! A kind is a `Copy` handle to a name leaked once per process: literals are
//! `'static` already, and each distinct data-defined name (a trophic spec's
//! tracer kind) is interned on first declaration and never freed. The leak is
//! bounded by the number of distinct names, and it is what makes handles `Copy`
//! and `'static` (Q, conservation#6).

use std::collections::BTreeSet;
use std::fmt;
use std::sync::{Mutex, PoisonError};

use conservation_core::{Affine, IdentifierError, Kind, KindRegistry, nonblank};
use num_rational::BigRational;
use num_traits::Zero;

/// A conserved ecosim currency. Equality, order and hashing are by name, the
/// identity conservation's former string kind identifier had, so every ordered
/// collection keeps its order.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EcosimKind(&'static str);

impl EcosimKind {
    /// The material-equivalent currency of the food-web and trophic stocks.
    pub const MATERIAL: Self = Self("material-equivalent");
    /// The energy currency of the two-compartment `World`.
    pub const ENERGY: Self = Self("energy");
    const LITERALS: [Self; 2] = [Self::ENERGY, Self::MATERIAL];

    /// The registry name.
    pub fn name(self) -> &'static str {
        self.0
    }
}

impl fmt::Display for EcosimKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

/// Every ecosim currency is an amount: linear, and never below zero.
impl Kind for EcosimKind {
    fn affine(self) -> Affine<Self> {
        Affine::Linear
    }
    fn floor(self) -> Option<BigRational> {
        Some(BigRational::zero())
    }
}

static DECLARED: Mutex<BTreeSet<&'static str>> = Mutex::new(BTreeSet::new());

/// Ecosim's kind registry: the two literals plus every declared data name.
#[derive(Clone, Copy, Debug, Default)]
pub struct EcosimKinds;

impl EcosimKinds {
    /// Declares a kind by name, returning the one handle for that name.
    pub fn declare(name: &str) -> Result<EcosimKind, IdentifierError> {
        nonblank(name)?;
        if let Some(literal) = EcosimKind::LITERALS.into_iter().find(|kind| kind.0 == name) {
            return Ok(literal);
        }
        let mut declared = DECLARED.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(existing) = declared.get(name) {
            return Ok(EcosimKind(existing));
        }
        let leaked: &'static str = Box::leak(Box::<str>::from(name));
        declared.insert(leaked);
        Ok(EcosimKind(leaked))
    }
}

impl KindRegistry for EcosimKinds {
    type Kind = EcosimKind;
    fn resolve(&self, name: &str) -> Option<EcosimKind> {
        EcosimKind::LITERALS
            .into_iter()
            .find(|kind| kind.0 == name)
            .or_else(|| {
                DECLARED
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .get(name)
                    .map(|existing| EcosimKind(existing))
            })
    }
}
