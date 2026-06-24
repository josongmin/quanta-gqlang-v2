//! Declaration + META catalog semantics over `hir::CatalogDeclHirV1` /
//! `hir::MetaHirV1` (ticket 11, families VIEW/MODEL/PACK/PROFILE/GATE/RULE and
//! the USE PACK / REQUIRE CERTIFIED meta directives).
//!
//! This is the FRONTEND-CONTRACT seam for the declaration families: each
//! declaration PARSES (upstream) then BINDS into a catalog of stable assets.
//! Declarations produce NO relational plan; they register in a
//! [`CatalogV1`] and resolve fail-closed under the v2 admissibility rules:
//!
//! - duplicate asset name (`<kind>::<name>` collision) rejects
//!   ([`ModuleDiagnosticV1::DuplicateCatalogName`]);
//! - a VIEW whose inline SELECT references (transitively) itself rejects
//!   ([`ModuleDiagnosticV1::ViewInlineCycle`]);
//! - `REQUIRE CERTIFIED` forces every PROFILE to admission `certified-only`;
//!   any non-certified profile then rejects
//!   ([`ModuleDiagnosticV1::CertifiedRequiredViolation`]);
//! - `USE PACK <id>` whose pack name has no resolvable manifest rejects
//!   ([`ModuleDiagnosticV1::UnknownPackManifest`]);
//! - `USE PACK` lockfile paths must end in `.lock`
//!   ([`ModuleDiagnosticV1::InvalidLockfilePath`]).
//!
//! Catalog asset ids are the stable string `"<kind>::<name>"`, matching the
//! SHOW catalog categories so SHOW-style enumeration is a direct projection.

use std::collections::BTreeMap;

use quanta_gqlang_v2_hir::{
    ActivationV1, AdmissionV1, CatalogDeclHirV1, CatalogPayloadV1, MetaHirV1, MetaKindV1,
    ViewBodyV1,
};

use crate::diagnostic::ModuleDiagnosticV1;
use crate::manifest::PackGraphV1;

/// The closed set of catalog asset kinds, one per declaration family.
///
/// The string form is the `<kind>` prefix of an asset id and matches the SHOW
/// catalog category vocabulary (`views`/`models`/`packs`/`profiles`/`gates`/
/// `rules`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CatalogKindV1 {
    /// A VIEW declaration.
    View,
    /// A MODEL declaration.
    Model,
    /// A PACK declaration.
    Pack,
    /// A PROFILE declaration.
    Profile,
    /// A GATE declaration.
    Gate,
    /// A RULE binding declaration.
    Rule,
}

impl CatalogKindV1 {
    /// The stable lowercase tag used as the `<kind>` prefix of an asset id.
    #[must_use]
    pub const fn tag(self) -> &'static str {
        match self {
            Self::View => "view",
            Self::Model => "model",
            Self::Pack => "pack",
            Self::Profile => "profile",
            Self::Gate => "gate",
            Self::Rule => "rule",
        }
    }

    /// The kind a catalog payload registers under.
    #[must_use]
    pub const fn of_payload(payload: &CatalogPayloadV1) -> Self {
        match payload {
            CatalogPayloadV1::View(_) => Self::View,
            CatalogPayloadV1::Model { .. } => Self::Model,
            CatalogPayloadV1::Pack { .. } => Self::Pack,
            CatalogPayloadV1::Profile { .. } => Self::Profile,
            CatalogPayloadV1::Gate { .. } => Self::Gate,
            CatalogPayloadV1::RuleBinding { .. } => Self::Rule,
        }
    }
}

/// A registered catalog asset: a stable id plus the declaration that produced
/// it. The declaration is retained by value so resolution and SHOW-style
/// enumeration read the same frozen structure the binder consumed.
#[derive(Clone, Debug, PartialEq)]
pub struct CatalogAssetV1 {
    /// The asset kind.
    pub kind: CatalogKindV1,
    /// The declared (unqualified) name.
    pub name: String,
    /// The originating declaration.
    pub decl: CatalogDeclHirV1,
}

impl CatalogAssetV1 {
    /// The stable asset id, `"<kind>::<name>"`.
    #[must_use]
    pub fn asset_id(&self) -> String {
        format!("{}::{}", self.kind.tag(), self.name)
    }
}

/// A catalog/registry of VIEW/MODEL/PACK/PROFILE/GATE/RULE declarations plus the
/// active META state (`REQUIRE CERTIFIED`, `USE PACK` activations).
///
/// Registration is incremental and fail-closed: re-registering a colliding
/// `<kind>::<name>` rejects immediately. [`CatalogV1::resolve`] then applies the
/// cross-declaration admissibility rules (VIEW inline-select cycles, REQUIRE
/// CERTIFIED admission forcing) against a host [`PackGraphV1`].
#[derive(Clone, Debug, Default)]
pub struct CatalogV1 {
    /// Assets keyed by stable asset id for deterministic enumeration.
    assets: BTreeMap<String, CatalogAssetV1>,
    /// Whether `REQUIRE CERTIFIED` is in force.
    require_certified: bool,
    /// Recorded `USE PACK` activations, pack id -> activation.
    active_packs: BTreeMap<String, ActivationV1>,
}

impl CatalogV1 {
    /// An empty catalog with no active META state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            assets: BTreeMap::new(),
            require_certified: false,
            active_packs: BTreeMap::new(),
        }
    }

    /// Register one declaration, minting its `<kind>::<name>` asset id.
    /// Fail-closed with [`ModuleDiagnosticV1::DuplicateCatalogName`] when an
    /// asset of the same id is already registered.
    pub fn register(&mut self, decl: CatalogDeclHirV1) -> Result<(), ModuleDiagnosticV1> {
        let kind = CatalogKindV1::of_payload(&decl.payload);
        let asset = CatalogAssetV1 {
            kind,
            name: decl.name.clone(),
            decl,
        };
        let id = asset.asset_id();
        if self.assets.contains_key(&id) {
            return Err(ModuleDiagnosticV1::DuplicateCatalogName {
                kind: kind.tag(),
                name: asset.name,
            });
        }
        let _ = self.assets.insert(id, asset);
        Ok(())
    }

    /// Apply a META directive to the catalog state. `USE PACK` validates the
    /// lockfile path shape and the pack-name -> manifest resolution against
    /// `manifests`; `REQUIRE CERTIFIED` flips the certified-only requirement.
    pub fn apply_meta(
        &mut self,
        meta: &MetaHirV1,
        manifests: &PackGraphV1,
    ) -> Result<(), ModuleDiagnosticV1> {
        match &meta.kind {
            MetaKindV1::RequireCertified => {
                self.require_certified = true;
                Ok(())
            }
            MetaKindV1::UsePack {
                pack_id,
                activation,
                lockfile,
            } => {
                if let Some(path) = lockfile {
                    if !path.ends_with(".lock") {
                        return Err(ModuleDiagnosticV1::InvalidLockfilePath {
                            pack: pack_id.clone(),
                            path: path.clone(),
                        });
                    }
                }
                if manifests.manifest(pack_id).is_none() {
                    return Err(ModuleDiagnosticV1::UnknownPackManifest {
                        pack: pack_id.clone(),
                    });
                }
                let _ = self.active_packs.insert(pack_id.clone(), *activation);
                Ok(())
            }
        }
    }

    /// Whether `REQUIRE CERTIFIED` is in force.
    #[must_use]
    pub const fn certified_required(&self) -> bool {
        self.require_certified
    }

    /// Borrow a registered asset by its stable `<kind>::<name>` id.
    #[must_use]
    pub fn asset(&self, asset_id: &str) -> Option<&CatalogAssetV1> {
        self.assets.get(asset_id)
    }

    /// Number of registered assets.
    #[must_use]
    pub fn len(&self) -> usize {
        self.assets.len()
    }

    /// Whether the catalog holds no assets.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.assets.is_empty()
    }

    /// All asset ids in deterministic sorted order (SHOW-style enumeration over
    /// every category).
    #[must_use]
    pub fn asset_ids(&self) -> Vec<String> {
        self.assets.keys().cloned().collect()
    }

    /// Asset ids of a single kind, in deterministic sorted order. This is the
    /// direct projection a `SHOW <category>` introspection consumes.
    #[must_use]
    pub fn enumerate(&self, kind: CatalogKindV1) -> Vec<String> {
        self.assets
            .iter()
            .filter(|(_, asset)| asset.kind == kind)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Resolve the whole catalog fail-closed under the v2 admissibility rules.
    ///
    /// Order is deterministic: (1) every VIEW inline-select reference cycle, then
    /// (2) the REQUIRE CERTIFIED admission forcing over every PROFILE. All
    /// iteration is over sorted `BTreeMap`s so the first reject is stable.
    pub fn resolve(&self) -> Result<ResolvedCatalogV1, ModuleDiagnosticV1> {
        self.check_view_cycles()?;
        self.check_certified_admission()?;
        Ok(ResolvedCatalogV1 {
            asset_ids: self.asset_ids(),
        })
    }

    /// (1) Reject any VIEW whose inline SELECT transitively references itself.
    ///
    /// Only inline VIEW bodies form edges: an inline VIEW's relation sources that
    /// name another VIEW (`view::<name>`) are edges to follow. A VIEW referencing
    /// a non-VIEW source (a base table) is a leaf. Detection is a DFS over the
    /// VIEW subgraph; the first cycle (smallest entry under sorted iteration)
    /// rejects.
    fn check_view_cycles(&self) -> Result<(), ModuleDiagnosticV1> {
        // Build the VIEW -> referenced-VIEW-name adjacency from inline bodies.
        let mut edges: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for asset in self.assets.values() {
            if asset.kind != CatalogKindV1::View {
                continue;
            }
            let CatalogPayloadV1::View(body) = &asset.decl.payload else {
                continue;
            };
            let mut targets = Vec::new();
            match body {
                ViewBodyV1::Inline(query) => {
                    collect_view_refs(&query.body, &mut targets);
                }
                ViewBodyV1::QueryRef(_) => {}
            }
            // Keep only edges to names that are themselves registered VIEWs.
            targets.retain(|name| {
                self.assets
                    .contains_key(&format!("{}::{}", CatalogKindV1::View.tag(), name))
            });
            targets.sort();
            targets.dedup();
            let _ = edges.insert(asset.name.clone(), targets);
        }

        // DFS for a back-edge, starting from each VIEW in sorted order.
        for start in edges.keys() {
            let mut stack: Vec<String> = Vec::new();
            if let Some(cycle) = dfs_cycle(start, &edges, &mut stack) {
                return Err(ModuleDiagnosticV1::ViewInlineCycle { cycle });
            }
        }
        Ok(())
    }

    /// (2) When `REQUIRE CERTIFIED` is in force every PROFILE must be admission
    /// `certified-only`; the first non-certified PROFILE (sorted) rejects.
    fn check_certified_admission(&self) -> Result<(), ModuleDiagnosticV1> {
        if !self.require_certified {
            return Ok(());
        }
        for asset in self.assets.values() {
            if asset.kind != CatalogKindV1::Profile {
                continue;
            }
            if let CatalogPayloadV1::Profile { admission, .. } = &asset.decl.payload {
                if *admission != AdmissionV1::CertifiedOnly {
                    return Err(ModuleDiagnosticV1::CertifiedRequiredViolation {
                        profile: asset.name.clone(),
                        admission: admission_tag(*admission),
                    });
                }
            }
        }
        Ok(())
    }
}

/// The frozen result of resolving a [`CatalogV1`]: the deterministically-sorted
/// asset ids that survived every admissibility rule. No plan is produced;
/// declarations are catalog state, not relational producers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedCatalogV1 {
    asset_ids: Vec<String>,
}

impl ResolvedCatalogV1 {
    /// The resolved asset ids in deterministic sorted order.
    #[must_use]
    pub fn asset_ids(&self) -> &[String] {
        &self.asset_ids
    }
}

/// Stable lowercase tag for a PROFILE admission policy.
const fn admission_tag(admission: AdmissionV1) -> &'static str {
    match admission {
        AdmissionV1::Allowlist => "allowlist",
        AdmissionV1::ReviewRequired => "review-required",
        AdmissionV1::CertifiedOnly => "certified-only",
    }
}

/// Collect the relation-source names a VIEW inline body references, recursing
/// through projection / filter / composition. Each leaf `Source` name is a
/// candidate VIEW edge; the caller filters to registered VIEWs.
fn collect_view_refs(relation: &quanta_gqlang_v2_hir::RelationExprHirV1, out: &mut Vec<String>) {
    use quanta_gqlang_v2_hir::RelationExprKindV1;
    match &relation.kind {
        RelationExprKindV1::Source(source) => out.push(source.name.clone()),
        RelationExprKindV1::Project(project) => collect_view_refs(&project.input, out),
        RelationExprKindV1::Filter(filter) => collect_view_refs(&filter.input, out),
        RelationExprKindV1::Compose(compose) => {
            for operand in &compose.operands {
                collect_view_refs(operand, out);
            }
        }
    }
}

/// DFS from `node` over `edges`; returns a cycle path (closed at the repeated
/// entry) if a back-edge to a node already on `stack` is found.
fn dfs_cycle(
    node: &str,
    edges: &BTreeMap<String, Vec<String>>,
    stack: &mut Vec<String>,
) -> Option<Vec<String>> {
    if let Some(pos) = stack.iter().position(|n| n == node) {
        // Back-edge: the cycle is the stack tail from `pos`, closed at `node`.
        let mut cycle: Vec<String> = stack.get(pos..).map_or_else(Vec::new, <[String]>::to_vec);
        cycle.push(node.to_string());
        return Some(cycle);
    }
    stack.push(node.to_string());
    if let Some(targets) = edges.get(node) {
        for target in targets {
            if let Some(cycle) = dfs_cycle(target, edges, stack) {
                return Some(cycle);
            }
        }
    }
    let _ = stack.pop();
    None
}
