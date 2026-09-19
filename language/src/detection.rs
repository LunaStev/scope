use crate::backend::Engine;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Evidence { Filename, Extension, Shebang, Unknown }
/// Identity and the ability to classify its physical lines are separate facts.
#[derive(Clone, Debug)]
pub struct Detection {
    pub id: String,
    pub name: String,
    pub evidence: Evidence,
    pub(crate) engine: Engine,
}
impl Detection { pub fn is_known(&self) -> bool { self.evidence != Evidence::Unknown } }
