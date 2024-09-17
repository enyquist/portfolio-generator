// src/objective.rs

use crate::models::OptimizationParams;
use crate::utils::{
    calculate_cagr, calculate_diversity_penalty, calculate_dividend_growth,
    calculate_expense_ratio, calculate_yield,
};
use crate::taxbrackets::calculate_taxes;


pub fn objective_function(
    x: &[f64],
    grad: Option<&mut [f64]>,
    params: &mut OptimizationParams,
) -> f64 {
    // Compute the objective value
    let obj_value = calculate_objective(x, params);

    // If gradient is requested, compute numerical gradient
    if let Some(grad) = grad {
        let eps = 1e-8;
        for i in 0..x.len() {
            let mut x_eps = x.to_vec();
            x_eps[i] += eps;
            let f_eps = calculate_objective(&x_eps, params);
            grad[i] = (f_eps - obj_value) / eps;
        }
    }

    obj_value
}

fn calculate_objective(x: &[f64], params: &OptimizationParams) -> f64 {
    // Compute weighted metrics
    let weighted_dividend_growth = calculate_dividend_growth(x, &params.columns);
    let weighted_cagr = calculate_cagr(x, &params.columns);
    let weighted_yield = calculate_yield(x, &params.columns, None).unwrap();
    let weighted_expense_ratio = calculate_expense_ratio(x, &params.columns);

    // Handle the Result from calculate_taxes using match
    let net_income = match calculate_taxes(
        x,
        params.initial_capital,
        &params.columns,
        params.salary,
        &params.qualified_brackets,
        &params.non_qualified_brackets,
    ) {
        Ok(tax) => weighted_yield * params.initial_capital - tax,
        Err(e) => {
            eprintln!("Error calculating taxes: {}", e);
            return f64::MAX; // Return a high penalty value, or handle it differently
        }
    };

    // Calculate penalties
    let div_growth_penalty = (params.min_div_growth - weighted_dividend_growth).max(0.0)
        / params.min_div_growth
        * 1000.0;
    let cagr_penalty =
        (params.min_cagr - weighted_cagr).max(0.0) / params.min_cagr * 1000.0;
    let yield_penalty =
        (params.min_yield - weighted_yield).max(0.0) / params.min_yield * 1000.0;
    let income_penalty = (params.required_income - net_income).max(0.0)
        / params.required_income
        * 1000.0;
    let expense_penalty = weighted_expense_ratio * 1000.0;
    let diversity_penalty = calculate_diversity_penalty(x, &params.columns);

    // Calculate gains from dividends, CAGR, and yield
    let gains = params.div_preference * weighted_dividend_growth
        + params.cagr_preference * weighted_cagr
        + params.yield_preference * weighted_yield;

    // Calculate total penalties
    let penalties = div_growth_penalty
        + cagr_penalty
        + yield_penalty
        + income_penalty
        + expense_penalty
        + diversity_penalty;

    // Calculate total objective value (we minimize this value)
    -gains + penalties
}
