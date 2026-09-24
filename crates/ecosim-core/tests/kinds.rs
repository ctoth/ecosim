use conservation_core::{Affine, IdentifierError, Kind, KindRegistry};
use ecosim_core::{EcosimKind, EcosimKinds};
use num_rational::BigRational;
use num_traits::Zero;

#[test]
fn ecosim_kinds_are_linear_with_floor_zero() {
    let declared = EcosimKinds::declare("kinds-test-carbon").unwrap();
    for kind in [EcosimKind::MATERIAL, EcosimKind::ENERGY, declared] {
        assert_eq!(kind.affine(), Affine::Linear, "{kind}");
        assert_eq!(kind.floor(), Some(BigRational::zero()), "{kind}");
        assert_eq!(kind.difference(), kind, "{kind}");
    }
}

#[test]
fn declaring_a_name_twice_returns_one_handle() {
    let a = EcosimKinds::declare("kinds-test-twice").unwrap();
    let b = EcosimKinds::declare("kinds-test-twice").unwrap();
    assert_eq!(a, b);
    assert!(std::ptr::eq(a.name(), b.name()));
    assert_eq!(
        EcosimKinds::declare("material-equivalent").unwrap(),
        EcosimKind::MATERIAL
    );
}

#[test]
fn registry_resolves_declared_names_and_literals() {
    assert_eq!(EcosimKinds.resolve("energy"), Some(EcosimKind::ENERGY));
    let sulfur = EcosimKinds::declare("kinds-test-sulfur").unwrap();
    assert_eq!(EcosimKinds.resolve("kinds-test-sulfur"), Some(sulfur));
    assert_eq!(EcosimKinds.resolve("kinds-test-never-declared"), None);
    assert_eq!(EcosimKinds::declare(""), Err(IdentifierError::Blank));
    assert_eq!(EcosimKinds::declare("  "), Err(IdentifierError::Blank));
    assert_eq!(sulfur.to_string(), sulfur.name());
}

#[test]
fn kind_order_is_name_order() {
    let mut names = ["material-equivalent", "carbon", "energy"];
    let mut kinds = names.map(|name| EcosimKinds::declare(name).unwrap());
    names.sort();
    kinds.sort();
    assert_eq!(kinds.map(EcosimKind::name), names);
}
