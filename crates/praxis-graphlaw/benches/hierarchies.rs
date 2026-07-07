#[macro_use]
extern crate bencher;

use bencher::Bencher;

use praxis_graphlaw::TripleStore;

fn infer_hierarchy(max_depth: i32) {
    let mut data = ":a a :U0\n".to_owned();
    for i in 0..max_depth {
        data += format!("{{?a a :U{}}}=>{{?a a :U{}}}\n", i, i + 1).as_str();
        data += format!("{{?a a :U{}}}=>{{?a a :J{}}}\n", i, i + 1).as_str();
        data += format!("{{?a a :U{}}}=>{{?a a :Q{}}}\n", i, i + 1).as_str();
    }
    let mut store = TripleStore::from(data.as_str());
    store.materialize();
}
fn infer_hierarchy_rdf_rule(max_depth: i32) {
    let mut data = ":a a :U0\n\
                        {?a :subClassOf ?b.?b :subClassOf ?c}=>{?a :subClassOf ?c}\n"
        .to_owned();
    for i in 0..max_depth {
        data += format!(":U{} :subClassOf :U{}.\n", i, i + 1).as_str();
        data += format!(":U{} :subClassOf :J{}.\n", i, i + 1).as_str();
        data += format!(":U{} :subClassOf :Q{}.\n", i, i + 1).as_str();
    }
    let mut store = TripleStore::from(data.as_str());
    store.materialize();
}
// Kept out of benchmark_group! (too slow for routine runs) but retained for manual use.
#[allow(dead_code)]
fn test_hierarchy_10000(bench: &mut Bencher) {
    bench.iter(|| {
        let max_depth = 10000;
        infer_hierarchy(max_depth);
    });
}
fn test_hierarchy_1000(bench: &mut Bencher) {
    bench.iter(|| {
        let max_depth = 1000;
        infer_hierarchy(max_depth);
    });
}
fn test_hierarchy_100(bench: &mut Bencher) {
    bench.iter(|| {
        let max_depth = 100;
        infer_hierarchy(max_depth);
    });
}
fn test_hierarchy_10(bench: &mut Bencher) {
    bench.iter(|| {
        let max_depth = 10;
        infer_hierarchy(max_depth);
    });
}
// Kept out of benchmark_group! (too slow for routine runs) but retained for manual use.
#[allow(dead_code)]
fn test_rdf_hierarchy_10000(bench: &mut Bencher) {
    bench.iter(|| {
        let max_depth = 10000;
        infer_hierarchy_rdf_rule(max_depth);
    });
}
fn test_rdf_hierarchy_1000(bench: &mut Bencher) {
    bench.iter(|| {
        let max_depth = 1000;
        infer_hierarchy_rdf_rule(max_depth);
    });
}
fn test_rdf_hierarchy_100(bench: &mut Bencher) {
    bench.iter(|| {
        let max_depth = 100;
        infer_hierarchy_rdf_rule(max_depth);
    });
}
fn test_rdf_hierarchy_10(bench: &mut Bencher) {
    bench.iter(|| {
        let max_depth = 10;
        infer_hierarchy_rdf_rule(max_depth);
    });
}
benchmark_group!(
    benches,
    test_hierarchy_10,
    test_rdf_hierarchy_10,
    test_hierarchy_100,
    test_rdf_hierarchy_100,
    test_hierarchy_1000,
    test_rdf_hierarchy_1000
);
benchmark_main!(benches);
