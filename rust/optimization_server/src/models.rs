// src/models.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize)]
pub struct OptimizationRequest {
    pub dimension: usize,
    pub lower_bounds: Vec<f64>,
    pub upper_bounds: Vec<f64>,

    // Objective function parameters
    pub initial_capital: f64,
    pub salary: f64,
    pub required_income: f64,
    pub min_div_growth: f64,
    pub min_cagr: f64,
    pub min_yield: f64,
    pub div_preference: f64,
    pub cagr_preference: f64,
    pub yield_preference: f64,

    // Filing status
    pub filing_status: FilingStatus,

    // Columns as key-value pairs
    pub columns: HashMap<String, Vec<f64>>,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum FilingStatus {
    Single,
    MarriedFilingJointly,
    MarriedFilingSeparately,
    HeadOfHousehold,
}

#[derive(Deserialize, Clone)]
pub struct TaxBracket {
    pub rate: f64,
    pub threshold: Option<f64>,
}

#[derive(Serialize, Deserialize)]
pub struct OptimizationResult {
    pub success: bool,
    pub x: Option<Vec<f64>>,
    pub objective_value: Option<f64>,
    pub message: String,
}

#[derive(Clone)]
pub struct OptimizationParams {
    pub initial_capital: f64,
    pub salary: f64,
    pub required_income: f64,
    pub min_div_growth: f64,
    pub min_cagr: f64,
    pub min_yield: f64,
    pub div_preference: f64,
    pub cagr_preference: f64,
    pub yield_preference: f64,
    pub qualified_brackets: Vec<TaxBracket>,
    pub non_qualified_brackets: Vec<TaxBracket>,
    pub columns: HashMap<String, Vec<f64>>,
}
