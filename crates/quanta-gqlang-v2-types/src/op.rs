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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn closed_registry_preserves_complete_operator_surface_and_explicit_ids() {
        let registry=OperatorRegistryV2::try_closed().expect("valid static registry");
        for symbol in ["+","-","*","/","%","<","<=",">",">=","=","!=","and","or","xor","++"] { assert!(!registry.binary_overloads(symbol).is_empty(),"missing {symbol}"); }
        for symbol in ["+","-","not"] { assert!(!registry.unary_overloads(symbol).is_empty(),"missing {symbol}"); }
        let ids=registry.binary.values().flatten().map(|entry|entry.id).chain(registry.unary.values().flatten().map(|entry|entry.id)).collect::<BTreeSet<_>>();
        let count=registry.binary.values().map(Vec::len).sum::<usize>()+registry.unary.values().map(Vec::len).sum::<usize>();
        assert_eq!(ids.len(),count);
    }
    #[test]
    fn duplicate_id_and_semantic_domain_reject() {
        let one=u(1,"-",TypeV1::int(),TypeV1::int());
        let duplicate_id=u(1,"+",TypeV1::int(),TypeV1::int());
        assert_eq!(OperatorRegistryV2::try_from_entries(vec![one.clone(),duplicate_id],vec![]).unwrap_err().code(),"types.registry_duplicate_id");
        let duplicate_domain=u(2,"-",TypeV1::int(),TypeV1::float());
        assert_eq!(OperatorRegistryV2::try_from_entries(vec![one,duplicate_domain],vec![]).unwrap_err().code(),"types.registry_duplicate_id");
    }
}

impl OperatorRegistryV2 {
    fn try_from_entries(
        unary: Vec<(String, UnaryOverloadV1)>, binary: Vec<(String, BinaryOverloadV1)>,
    ) -> Result<Self, TypeDiagnosticV2> {
        let mut ids = BTreeSet::new();
        let mut unary_semantics: Vec<(String, TypeV1)> = Vec::new();
        let mut binary_semantics: Vec<(String, TypeV1, TypeV1)> = Vec::new();
        let mut unary_map: BTreeMap<String, Vec<UnaryOverloadV1>> = BTreeMap::new();
        let mut binary_map: BTreeMap<String, Vec<BinaryOverloadV1>> = BTreeMap::new();
        for (symbol, overload) in unary {
            if !ids.insert(overload.id) { return Err(TypeDiagnosticV2::RegistryDuplicateId { registry: "operator", id: overload.id.raw() }); }
            let semantic = (symbol.clone(), overload.operand.clone());
            if unary_semantics.contains(&semantic) { return Err(TypeDiagnosticV2::RegistryDuplicateId { registry: "operator_signature", id: overload.id.raw() }); }
            unary_semantics.push(semantic);
            unary_map.entry(symbol).or_default().push(overload);
        }
        for (symbol, overload) in binary {
            if !ids.insert(overload.id) { return Err(TypeDiagnosticV2::RegistryDuplicateId { registry: "operator", id: overload.id.raw() }); }
            let semantic = (symbol.clone(), overload.lhs.clone(), overload.rhs.clone());
            if binary_semantics.contains(&semantic) { return Err(TypeDiagnosticV2::RegistryDuplicateId { registry: "operator_signature", id: overload.id.raw() }); }
            binary_semantics.push(semantic);
            binary_map.entry(symbol).or_default().push(overload);
        }
        Ok(Self { unary: unary_map, binary: binary_map })
    }

    pub fn try_closed() -> Result<Self, TypeDiagnosticV2> {
        use crate::ScalarTyV1;
        let duration = || TypeV1::scalar(ScalarTyV1::Duration);
        let unary = vec![
            u(1, "-", TypeV1::int(), TypeV1::int()), u(2, "-", TypeV1::float(), TypeV1::float()), u(3, "-", duration(), duration()),
            u(4, "+", TypeV1::int(), TypeV1::int()), u(5, "+", TypeV1::float(), TypeV1::float()), u(6, "+", duration(), duration()),
            u(7, "not", TypeV1::bool(), TypeV1::bool()),
        ];
        let mut binary = vec![
            b(101,"+",TypeV1::int(),TypeV1::int(),TypeV1::int()), b(102,"-",TypeV1::int(),TypeV1::int(),TypeV1::int()), b(103,"*",TypeV1::int(),TypeV1::int(),TypeV1::int()), b(104,"/",TypeV1::int(),TypeV1::int(),TypeV1::int()), b(105,"%",TypeV1::int(),TypeV1::int(),TypeV1::int()),
            b(106,"+",TypeV1::float(),TypeV1::float(),TypeV1::float()), b(107,"-",TypeV1::float(),TypeV1::float(),TypeV1::float()), b(108,"*",TypeV1::float(),TypeV1::float(),TypeV1::float()), b(109,"/",TypeV1::float(),TypeV1::float(),TypeV1::float()),
            b(110,"+",duration(),duration(),duration()), b(111,"-",duration(),duration(),duration()), b(112,"*",duration(),duration(),duration()), b(113,"*",duration(),TypeV1::int(),duration()), b(114,"/",duration(),TypeV1::int(),duration()),
            b(115,"and",TypeV1::bool(),TypeV1::bool(),TypeV1::bool()), b(116,"or",TypeV1::bool(),TypeV1::bool(),TypeV1::bool()), b(117,"xor",TypeV1::bool(),TypeV1::bool(),TypeV1::bool()),
            b(118,"++",TypeV1::string(),TypeV1::string(),TypeV1::string()),
        ];
        for (base, symbol) in [(200,"<"),(210,"<="),(220,">"),(230,">=")] {
            binary.push(b(base,symbol,TypeV1::int(),TypeV1::int(),TypeV1::bool()));
            binary.push(b(base+1,symbol,TypeV1::float(),TypeV1::float(),TypeV1::bool()));
            binary.push(b(base+2,symbol,TypeV1::string(),TypeV1::string(),TypeV1::bool()));
            binary.push(b(base+3,symbol,duration(),duration(),TypeV1::bool()));
        }
        let scalars = [ScalarTyV1::Bool, ScalarTyV1::Int, ScalarTyV1::Float, ScalarTyV1::String, ScalarTyV1::Duration, ScalarTyV1::Location, ScalarTyV1::Path, ScalarTyV1::RepoPath, ScalarTyV1::EntityRef, ScalarTyV1::SymbolRef, ScalarTyV1::RuleRef, ScalarTyV1::SourceScope, ScalarTyV1::ProofTree];
        let eq_ids = [300,301,302,303,304,305,306,307,308,309,310,311,312];
        let ne_ids = [320,321,322,323,324,325,326,327,328,329,330,331,332];
        for ((scalar, eq_id), ne_id) in scalars.into_iter().zip(eq_ids).zip(ne_ids) {
            binary.push(b(eq_id,"=",TypeV1::scalar(scalar.clone()),TypeV1::scalar(scalar.clone()),TypeV1::bool()));
            binary.push(b(ne_id,"!=",TypeV1::scalar(scalar.clone()),TypeV1::scalar(scalar),TypeV1::bool()));
        }
        Self::try_from_entries(unary, binary)
    }
    #[must_use] pub fn unary_overloads(&self, symbol: &str) -> &[UnaryOverloadV1] { self.unary.get(symbol).map_or(&[], Vec::as_slice) }
    #[must_use] pub fn binary_overloads(&self, symbol: &str) -> &[BinaryOverloadV1] { self.binary.get(symbol).map_or(&[], Vec::as_slice) }
}

fn u(id: u32, symbol: &str, operand: TypeV1, result: TypeV1) -> (String, UnaryOverloadV1) {
    (symbol.to_owned(), UnaryOverloadV1 { id: OperatorIdV2(id), operand, result })
}
fn b(id: u32, symbol: &str, lhs: TypeV1, rhs: TypeV1, result: TypeV1) -> (String, BinaryOverloadV1) {
    (symbol.to_owned(), BinaryOverloadV1 { id: OperatorIdV2(id), lhs, rhs, result })
}
