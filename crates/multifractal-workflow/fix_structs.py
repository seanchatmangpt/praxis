import os
import re

os.chdir('/Users/sac/praxis/crates/multifractal-workflow')

# 1. f08
with open('src/f08_pddl_planning/hook_binder.rs', 'r') as f:
    content = f.read()
new_content = re.sub(
    r'Ok\(ActionCapabilityMap \{.*?\}\)',
    'Ok(ActionCapabilityMap { content_digest: "".to_string(), iri: "".to_string() })',
    content,
    flags=re.DOTALL
)
with open('src/f08_pddl_planning/hook_binder.rs', 'w') as f:
    f.write(new_content)
print("f08 fixed")

# 2. f09
with open('src/f09_mfw_growth.rs', 'r') as f:
    content = f.read()
pddl_domain_init = '''Pddl8Domain {
        name: "".into(),
        predicates: Vec::new(),
        actions: Vec::new(),
        types: Vec::new(),
        functions: Vec::new(),
        durative_actions: Vec::new(),
        derived: Vec::new(),
        constraints: Vec::new(),
        processes: Vec::new(),
        events: Vec::new(),
    }'''
pddl_problem_init = '''Pddl8Problem {
        name: "".into(),
        domain: "".into(),
        objects: Vec::new(),
        init: Vec::new(),
        goal: Vec::new(),
        object_types: Vec::new(),
        fn_values: Vec::new(),
        timed_inits: Vec::new(),
        preferences: Vec::new(),
        metric: None,
    }'''

new_content = re.sub(
    r'Ok\(ContinuationGoal \{.*?\}\)',
    f'Ok(ContinuationGoal {{ domain: {pddl_domain_init}, problem: {pddl_problem_init} }})',
    content,
    flags=re.DOTALL
)
with open('src/f09_mfw_growth.rs', 'w') as f:
    f.write(new_content)
print("f09 fixed")

# 3. f26
with open('src/f26_ontology_self_play.rs', 'r') as f:
    content = f.read()
new_content = re.sub(
    r'Ok\(ScenarioGraph \{.*?\}\)',
    'Ok(ScenarioGraph { facts_turtle: "".to_string() })',
    content,
    flags=re.DOTALL
)
with open('src/f26_ontology_self_play.rs', 'w') as f:
    f.write(new_content)
print("f26 fixed")
