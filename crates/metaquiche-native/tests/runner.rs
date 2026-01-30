#[path = "../../../tests/runner_utils.rs"]
mod runner_utils;

#[test]
fn integration_tests() {
    // Run tests using the transpiler logic (creates cargo project with runtime dep)
    runner_utils::run_self_hosted_tests("quiche-self", "quiche-self");
}
