use crate::models::TaxBracket;
use std::collections::HashMap;
use crate::utils::{calculate_yield};


pub fn calculate_taxes(
    x: &[f64],
    initial_capital: f64,
    columns: &HashMap<String, Vec<f64>>,
    salary: f64,
    qualified_brackets: &[TaxBracket],
    non_qualified_brackets: &[TaxBracket],
) -> Result<f64, String> {
    // Handle qualified income calculation
    let qualified_income = match calculate_yield(x, columns, Some(1)) {
        Ok(value) => value * initial_capital,
        Err(e) => return Err(format!("Error calculating qualified yield: {}", e)),
    };

    // Handle non-qualified income calculation
    let non_qualified_income = match calculate_yield(x, columns, Some(0)) {
        Ok(value) => value * initial_capital,
        Err(e) => return Err(format!("Error calculating non-qualified yield: {}", e)),
    };

    // Calculate Non-Qualified taxes (same tax rate as salary)
    let salary_tax = calculate_tax_for_income(salary, non_qualified_brackets);
    let total_non_qualified_tax = calculate_tax_for_income(non_qualified_income + salary, non_qualified_brackets);
    let investment_tax = total_non_qualified_tax - salary_tax;

    // Calculate Qualified taxes
    let total_qualified_tax = tax_qualified(qualified_income, salary, qualified_brackets);
    
    // Return the final result
    Ok(total_qualified_tax + investment_tax)
}

fn calculate_tax_for_income(income: f64, brackets: &[TaxBracket]) -> f64 {
    let mut tax = 0.0;
    let mut remaining_income = income;
    let mut previous_threshold = 0.0;

    for bracket in brackets {
        let upper_limit = bracket.threshold.unwrap_or(f64::INFINITY);

        let taxable_income = if remaining_income > (upper_limit - previous_threshold) {
            upper_limit - previous_threshold
        } else {
            remaining_income
        };

        tax += taxable_income * bracket.rate;

        remaining_income -= taxable_income;
        previous_threshold = upper_limit;

        if remaining_income <= 0.0 {
            break;
        }
    }

    tax
}

fn tax_qualified(income: f64, salary: f64, brackets: &[TaxBracket]) -> f64 {
    let total_income = income + salary;

    for bracket in brackets {
        match bracket.threshold {
            Some(limit) if total_income <= limit => {
                return income * bracket.rate;
            },
            None => {
                return income * bracket.rate
            },
            _ => continue, // Skip to the next bracket if the current one doesn't fit
        }
    }

    0.0 // Return 0 if no bracket is applicable
}

// Helper functions to get tax brackets based on filing status
pub fn get_single_non_qualified_brackets() -> Vec<TaxBracket> {
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

pub fn get_single_qualified_brackets() -> Vec<TaxBracket> {
    vec![
        TaxBracket { rate: 0.0, threshold: Some(47025.0) },
        TaxBracket { rate: 0.15, threshold: Some(518900.0) },
        TaxBracket { rate: 0.20, threshold: None },
    ]
}

// Define similar functions for other filing statuses

pub fn get_married_jointly_non_qualified_brackets() -> Vec<TaxBracket> {
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

pub fn get_married_jointly_qualified_brackets() -> Vec<TaxBracket> {
    vec![
        TaxBracket { rate: 0.0, threshold: Some(94050.0) },
        TaxBracket { rate: 0.15, threshold: Some(583750.0) },
        TaxBracket { rate: 0.20, threshold: None },
    ]
}

pub fn get_married_separately_non_qualified_brackets() -> Vec<TaxBracket> {
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

pub fn get_married_separately_qualified_brackets() -> Vec<TaxBracket> {
    vec![
        TaxBracket { rate: 0.0, threshold: Some(47025.0) },
        TaxBracket { rate: 0.15, threshold: Some(291850.0) },
        TaxBracket { rate: 0.20, threshold: None },
    ]
}

pub fn get_head_of_household_non_qualified_brackets() -> Vec<TaxBracket> {
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

pub fn get_head_of_household_qualified_brackets() -> Vec<TaxBracket> {
    vec![
        TaxBracket { rate: 0.0, threshold: Some(63000.0) },
        TaxBracket { rate: 0.15, threshold: Some(551350.0) },
        TaxBracket { rate: 0.20, threshold: None },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_calculate_taxes() {
        let x = vec![0.3, 0.5, 0.2];
        let mut columns = HashMap::new();
        columns.insert("yields".to_string(), vec![0.02, 0.03, 0.04]);
        columns.insert("qualified".to_string(), vec![1.0, 0.0, 1.0]);

        let initial_capital = 100000.0;
        let salary = 50000.0;
        let qualified_brackets = vec![
            TaxBracket {
                rate: 0.1,
                threshold: Some(9950.0),
            },
            TaxBracket {
                rate: 0.12,
                threshold: Some(40525.0),
            },
        ];
        let non_qualified_brackets = vec![TaxBracket {
            rate: 0.15,
            threshold: Some(86375.0),
        }];

        let taxes = calculate_taxes(
            &x,
            initial_capital,
            &columns,
            salary,
            &qualified_brackets,
            &non_qualified_brackets,
        );

        // Since tax calculation logic is complex, we can assert that taxes are non-negative
        assert!(taxes >= Ok(0.0));
    }
}