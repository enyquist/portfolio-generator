// tests/integration_tests.rs

use actix_web::{test, App};
use optimization_server::handlers::{optimize, health_check};
use optimization_server::models::{OptimizationRequest, OptimizationResult, FilingStatus};
use std::collections::HashMap;
use serde_json::{json, Value};

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
            columns.insert("dividend_growth_rates".to_string(), vec![0.04, 0.05, 0.06]);
            columns.insert("cagr_rates".to_string(), vec![0.06, 0.07, 0.08]);
            columns.insert("yields".to_string(), vec![0.02, 0.03, 0.04]);
            columns.insert("expense_ratios".to_string(), vec![0.001, 0.002, 0.003]);
            columns.insert("sector".to_string(), vec![1.0, 2.0, 1.0]);
            columns.insert("qualified".to_string(), vec![1.0, 0.0, 1.0]);
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
        columns: {
            let mut columns = HashMap::new();
            columns.insert("dividend_growth_rates".to_string(), vec![0.04, 0.05, 0.06]);
            columns.insert("cagr_rates".to_string(), vec![0.06, 0.07, 0.08]);
            columns.insert("yields".to_string(), vec![0.02, 0.03, 0.04]);
            columns.insert("expense_ratios".to_string(), vec![0.001, 0.002, 0.003]);
            columns.insert("sector".to_string(), vec![1.0, 2.0, 1.0]);
            columns.insert("qualified".to_string(), vec![1.0, 0.0, 1.0]);
            columns
        },
    };

    let req = test::TestRequest::post()
        .uri("/optimize")
        .set_json(&request)
        .to_request();

    let resp = test::call_service(&mut app, req).await;

    assert_eq!(resp.status(), 400);

    let response_body = test::read_body(resp).await;
    let response_json: Value = serde_json::from_slice(&response_body).unwrap();

    // Now, inspect the JSON to check for validation errors
    assert!(
        response_json.get("lower_bounds").is_some(),
        "Expected 'lower_bounds' validation error"
    );

    // Access the errors for 'lower_bounds'
    if let Some(lower_bounds_errors) = response_json.get("lower_bounds") {
        let error_array = lower_bounds_errors.as_array().unwrap();
        let first_error = &error_array[0];
        let error_code = first_error.get("code").unwrap().as_str().unwrap();
        let error_message = first_error.get("message").unwrap().as_str().unwrap();

        assert_eq!(error_code, "lower_bounds_length_mismatch");
        assert_eq!(error_message, "Bounds size does not match dimension");
    }
}

#[actix_web::test]
async fn test_optimize_endpoint_missing_info_request() {
    let mut app = test::init_service(App::new().service(optimize)).await;

    // Prepare invalid JSON payload (missing 'initial_capital')
    let payload = json!({
        "dimension": 3,
        "lower_bounds": [0.0, 0.0, 0.0],
        "upper_bounds": [1.0, 1.0, 1.0],
        // "initial_capital": 100000.0, // Missing
        "salary": 50000.0,
        "required_income": 20000.0,
        "min_div_growth": 0.05,
        "min_cagr": 0.07,
        "min_yield": 0.03,
        "div_preference": 0.5,
        "cagr_preference": 0.3,
        "yield_preference": 0.2,
        "filing_status": "single",
        "columns": {}
    });

    let req = test::TestRequest::post()
        .uri("/optimize")
        .insert_header(("Content-Type", "application/json"))
        .set_payload(payload.to_string())
        .to_request();

    let resp = test::call_service(&mut app, req).await;

    // Assert that the response status is 400 Bad Request
    assert_eq!(resp.status(), 400);

    // Optionally, check the response body
    let response_body = test::read_body(resp).await;
    let response_str = std::str::from_utf8(&response_body).unwrap();

    // The error message may vary; you can check if it contains certain text
    assert!(
        response_str.contains("missing field"),
        "Expected error message about missing field, got: {}",
        response_str
    );
}

#[actix_web::test]
async fn test_optimize_endpoint_invalid_filing_status() {
    let mut app = test::init_service(App::new().service(optimize)).await;

    // Prepare invalid JSON payload (invalid 'filing_status')
    let payload = json!({
        "dimension": 3,
        "lower_bounds": [0.0, 0.0, 0.0],
        "upper_bounds": [1.0, 1.0, 1.0],
        "initial_capital": 100000.0,
        "salary": 50000.0,
        "required_income": 20000.0,
        "min_div_growth": 0.05,
        "min_cagr": 0.07,
        "min_yield": 0.03,
        "div_preference": 0.5,
        "cagr_preference": 0.3,
        "yield_preference": 0.2,
        "filing_status": "baller",
        "columns": {}
    });

    let req = test::TestRequest::post()
        .uri("/optimize")
        .insert_header(("Content-Type", "application/json"))
        .set_payload(payload.to_string())
        .to_request();

    let resp = test::call_service(&mut app, req).await;

    // Assert that the response status is 400 Bad Request
    assert_eq!(resp.status(), 400);

    // Optionally, check the response body
    let response_body = test::read_body(resp).await;
    let response_str = std::str::from_utf8(&response_body).unwrap();

    // The error message may vary; you can check if it contains certain text
    assert!(
        response_str.contains("unknown variant"),
        "Expected error message about unknown variant, got: {}",
        response_str
    );
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
    let response_json: Value = serde_json::from_slice(&response_body).unwrap();

    println!("{}", response_json);

    // Now, inspect the JSON to check for validation errors
    assert!(
        response_json.get("upper_bounds_sum").is_some(),
        "Expected 'upper_bounds_sum' validation error"
    );

    // Access the errors for 'upper_bounds'
    if let Some(lower_bounds_errors) = response_json.get("upper_bounds_sum") {
        let error_array = lower_bounds_errors.as_array().unwrap();
        let first_error = &error_array[0];
        let error_code = first_error.get("code").unwrap().as_str().unwrap();
        let error_message = first_error.get("message").unwrap().as_str().unwrap();

        assert_eq!(error_code, "upper_bounds_sum");
        assert_eq!(error_message, "Sum of upper bounds must be >= 1");
    }
}

#[actix_rt::test]
async fn test_optimize_multiple_errors() {
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
    let response_json: Value = serde_json::from_slice(&response_body).unwrap();

    println!("{}", response_json);

    // Now, inspect the JSON to check for validation errors
    assert!(
        response_json.get("upper_bounds_sum").is_some(),
        "Expected 'upper_bounds_sum' validation error"
    );

    // Access the errors for 'upper_bounds'
    if let Some(lower_bounds_errors) = response_json.get("upper_bounds_sum") {
        let error_array = lower_bounds_errors.as_array().unwrap();
        let first_error = &error_array[0];
        let error_code = first_error.get("code").unwrap().as_str().unwrap();
        let error_message = first_error.get("message").unwrap().as_str().unwrap();

        assert_eq!(error_code, "upper_bounds_sum");
        assert_eq!(error_message, "Sum of upper bounds must be >= 1");
    }

    assert!(
        response_json.get("columns").is_some(),
        "Expected 'columns' validation error"
    );

    // Access the errors for 'columns'
    if let Some(columns_errors) = response_json.get("columns") {
        let error_array = columns_errors.as_array().unwrap();
        let first_error = &error_array[0];
        let error_code = first_error.get("code").unwrap().as_str().unwrap();
        let error_message = first_error.get("message").unwrap().as_str().unwrap();

        assert_eq!(error_code, "missing_key");
        assert_eq!(error_message, "Missing required columns");
    }
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