// src/models.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::borrow::Cow::Borrowed;
use validator::{Validate, ValidationError};

fn validate_filing_status(filing_status: &FilingStatus) -> Result<(), ValidationError> {
    match filing_status {
        FilingStatus::Single => Ok(()),
        FilingStatus::MarriedFilingJointly => Ok(()),
        FilingStatus::MarriedFilingSeparately => Ok(()),
        FilingStatus::HeadOfHousehold => Ok(()),
    }
}

fn validate_columns(columns: &HashMap<String, Vec<f64>>) -> Result<(), ValidationError> {
    let required_keys = [
        "dividend_growth_rates",
        "cagr_rates",
        "yields",
        "expense_ratios",
        "sector",
    ];

    for &key in &required_keys {
        if !columns.contains_key(key) {
            let mut error = ValidationError::new("missing_key");
            error.add_param("key".into(), &key);
            return Err(error);
        }
    }

    Ok(())
}

#[derive(Deserialize, Serialize)]
pub struct OptimizationRequest {
    pub dimension: usize,

    // Bounds
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

// Implement Validate for OptimizationRequest
impl Validate for OptimizationRequest {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        let mut errors = validator::ValidationErrors::new();

        // Validate 'dimension'
        if self.dimension < 1 {
            let error = ValidationError::new("range");
            errors.add("dimension", error.with_message(Borrowed("Dimension must be >= 1")));
        }

        // Validate 'initial_capital'
        if self.initial_capital < 0.0 {
            let error = ValidationError::new("range");
            errors.add("initial_capital", error.with_message(Borrowed("Initial capital must be >= 0")));
        }

        // Validate 'salary'
        if self.salary < 0.0 {
            let error = ValidationError::new("range");
            errors.add("salary", error.with_message(Borrowed("Salary must be >= 0")));
        }

        // Validate 'required_income'
        if self.required_income < 0.0 {
            let error = ValidationError::new("range");
            errors.add("required_income", error.with_message(Borrowed("Required income must be >= 0")));
        }

        // Validate 'min_div_growth'
        if self.min_div_growth < 0.0  || self.min_div_growth > 1.0 {
            let error = ValidationError::new("range");
            errors.add("min_div_growth", error.with_message(Borrowed("Minimum dividend growth must be in [0, 1]")));
        }

        // Validate 'min_cagr'
        if self.min_cagr < 0.0  || self.min_cagr > 1.0 {
            let error = ValidationError::new("range");
            errors.add("min_cagr", error.with_message(Borrowed("Minimum CAGR must be in [0, 1]")));
        }

        // Validate 'min_yield'
        if self.min_yield < 0.0  || self.min_yield > 1.0 {
            let error = ValidationError::new("range");
            errors.add("min_yield", error.with_message(Borrowed("Minimum yield must be in [0, 1]")));
        }

        // Validate 'div_preference'
        if self.div_preference < 0.0 || self.div_preference > 1.0 {
            let error = ValidationError::new("range");
            errors.add("div_preference", error.with_message(Borrowed("Dividend preference must be in [0, 1]")));
        }

        // Validate 'cagr_preference'
        if self.cagr_preference < 0.0 || self.cagr_preference > 1.0 {
            let error = ValidationError::new("range");
            errors.add("cagr_preference", error.with_message(Borrowed("CAGR preference must be in [0, 1]")));
        }

        // Validate 'yield_preference'
        if self.yield_preference < 0.0 || self.yield_preference > 1.0 {
            let error = ValidationError::new("range");
            errors.add("yield_preference", error.with_message(Borrowed("Yield preference must be in [0, 1]")));
        }

        // Validate 'filing_status'
        if let Err(e) = validate_filing_status(&self.filing_status) {
            errors.add("filing_status", e);
        }

        // Validate bounds lengths
        if self.lower_bounds.len() != self.dimension {
            let mut error = ValidationError::new("lower_bounds_length_mismatch");
            error.add_param(Borrowed("expected"), &self.dimension);
            error.add_param(Borrowed("found"), &self.lower_bounds.len());
            errors.add("lower_bounds", error.with_message(Borrowed("Bounds size does not match dimension")));

        }
        if self.upper_bounds.len() != self.dimension {
            let mut error = ValidationError::new("upper_bounds_length_mismatch");
            error.add_param(Borrowed("expected"), &self.dimension);
            error.add_param(Borrowed("found"), &self.upper_bounds.len());
            errors.add("upper_bounds", error.with_message(Borrowed("Bounds size does not match dimension")));
        }

        // Validate sum of upper bounds
        let sum_upper_bounds: f64 = self.upper_bounds.iter().sum();
        if sum_upper_bounds < 1.0 {
            let mut error = ValidationError::new("upper_bounds_sum");
            error.add_param(Borrowed("sum"), &sum_upper_bounds);
            errors.add("upper_bounds_sum", error.with_message(Borrowed("Sum of upper bounds must be >= 1")));
        }

        // Validate that div_preference + cagr_preference + yield_preference == 1
        let sum_preferences = self.div_preference + self.cagr_preference + self.yield_preference;
        if (sum_preferences - 1.0).abs() > f64::EPSILON {
            let mut error = ValidationError::new("preference_sum");
            error.add_param(Borrowed("sum"), &sum_preferences);
            errors.add("div_preference", error.clone().with_message(Borrowed("Sum of preferences must be 1")));
            errors.add("cagr_preference", error.clone().with_message(Borrowed("Sum of preferences must be 1")));
            errors.add("yield_preference", error.clone().with_message(Borrowed("Sum of preferences must be 1")));
        }

        // Validate columns
        if let Err(e) = validate_columns(&self.columns) {
            errors.add("columns", e.with_message(Borrowed("Missing required columns")));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
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
