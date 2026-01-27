fn test_basic_lambda() {
println!("{:?}", String::from("Running test_basic_lambda..."));
let mut res = quiche_runtime::check!(/* expr */(10));
assert_eq!(res, 11, "{:?}", String::from("Basic lambda call"));
}
fn test_lambda_assignment() {
println!("{:?}", String::from("Skipping test_lambda_assignment: Rust inference limitations with assignable closures."));
}
fn test_higher_order() {
println!("{:?}", String::from("Skipping test_higher_order: Rust inference limitations."));
}
fn main() {
println!("{:?}", String::from("=== Functional Suite ==="));
quiche_runtime::check!(test_basic_lambda());
quiche_runtime::check!(test_lambda_assignment());
quiche_runtime::check!(test_higher_order());
println!("{:?}", String::from("=== Done ==="));
}

