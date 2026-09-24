//! How ecosim processes settle in conservation-dynamics.

use std::collections::BTreeSet;

use conservation_dynamics::{FlowSpec, ProcessDefinition, Rationing};

use crate::kinds::EcosimKind;

/// Declares every process with a flow in `flows` `Rationing::Ration`.
///
/// All ecosim mechanisms propose from one pre-step state; competing withdrawals
/// from a stock share the proportional limit (available − floor) / withdrawal.
/// This is the one place ecosim states that.
pub(crate) fn rationed(flows: &[FlowSpec<EcosimKind>]) -> Vec<ProcessDefinition> {
    flows
        .iter()
        .map(|flow| flow.process.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|id| ProcessDefinition {
            id,
            rationing: Rationing::Ration,
        })
        .collect()
}
