// src/handlers.rs

use crate::models::{OptimizationParams, OptimizationRequest, OptimizationResult, FilingStatus, TaxBracket};
use crate::objective::objective_function;
use actix_web::{post, web, get, HttpResponse, Responder};
use nlopt::{Algorithm, Nlopt, Target};
use validator::Validate;

#[post("/optimize")]
pub async fn optimize(params: web::Json<OptimizationRequest>) -> impl Responder {
    // Extract parameters
    let dimension = params.dimension;
    let lower_bounds = &params.lower_bounds;
    let upper_bounds = &params.upper_bounds;

     // Perform validation
     if let Err(validation_errors) = params.validate() {
        return HttpResponse::BadRequest().json(validation_errors);
    }

    // Define tax brackets based on filing status
    let (qualified_brackets, non_qualified_brackets) = match params.filing_status {
        FilingStatus::Single => (get_single_qualified_brackets(), get_single_non_qualified_brackets()),
        FilingStatus::MarriedFilingJointly => (
            get_married_jointly_qualified_brackets(),
            get_married_jointly_non_qualified_brackets(),
        ),
        FilingStatus::MarriedFilingSeparately => (
            get_married_separately_qualified_brackets(),
            get_married_separately_non_qualified_brackets(),
        ),
        FilingStatus::HeadOfHousehold => (
            get_head_of_household_qualified_brackets(),
            get_head_of_household_non_qualified_brackets(),
        ),
    };

    // Prepare optimization parameters
    let opt_params = OptimizationParams {
        initial_capital: params.initial_capital,
        salary: params.salary,
        required_income: params.required_income,
        min_div_growth: params.min_div_growth,
        min_cagr: params.min_cagr,
        min_yield: params.min_yield,
        div_preference: params.div_preference,
        cagr_preference: params.cagr_preference,
        yield_preference: params.yield_preference,
        qualified_brackets,
        non_qualified_brackets,
        columns: params.columns.clone(),
    };

    // Define the objective function closure
    let obj_func = |x: &[f64], grad: Option<&mut [f64]>, user_data: &mut OptimizationParams| {
        objective_function(x, grad, user_data)
    };

    // Create the optimizer
    let mut opt = Nlopt::new(
        Algorithm::Slsqp,
        dimension,
        obj_func,
        Target::Minimize,
        opt_params, // Pass mutable reference
    );

    // Set bounds
    if let Err(err) = opt.set_lower_bounds(lower_bounds) {
        return HttpResponse::BadRequest().json(OptimizationResult {
            success: false,
            x: None,
            objective_value: None,
            message: format!("Failed to set lower bounds: {:?}", err),
        });
    }
    if let Err(err) = opt.set_upper_bounds(upper_bounds) {
        return HttpResponse::BadRequest().json(OptimizationResult {
            success: false,
            x: None,
            objective_value: None,
            message: format!("Failed to set upper bounds: {:?}", err),
        });
    }

    // Add equality constraint: sum of x_i == 1
    let sum_constraint = |x: &[f64], grad: Option<&mut [f64]>, _user_data: &mut ()| {
        let sum: f64 = x.iter().sum();
        if let Some(grad) = grad {
            for g in grad.iter_mut() {
                *g = 1.0;
            }
        }
        sum - 1.0
    };

    if let Err(err) = opt.add_equality_constraint(sum_constraint, (), 1e-8) {
        return HttpResponse::InternalServerError().json(OptimizationResult {
            success: false,
            x: None,
            objective_value: None,
            message: format!("Failed to add equality constraint: {:?}", err),
        });
    }

    // Set optimization parameters
    if let Err(err) = opt.set_xtol_rel(1e-6) {
        return HttpResponse::InternalServerError().json(OptimizationResult {
            success: false,
            x: None,
            objective_value: None,
            message: format!("Failed to set xtol_rel: {:?}", err),
        });
    }

    // Initial guess
    let mut x = vec![1.0 / dimension as f64; dimension];

    // Run the optimization
    match opt.optimize(&mut x) {
        Ok((success_state, obj_val)) => HttpResponse::Ok().json(OptimizationResult {
            success: true,
            x: Some(x.clone()), // x has been modified to contain the optimized variables
            objective_value: Some(obj_val),
            message: format!("Optimization succeeded with status: {:?}", success_state),
        }),
        Err(err) => HttpResponse::Ok().json(OptimizationResult {
            success: false,
            x: None,
            objective_value: None,
            message: format!("Optimization failed: {:?}", err),
        }),
    }
}

// Helper functions to get tax brackets based on filing status
fn get_single_non_qualified_brackets() -> Vec<TaxBracket> {
    vec![
        TaxBracket { rate: 0.0, threshold: Some(11600.0) },
        TaxBracket { rate: 0.12, threshold: Some(47150.0) },
        TaxBracket { rate: 0.22, threshold: Some(100526.0) },
        TaxBracket { rate: 0.24, threshold: Some(191950.0) },
        TaxBracket { rate: 0.32, threshold: Some(243725.0) },
        TaxBracket { rate: 0.35, threshold: Some(609350.0) },
        TaxBracket { rate: 0.37, threshold: None }, // No upper limit
    ]
}

fn get_single_qualified_brackets() -> Vec<TaxBracket> {
    vec![
        TaxBracket { rate: 0.0, threshold: Some(47025.0) },
        TaxBracket { rate: 0.15, threshold: Some(518900.0) },
        TaxBracket { rate: 0.20, threshold: None },
    ]
}

// Define similar functions for other filing statuses

fn get_married_jointly_non_qualified_brackets() -> Vec<TaxBracket> {
    vec![
        TaxBracket { rate: 0.0, threshold: Some(23200.0) },
        TaxBracket { rate: 0.12, threshold: Some(94300.0) },
        TaxBracket { rate: 0.22, threshold: Some(201050.0) },
        TaxBracket { rate: 0.24, threshold: Some(383900.0) },
        TaxBracket { rate: 0.32, threshold: Some(487450.0) },
        TaxBracket { rate: 0.35, threshold: Some(731200.0) },
        TaxBracket { rate: 0.37, threshold: None },
    ]
}

fn get_married_jointly_qualified_brackets() -> Vec<TaxBracket> {
    vec![
        TaxBracket { rate: 0.0, threshold: Some(94050.0) },
        TaxBracket { rate: 0.15, threshold: Some(583750.0) },
        TaxBracket { rate: 0.20, threshold: None },
    ]
}

fn get_married_separately_non_qualified_brackets() -> Vec<TaxBracket> {
    vec![
        TaxBracket { rate: 0.0, threshold: Some(11600.0) },
        TaxBracket { rate: 0.12, threshold: Some(47150.0) },
        TaxBracket { rate: 0.22, threshold: Some(100525.0) },
        TaxBracket { rate: 0.24, threshold: Some(191950.0) },
        TaxBracket { rate: 0.32, threshold: Some(243725.0) },
        TaxBracket { rate: 0.35, threshold: Some(365600.0) },
        TaxBracket { rate: 0.37, threshold: None },
    ]
}

fn get_married_separately_qualified_brackets() -> Vec<TaxBracket> {
    vec![
        TaxBracket { rate: 0.0, threshold: Some(47025.0) },
        TaxBracket { rate: 0.15, threshold: Some(291850.0) },
        TaxBracket { rate: 0.20, threshold: None },
    ]
}

fn get_head_of_household_non_qualified_brackets() -> Vec<TaxBracket> {
    vec![
        TaxBracket { rate: 0.0, threshold: Some(16550.0) },
        TaxBracket { rate: 0.12, threshold: Some(63100.0) },
        TaxBracket { rate: 0.22, threshold: Some(100500.0) },
        TaxBracket { rate: 0.24, threshold: Some(191950.0) },
        TaxBracket { rate: 0.32, threshold: Some(243700.0) },
        TaxBracket { rate: 0.35, threshold: Some(609350.0) },
        TaxBracket { rate: 0.37, threshold: None },
    ]
}

fn get_head_of_household_qualified_brackets() -> Vec<TaxBracket> {
    vec![
        TaxBracket { rate: 0.0, threshold: Some(63000.0) },
        TaxBracket { rate: 0.15, threshold: Some(551350.0) },
        TaxBracket { rate: 0.20, threshold: None },
    ]
}

#[get("/health")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_missing_column_keys() {
        let request = OptimizationRequest {
            dimension: 3,
            lower_bounds: vec![0.0; 3],
            upper_bounds: vec![1.0; 3],
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
            columns: HashMap::new(), // Empty columns
        };

        let result = request.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("columns"));
    }
}
