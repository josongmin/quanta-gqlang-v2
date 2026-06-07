use crate::declarations::pack::{GateDeclAstV1, PackDeclAstV1, ProfileDeclAstV1};
use crate::declarations::view::{ModelDeclAstV1, RuleDeclAstV1, ViewDeclAstV1};
use crate::interactive::meta::{MetaCommandAstV1, WithClauseAstV1};
use crate::interactive::sample::{CompareRootAstV1, ExportRootAstV1, SampleRootAstV1};
use crate::interactive::show::{DescribeRootAstV1, ShowRootAstV1};
use crate::roots::check::CheckRootAstV1;
use crate::roots::explain::ExplainRootAstV1;
use crate::roots::find::FindRootAstV1;
use crate::roots::select::SelectRootAstV1;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DocumentAstV1 {
    pub roots: Vec<AstRootV1>,
    pub with_clause: Option<WithClauseAstV1>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AstRootV1 {
    Select(SelectRootAstV1),
    Find(FindRootAstV1),
    Check(CheckRootAstV1),
    Explain(ExplainRootAstV1),
    Show(ShowRootAstV1),
    Describe(DescribeRootAstV1),
    Sample(SampleRootAstV1),
    Compare(CompareRootAstV1),
    Export(ExportRootAstV1),
    View(ViewDeclAstV1),
    Rule(RuleDeclAstV1),
    Model(ModelDeclAstV1),
    Pack(PackDeclAstV1),
    Profile(ProfileDeclAstV1),
    Gate(GateDeclAstV1),
    Meta(MetaCommandAstV1),
}
