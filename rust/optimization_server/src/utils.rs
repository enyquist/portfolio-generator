// src/utils.rs

use std::collections::HashMap;
use ordered_float::NotNan;

pub fn calculate_dividend_growth(x: &[f64], columns: &HashMap<String, Vec<f64>>) -> f64 {
    let div_growth_rates = &columns["dividend_growth_rates"]; // Replace with actual key
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

pub fn calculate_yield(x: &[f64], columns: &HashMap<String, Vec<f64>>, filter: Option<i32>) -> Result<f64, String> {
    let yields = &columns["yields"]; // Replace with actual key
    let qualified = &columns["qualified"]; // Assuming "qualified" is also stored in columns as Vec<f64>

    let filtered_data: Vec<(f64, f64)> = match filter {
        None => x.iter().cloned().zip(yields.iter().cloned()).collect(),
        Some(0) => x.iter()
            .cloned()
            .zip(yields.iter().cloned())
            .zip(qualified.iter().cloned())
            .filter(|(_, q)| q == &0.0)
            .map(|((xi, y), _)| (xi, y))
            .collect(),
        Some(1) => x.iter()
            .cloned()
            .zip(yields.iter().cloned())
            .zip(qualified.iter().cloned())
            .filter(|(_, q)| q == &1.0)
            .map(|((xi, y), _)| (xi, y))
            .collect(),
        _ => return Err(String::from("Invalid filter value, must be None, 0, or 1")),
    };

    Ok(filtered_data
        .iter()
        .map(|(xi, y)| xi * y)
        .sum())
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



#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_calculate_dividend_growth() {
        let x = vec![0.3, 0.5, 0.2];
        let mut columns = HashMap::new();
        columns.insert("dividend_growth_rates".to_string(), vec![0.04, 0.05, 0.06]);

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
        columns.insert("qualified".to_string(), vec![1.0, 0.0, 1.0]);

        let result = calculate_yield(&x, &columns, None).unwrap();
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
    fn test_calculate_diversity_penalty() {
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

}
