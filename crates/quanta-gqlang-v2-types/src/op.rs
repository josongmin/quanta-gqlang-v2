use crate::{TypeDiagnosticV2, TypeV1};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OperatorIdV2(u32);
impl OperatorIdV2 { #[must_use] pub const fn from_raw(raw: u32) -> Self { Self(raw) } #[must_use] pub const fn raw(self) -> u32 { self.0 } }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnaryOverloadV1 { pub id: OperatorIdV2, pub operand: TypeV1, pub result: TypeV1 }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BinaryOverloadV1 { pub id: OperatorIdV2, pub lhs: TypeV1, pub rhs: TypeV1, pub result: TypeV1 }

#[derive(Clone, Debug)]
pub struct OperatorRegistryV2 {
    unary: BTreeMap<String, Vec<UnaryOverloadV1>>,
    binary: BTreeMap<String, Vec<BinaryOverloadV1>>,
}

impl OperatorRegistryV2 {
    pub fn try_closed(
        unary: Vec<(String, UnaryOverloadV1)>, binary: Vec<(String, BinaryOverloadV1)>,
    ) -> Result<Self, TypeDiagnosticV2> {
        let mut ids = BTreeSet::new();
        let mut unary_map: BTreeMap<String, Vec<UnaryOverloadV1>> = BTreeMap::new();
        let mut binary_map: BTreeMap<String, Vec<BinaryOverloadV1>> = BTreeMap::new();
        for (symbol, overload) in unary {
            if !ids.insert(overload.id) { return Err(TypeDiagnosticV2::RegistryDuplicateId { registry: "operator", id: overload.id.raw() }); }
            unary_map.entry(symbol).or_default().push(overload);
        }
        for (symbol, overload) in binary {
            if !ids.insert(overload.id) { return Err(TypeDiagnosticV2::RegistryDuplicateId { registry: "operator", id: overload.id.raw() }); }
            binary_map.entry(symbol).or_default().push(overload);
        }
        Ok(Self { unary: unary_map, binary: binary_map })
    }

    #[must_use]
    pub fn closed() -> Self {
        let mut next = 1;
        let mut unary = Vec::new();
        let mut binary = Vec::new();
        let mut u = |symbol: &str, operand: TypeV1, result: TypeV1| { let id = OperatorIdV2(next); next += 1; unary.push((symbol.to_owned(), UnaryOverloadV1 { id, operand, result })); };
        u("-", TypeV1::int(), TypeV1::int()); u("-", TypeV1::float(), TypeV1::float());
        u("+", TypeV1::int(), TypeV1::int()); u("+", TypeV1::float(), TypeV1::float());
        u("not", TypeV1::bool(), TypeV1::bool());
        let mut b = |symbol: &str, lhs: TypeV1, rhs: TypeV1, result: TypeV1| { let id = OperatorIdV2(next); next += 1; binary.push((symbol.to_owned(), BinaryOverloadV1 { id, lhs, rhs, result })); };
        for symbol in ["+", "-", "*", "/", "%"] { b(symbol, TypeV1::int(), TypeV1::int(), TypeV1::int()); }
        for symbol in ["+", "-", "*", "/"] { b(symbol, TypeV1::float(), TypeV1::float(), TypeV1::float()); }
        for symbol in ["<", "<=", ">", ">=", "=", "!="] {
            b(symbol, TypeV1::int(), TypeV1::int(), TypeV1::bool());
            b(symbol, TypeV1::float(), TypeV1::float(), TypeV1::bool());
            b(symbol, TypeV1::string(), TypeV1::string(), TypeV1::bool());
        }
        for symbol in ["and", "or", "xor"] { b(symbol, TypeV1::bool(), TypeV1::bool(), TypeV1::bool()); }
        Self::try_closed(unary, binary).expect("closed operator IDs are statically unique")
    }
    #[must_use] pub fn unary_overloads(&self, symbol: &str) -> &[UnaryOverloadV1] { self.unary.get(symbol).map_or(&[], Vec::as_slice) }
    #[must_use] pub fn binary_overloads(&self, symbol: &str) -> &[BinaryOverloadV1] { self.binary.get(symbol).map_or(&[], Vec::as_slice) }
}
