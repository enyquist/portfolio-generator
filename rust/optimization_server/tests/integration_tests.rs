// tests/integration_tests.rs

use actix_web::{test, App};
use optimization_server::handlers::{optimize, health_check};
use optimization_server::models::{OptimizationRequest, OptimizationResult, FilingStatus};
use std::collections::HashMap;

#[actix_rt::test]
async fn test_optimize_endpoint_success() {
    let mut app = test::init_service(
        App::new().service(optimize)
    ).await;

    // Prepare test data
    let request = OptimizationRequest {
        dimension: 3,
        lower_bounds: vec![0.0, 0.0, 0.0],
        upper_bounds: vec![1.0, 1.0, 1.0],
        initial_capital: 100000.0,
        salary: 50000.0,
        required_income: 20000.0,
        min_div_growth: 0.05,
        min_cagr: 0.07,
        min_yield: 0.03,
        div_preference: 0.5,
        cagr_preference: 0.3,
        yield_preference: 0.2,
        filing_status: FilingStatus::Single,
        columns: {
            let mut columns = HashMap::new();
            columns.insert("div_growth_rates".to_string(), vec![0.04, 0.05, 0.06]);
            columns.insert("cagr_rates".to_string(), vec![0.06, 0.07, 0.08]);
            columns.insert("yields".to_string(), vec![0.02, 0.03, 0.04]);
            columns.insert("expense_ratios".to_string(), vec![0.001, 0.002, 0.003]);
            columns.insert("sector".to_string(), vec![1.0, 2.0, 1.0]);
            columns
        },
    };

    let req = test::TestRequest::post()
        .uri("/optimize")
        .set_json(&request)
        .to_request();

    let resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_success());

    let response_body = test::read_body(resp).await;
    let result: OptimizationResult = serde_json::from_slice(&response_body).unwrap();

    assert!(result.success);
    assert!(result.x.is_some());
    assert!(result.objective_value.is_some());
    assert!(result.message.contains("Optimization succeeded"));

    if let Some(ref x_values) = result.x {
        let sum_x: f64 = x_values.iter().sum();
        assert!((sum_x - 1.0).abs() < 1e-6, "Sum of x_i does not equal 1");
    }
    
}

#[actix_rt::test]
async fn test_optimize_endpoint_bad_request() {
    let mut app = test::init_service(
        App::new().service(optimize)
    ).await;

    // Prepare invalid test data (mismatched bounds and dimension)
    let request = OptimizationRequest {
        dimension: 3,
        lower_bounds: vec![0.0, 0.0], // Incorrect length
        upper_bounds: vec![1.0, 1.0, 1.0],
        // ... [other fields same as before] ...
        initial_capital: 100000.0,
        salary: 50000.0,
        required_income: 20000.0,
        min_div_growth: 0.05,
        min_cagr: 0.07,
        min_yield: 0.03,
        div_preference: 0.5,
        cagr_preference: 0.3,
        yield_preference: 0.2,
        filing_status: FilingStatus::Single,
        columns: HashMap::new(),
    };

    let req = test::TestRequest::post()
        .uri("/optimize")
        .set_json(&request)
        .to_request();

    let resp = test::call_service(&mut app, req).await;

    assert_eq!(resp.status(), 400);

    let response_body = test::read_body(resp).await;
    let result: OptimizationResult = serde_json::from_slice(&response_body).unwrap();

    assert!(!result.success);
    assert_eq!(result.message, "Bounds size does not match dimension".to_string());
}

#[actix_rt::test]
async fn test_optimize_upper_bounds_infeasible() {
    let mut app = test::init_service(App::new().service(optimize)).await;

    // Prepare test data with upper bounds sum less than 1
    let request = OptimizationRequest {
        dimension: 3,
        lower_bounds: vec![0.0, 0.0, 0.0],
        upper_bounds: vec![0.2, 0.3, 0.4], // Sum is 0.9
        // ... [other fields as before] ...
        initial_capital: 100000.0,
        salary: 50000.0,
        required_income: 20000.0,
        min_div_growth: 0.05,
        min_cagr: 0.07,
        min_yield: 0.03,
        div_preference: 0.5,
        cagr_preference: 0.3,
        yield_preference: 0.2,
        filing_status: FilingStatus::Single,
        columns: HashMap::new(),
    };

    let req = test::TestRequest::post()
        .uri("/optimize")
        .set_json(&request)
        .to_request();

    let resp = test::call_service(&mut app, req).await;

    assert_eq!(resp.status(), 400);

    let response_body = test::read_body(resp).await;
    let result: OptimizationResult =
        serde_json::from_slice(&response_body).unwrap();

    assert!(!result.success);
    assert_eq!(
        result.message,
        "Sum of upper bounds is less than 1, problem infeasible.".to_string()
    );
}

#[actix_rt::test]
async fn test_health_check() {
    let mut app = test::init_service(
        App::new().service(health_check)
    ).await;

    let req = test::TestRequest::get()
        .uri("/health")
        .to_request();

    let resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_success());
    let response_body = test::read_body(resp).await;
    assert_eq!(response_body, "OK");
}