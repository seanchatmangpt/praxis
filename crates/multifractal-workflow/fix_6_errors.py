import os
import re

os.chdir('/Users/sac/praxis/crates/multifractal-workflow')

# 1. f25 import
with open('src/f25_receipts_replay.rs', 'r') as f:
    content = f.read()
if "use crate::f11_bcinr_runtime::OcelCausalReceipt;" not in content:
    content = content.replace("use bcinr_powl_receipt::L2EngineError;", "use bcinr_powl_receipt::L2EngineError;\nuse crate::f11_bcinr_runtime::OcelCausalReceipt;")
with open('src/f25_receipts_replay.rs', 'w') as f:
    f.write(content)
print("f25 fixed")

# 2. f26 import and ScenarioGraph
with open('src/f26_ontology_self_play.rs', 'r') as f:
    content = f.read()
if "use crate::f20_self_play_catalog::DimensionCatalog;" not in content:
    content = content.replace("use praxis_core::shape::ShapeId;", "use praxis_core::shape::ShapeId;\nuse crate::f20_self_play_catalog::DimensionCatalog;")
content = content.replace("Ok(ScenarioGraph {})", 'Ok(ScenarioGraph { facts_turtle: "".to_string() })')
with open('src/f26_ontology_self_play.rs', 'w') as f:
    f.write(content)
print("f26 fixed")

# 3. f08 ActionCapabilityMap
with open('src/f08_pddl_planning/hook_binder.rs', 'r') as f:
    content = f.read()
content = content.replace("Ok(ActionCapabilityMap {})", 'Ok(ActionCapabilityMap { content_digest: "".to_string(), iri: "".to_string() })')
with open('src/f08_pddl_planning/hook_binder.rs', 'w') as f:
    f.write(content)
print("f08 fixed")

# 4. f09 ContinuationGoal
with open('src/f09_mfw_growth.rs', 'r') as f:
    content = f.read()
content = content.replace("Ok(ContinuationGoal {})", 'Ok(ContinuationGoal { domain: "".to_string(), problem: "".to_string() })')
with open('src/f09_mfw_growth.rs', 'w') as f:
    f.write(content)
print("f09 fixed")

# 5. f22 detect_timeout signature
with open('src/f22_compensation.rs', 'r') as f:
    content = f.read()
content = content.replace(
    "pub fn detect_timeout(_actuation: &PriorActuationRef) -> Result<FailureObservation, F22Refusal>",
    "pub fn detect_timeout(_actuation: &PriorActuationRef, _timeout: u64) -> Result<FailureObservation, F22Refusal>"
)
with open('src/f22_compensation.rs', 'w') as f:
    f.write(content)
print("f22 fixed")
