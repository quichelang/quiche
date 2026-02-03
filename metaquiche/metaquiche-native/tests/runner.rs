#[path = "../../../tests/runner_utils.rs"]
mod runner_utils;

#[test]
fn integration_tests() {
    // Skip if QUICHE_COMPILER_BIN is not set - the native compiler requires Stage 1 build
    if std::env::var("QUICHE_COMPILER_BIN").is_err() {
        println!(
            "Skipping: QUICHE_COMPILER_BIN not set. Run 'make test' for full integration tests."
        );
        return;
    }
    // Run tests using the transpiler logic (creates cargo project with runtime dep)
    runner_utils::run_self_hosted_tests("metaquiche-native", "metaquiche-native");
}
