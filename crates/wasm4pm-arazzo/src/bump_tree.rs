use bumpalo::Bump;
use bumpalo::collections::{String, Vec};

pub struct BumpArazzo<'bump> {
    pub arazzo: String<'bump>,
    pub workflows: Vec<'bump, BumpWorkflow<'bump>>,
}

pub struct BumpWorkflow<'bump> {
    pub name: String<'bump>,
    pub steps: Vec<'bump, BumpStep<'bump>>,
}

pub struct BumpStep<'bump> {
    pub name: String<'bump>,
    pub target_url: String<'bump>,
    pub parameters: Vec<'bump, String<'bump>>,
}
