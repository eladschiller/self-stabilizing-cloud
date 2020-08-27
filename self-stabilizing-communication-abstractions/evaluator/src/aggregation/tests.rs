
use super::*;
use super::example_data::example;

use commons::variant::Variant;
/*
#[test]
fn test_get_scenario_round_id_existing() {
    let scenario = Scenario::new(3,0,3,Variant::Algorithm3);
    let result = get_scenario_round_id(&example(), &scenario, 1, 2);

    assert_eq!(result.write_ops, 1745);
}

#[test]
#[should_panic]
fn test_get_scenario_round_id_non_existing_scenario() {
    let scenario = Scenario::new(3,0,3,Variant::Algorithm1);
    let _result = get_scenario_round_id(&example(), &scenario, 1, 2);
}

#[test]
#[should_panic]
fn test_get_scenario_round_id_non_existing_round() {
    let scenario = Scenario::new(3,0,3,Variant::Algorithm3);
    let _result = get_scenario_round_id(&example(), &scenario, 2, 2);
}

#[test]
#[should_panic]
fn test_get_scenario_round_id_non_existing_id() {
    let scenario = Scenario::new(3,0,3,Variant::Algorithm3);
    let _result = get_scenario_round_id(&example(), &scenario, 1, 10);
}

#[test]
fn test_node_averaged_write_latency_for_scenario_round_all_writers() {
    let scenario = Scenario::new(3,3,3,Variant::Algorithm3);
    let result = node_averaged_write_latency_for_scenario_round(&example(), &scenario, 0);
    let run_length = 3.0;
    let expected = ((run_length / 767.0) + (run_length / 829.0) + (run_length / 709.0)) / 3.0;

    assert!(eq_eps(result, expected));
}

fn eq_eps(a: f64, b: f64) -> bool {
    (a-b).abs() < 0.0000001
}
*/
