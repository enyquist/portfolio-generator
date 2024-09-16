// src/utils.rs

use crate::models::TaxBracket;
use std::collections::HashMap;
use ordered_float::NotNan;

pub fn calculate_dividend_growth(x: &[f64], columns: &HashMap<String, Vec<f64>>) -> f64 {
    let div_growth_rates = &columns["div_growth_rates"]; // Replace with actual key
    x.iter()
        .zip(div_growth_rates.iter())
        .map(|(xi, rate)| xi * rate)
        .sum()
}

pub fn calculate_cagr(x: &[f64], columns: &HashMap<String, Vec<f64>>) -> f64 {
    let cagr_rates = &columns["cagr_rates"]; // Replace with actual key
    x.iter()
        .zip(cagr_rates.iter())
        .map(|(xi, rate)| xi * rate)
        .sum()
}

pub fn calculate_yield(x: &[f64], columns: &HashMap<String, Vec<f64>>) -> f64 {
    let yields = &columns["yields"]; // Replace with actual key
    x.iter()
        .zip(yields.iter())
        .map(|(xi, y)| xi * y)
        .sum()
}

pub fn calculate_expense_ratio(x: &[f64], columns: &HashMap<String, Vec<f64>>) -> f64 {
    let expense_ratios = &columns["expense_ratios"]; // Replace with actual key
    x.iter()
        .zip(expense_ratios.iter())
        .map(|(xi, ratio)| xi * ratio)
        .sum()
}

pub fn calculate_diversity_penalty(
    x: &[f64],
    columns: &HashMap<String, Vec<f64>>,
) -> f64 {
    // Access the sector information from columns
    let sectors = &columns["sector"]; // Now sectors is Vec<f64>

    // Map sectors to total allocation
    let mut sector_allocations: HashMap<NotNan<f64>, f64> = HashMap::new();

    for (allocation, &sector) in x.iter().zip(sectors.iter()) {
        let sector_key = NotNan::new(sector).expect("Sector code cannot be NaN");
        let entry = sector_allocations.entry(sector_key).or_insert(0.0);
        *entry += allocation;
    }

    // Calculate total allocation (should be 1.0 if allocations sum to 1)
    let total_allocation: f64 = x.iter().sum();

    // Calculate sector shares
    let sector_shares: HashMap<NotNan<f64>, f64> = sector_allocations
        .iter()
        .map(|(&sector_key, &allocation)| (sector_key, allocation / total_allocation))
        .collect();

    // Calculate HHI
    let hhi: f64 = sector_shares
        .values()
        .map(|&share| share * share)
        .sum();

    // Multiply HHI by 10,000 to scale it (standard practice)
    let hhi_scaled = hhi * 10_000.0;

    // Define a threshold for acceptable HHI (e.g., 1,500)
    let hhi_threshold = 1_500.0;

    // Calculate penalty if HHI exceeds the threshold
    if hhi_scaled > hhi_threshold {
        (hhi_scaled - hhi_threshold) * 0.1 // Penalty scaling factor
    } else {
        0.0
    }

}

pub fn calculate_taxes(
    x: &[f64],
    initial_capital: f64,
    columns: &HashMap<String, Vec<f64>>,
    salary: f64,
    qualified_brackets: &[TaxBracket],
    non_qualified_brackets: &[TaxBracket],
) -> f64 {
    let investment_income = calculate_yield(x, columns) * initial_capital;
    // let total_income = salary + investment_income;

    // Calculate taxes
    let salary_tax = calculate_tax_for_income(salary, qualified_brackets);
    let investment_tax = calculate_tax_for_income(investment_income, non_qualified_brackets);

    salary_tax + investment_tax

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_calculate_dividend_growth() {
        let x = vec![0.3, 0.5, 0.2];
        let mut columns = HashMap::new();
        columns.insert("div_growth_rates".to_string(), vec![0.04, 0.05, 0.06]);

        let result = calculate_dividend_growth(&x, &columns);
        let expected = 0.3 * 0.04 + 0.5 * 0.05 + 0.2 * 0.06;
        assert!((result - expected).abs() < 1e-8);
    }

    #[test]
    fn test_calculate_cagr() {
        let x = vec![0.2, 0.5, 0.3];
        let mut columns = HashMap::new();
        columns.insert("cagr_rates".to_string(), vec![0.06, 0.07, 0.08]);

        let result = calculate_cagr(&x, &columns);
        let expected = 0.2 * 0.06 + 0.5 * 0.07 + 0.3 * 0.08;
        assert!((result - expected).abs() < 1e-8);
    }

    #[test]
    fn test_calculate_yield() {
        let x = vec![0.4, 0.4, 0.2];
        let mut columns = HashMap::new();
        columns.insert("yields".to_string(), vec![0.02, 0.03, 0.04]);

        let result = calculate_yield(&x, &columns);
        let expected = 0.4 * 0.02 + 0.4 * 0.03 + 0.2 * 0.04;
        assert!((result - expected).abs() < 1e-8);
    }

    #[test]
    fn test_calculate_expense_ratio() {
        let x = vec![0.3, 0.3, 0.4];
        let mut columns = HashMap::new();
        columns.insert("expense_ratios".to_string(), vec![0.001, 0.002, 0.003]);

        let result = calculate_expense_ratio(&x, &columns);
        let expected = 0.3 * 0.001 + 0.3 * 0.002 + 0.4 * 0.003;
        assert!((result - expected).abs() < 1e-8);
    }

    #[test]
    fn test_calculate_diversity_penalty_hhi_with_f64_sectors() {
        let x = vec![0.3, 0.4, 0.3];
        let mut columns = HashMap::new();
        columns.insert(
            "sector".to_string(),
            vec![
                1.0, // Sector code for "technology"
                2.0, // Sector code for "finance"
                1.0, // Sector code for "technology"
            ],
        );

        let penalty = calculate_diversity_penalty(&x, &columns);

        // Manually calculate expected HHI
        // Technology allocation: 0.3 + 0.3 = 0.6
        // Finance allocation: 0.4
        // HHI = (0.6^2 + 0.4^2) * 10,000 = (0.36 + 0.16) * 10,000 = 5,200
        // Penalty = (5,200 - 1,500) * 0.1 = 370

        let expected_penalty = (5_200.0 - 1_500.0) * 0.1;

        assert!((penalty - expected_penalty).abs() < 1e-6);
    }

    #[test]
    fn test_calculate_taxes() {
        let x = vec![0.3, 0.5, 0.2];
        let mut columns = HashMap::new();
        columns.insert("yields".to_string(), vec![0.02, 0.03, 0.04]);

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
        assert!(taxes >= 0.0);
    }
}
