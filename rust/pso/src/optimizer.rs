use pyo3::prelude::*;
use pyo3::types::PyDict;
use polars::prelude::*;
use polars::prelude::IndexOrder;
use ndarray::Array1;
use std::collections::HashMap;

use crate::utils::{TaxBracket, calculate_taxes, QUALIFIED_TAX_BRACKETS, ORDINARY_TAX_BRACKETS, AssetConfig};
use crate::particle::{Particle, normalize_and_adjust_weights, update_particles, initialize_particles};


pub fn objective_function(
    particle: &Particle,
    df: &DataFrame,
    min_div_growth: f64,
    min_cagr: f64,
    min_yield: f64,
    required_income: f64,
    initial_capital: f64,
    div_preference: f64,
    cagr_preference: f64,
    yield_preference: f64,
    salary: f64,
    qualified_brackets: &[TaxBracket],
    non_qualified_brackets: &[TaxBracket],
) -> f64 {
    // Calculate weighted metrics
    let weighted_dividend_growth = calculate_dividend_growth(&particle, &df);
    let weighted_cagr = calculate_cagr(&particle, &df);
    let weighted_yield = calculate_yield(&particle, &df);
    let weighted_expense_ratio = calculate_expense_ratio(&particle, &df);

    // Calculate net income
    let net_income = weighted_yield * initial_capital - calculate_taxes(&particle.position(), initial_capital, df, salary, &qualified_brackets, &non_qualified_brackets);

    // Calculate penalties
    let div_growth_penalty = ((min_div_growth - weighted_dividend_growth).max(0.0) / min_div_growth * 1000.0) as f64;
    let cagr_penalty = ((min_cagr - weighted_cagr).max(0.0) / min_cagr * 1000.0) as f64;
    let yield_penalty = ((min_yield - weighted_yield).max(0.0) / min_yield * 1000.0) as f64;
    let income_penalty = ((required_income - net_income).max(0.0) / required_income * 1000.0) as f64;
    let expense_penalty = weighted_expense_ratio * 1000.0;
    dbg!(&df);
    let diversity_penalty = calculate_diversity_penalty(&particle, &df);
    dbg!(&diversity_penalty);

    // Calculate total objective value
    let objective_value = div_preference * weighted_dividend_growth
        + cagr_preference * weighted_cagr
        + yield_preference * weighted_yield
        - (div_growth_penalty
            + cagr_penalty
            + yield_penalty
            + income_penalty
            + expense_penalty
            + diversity_penalty);

    objective_value
}


fn calculate_cagr(particle: &Particle, df: &DataFrame) -> f64 {
    // Extract the "5 Yr CAGR" column and convert it to ndarray
    let cagr_series = df.column("5 Yr CAGR").unwrap();
    let cagr_values = cagr_series.f64().unwrap(); // Gets a reference to the underlying Float64Chunked

    // Convert Polars ChunkedArray to ndarray
    let cagr_ndarray: Array1<f64> = Array1::from_iter(cagr_values.into_iter().map(|v| v.unwrap_or(0.0)));

    // Perform dot product
    let weighted_cagr = particle.position().dot(&cagr_ndarray);

    weighted_cagr
}


fn calculate_dividend_growth(particle: &Particle, df: &DataFrame) -> f64 {
    // Extract the "5 Yr Dividend Growth" column and convert it to ndarray
    let dividend_growth_series = df.column("5 Yr Dividend Growth").unwrap();
    let dividend_growth_values = dividend_growth_series.f64().unwrap(); // Gets a reference to the underlying Float64Chunked

    // Convert Polars ChunkedArray to ndarray
    let dividend_growth_ndarray: Array1<f64> = Array1::from_iter(dividend_growth_values.into_iter().map(|v| v.unwrap_or(0.0)));

    // Perform dot product
    let weighted_dividend_growth = particle.position().dot(&dividend_growth_ndarray);

    weighted_dividend_growth
}


fn calculate_expense_ratio(particle: &Particle, df: &DataFrame) -> f64 {
    // Extract the "Expense Ratio" column and convert it to ndarray
    let expense_ratio_series = df.column("Expense Ratio").unwrap();
    let expense_ratio_values = expense_ratio_series.f64().unwrap(); // Gets a reference to the underlying Float64Chunked

    // Convert Polars ChunkedArray to ndarray
    let expense_ratio_ndarray: Array1<f64> = Array1::from_iter(expense_ratio_values.into_iter().map(|v| v.unwrap_or(0.0)));

    // Perform dot product
    let weighted_expense_ratio = particle.position().dot(&expense_ratio_ndarray);
    weighted_expense_ratio
}


fn calculate_yield(particle: &Particle, df: &DataFrame) -> f64 {
    // Extract the "Yield" column and convert it to ndarray
    let yield_series = df.column("Yield").unwrap();
    let yield_values = yield_series.f64().unwrap(); // Gets a reference to the underlying Float64Chunked

    // Convert Polars ChunkedArray to ndarray
    let yield_ndarray: Array1<f64> = Array1::from_iter(yield_values.into_iter().map(|v| v.unwrap_or(0.0)));

    // Perform dot product
    let weighted_yield = particle.position().dot(&yield_ndarray);
    weighted_yield
}


// Calculate the Herfindahl-Hirschman Index (HHI) as a diversity penalty
fn calculate_diversity_penalty(particle: &Particle, df: &DataFrame) -> f64 {
    let sectors = df.get_column_names()
        .into_iter()
        .filter(|name| name.contains("Sector"))
        .collect::<Vec<_>>();

    let sector_data = df.select(sectors.clone()).unwrap();
    let sector_values = sector_data.to_ndarray::<Float64Type>(IndexOrder::C).unwrap();

    let weights = particle.position().view();
    let sector_allocations = weights.dot(&sector_values);
    let sector_proportions = &sector_allocations / sector_allocations.sum();
    let hhi = sector_proportions.mapv(|x| x.powi(2)).sum();
    let num_sectors = sectors.len() as f64;
    let hhi_normalized = (hhi - 1.0 / num_sectors) / (1.0 - 1.0 / num_sectors);

    hhi_normalized * 1e3
}

#[pymodule]
fn rspso(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(optimize, m)?)?;
    Ok(())
}


#[pyfunction]
fn optimize(
    num_particles: usize,
    asset_configs: Vec<AssetConfig>,
    num_assets: usize,
    inertia: f64,
    cognitive: f64,
    social: f64,
    num_iterations: usize,
    df_dict: &Bound<'_, PyDict>,
    salary: f64,
    min_div_growth: f64,
    min_cagr: f64,
    min_yield: f64,
    required_income: f64,
    initial_capital: f64,
    div_preference: f64,
    cagr_preference: f64,
    yield_preference: f64,
    filing_status: String,
) -> PyResult<Vec<f64>> {
    let qualified_brackets = QUALIFIED_TAX_BRACKETS.get(filing_status.as_str())
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid filing status: {}", filing_status)))?;

    let non_qualified_brackets = ORDINARY_TAX_BRACKETS.get(filing_status.as_str())
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid filing status: {}", filing_status)))?;

    // Convert Python dictionary to HashMap and then to Polars DataFrame
    let mut columns: HashMap<String, Vec<f64>> = HashMap::new();
    for (key, value) in df_dict.iter() {
        let key: String = key.extract()?;
        let col_data: Vec<f64> = value.extract()?;
        columns.insert(key, col_data);
    }

    // Initially, check if there are any columns to add
    if columns.is_empty() {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "No data provided to create DataFrame"
        ));
    }

    // Create DataFrame from the first series and then add others if more exist
    let df = {
        let (first_key, first_col_data) = columns.iter().next().unwrap(); // Safe because of the check above
        let first_series = Series::new(first_key, first_col_data);
        DataFrame::new(vec![first_series]).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            format!("Failed to create DataFrame: {}", e)
        ))?
    };

    // Add other columns to the DataFrame
    for (name, data) in columns.iter() {
        if name != df.get_column_names().first().unwrap() { // Check to avoid adding the first column again
            let series = Series::new(name, data);
            df.hstack(&[series]).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                format!("Failed to horizontally stack DataFrame: {}", e)
            ))?;
        }
    }

    let mut particles = initialize_particles(num_particles, &asset_configs);
    let mut global_best = Array1::zeros(num_assets);

    for _ in 0..num_iterations {
        update_particles(&mut particles, &global_best, inertia, cognitive, social, &df, min_div_growth, min_cagr, min_yield, required_income, initial_capital, div_preference, cagr_preference, yield_preference, salary, &qualified_brackets, &non_qualified_brackets);

        // Update Global Best if any particle finds a better solution
        for particle in &mut particles {
            let score = objective_function(particle, &df, min_div_growth, min_cagr, min_yield, required_income, initial_capital, div_preference, cagr_preference, yield_preference, salary, &qualified_brackets, &non_qualified_brackets);
            if score < *particle.best_score() {
                particle.set_best_score(score);
                particle.set_best_position(particle.position().clone());
                global_best = particle.best_position().clone();
            }
        }
    }

    normalize_and_adjust_weights(&mut particles);

    // Extract the position of the best particle
    let best_particle = particles.iter().min_by(|x, y| x.best_score().partial_cmp(&y.best_score()).unwrap()).unwrap();
    Ok(best_particle.position().to_vec())
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{AssetType, AssetRange, AssetConfig};

    // Helper function to create AssetConfig
    fn create_asset_configs() -> Vec<AssetConfig> {
        let mut configs = Vec::new();
    
        let stock_range = AssetRange { min: 0.0, max: 0.05 };
        let stock_config = AssetConfig {
            asset_type: AssetType::Stock,
            range: stock_range,
        };
    
        let etf_range = AssetRange { min: 0.0, max: 0.35 };
        let etf_config = AssetConfig {
            asset_type: AssetType::ETF,
            range: etf_range,
        };
    
        // Add to configs vector
        configs.push(stock_config);
        configs.push(etf_config);
    
        configs
    }

    #[test]
    fn test_calculate_cagr() {
        let asset_configs = create_asset_configs();

        let df = DataFrame::new(vec![
            Series::new("5 Yr CAGR", &[0.10, 0.05]),
        ]).unwrap();

        let particle = &mut initialize_particles(1, &asset_configs)[0];

        particle.set_position(Array1::from(vec![0.5, 0.5]));
        particle.set_best_position(Array1::from(vec![0.5, 0.5]));
        particle.set_best_score(0.0);
        particle.set_asset_types(vec![AssetType::Stock, AssetType::ETF]);

        let cagr = calculate_cagr(&particle, &df);
        assert_eq!((cagr * 1000.0).round() / 1000.0, 0.075);
    }

    #[test]
    fn test_calculate_dividend_growth() {
        let asset_configs = create_asset_configs();

        let df = DataFrame::new(vec![
            Series::new("5 Yr Dividend Growth", &[0.10, 0.05]),
        ]).unwrap();

        let particle = &mut initialize_particles(1, &asset_configs)[0];

        particle.set_position(Array1::from(vec![0.5, 0.5]));
        particle.set_best_position(Array1::from(vec![0.5, 0.5]));
        particle.set_best_score(0.0);
        particle.set_asset_types(vec![AssetType::Stock, AssetType::ETF]);

        let dividend_growth = calculate_dividend_growth(&particle, &df);
        assert_eq!((dividend_growth * 1000.0).round() / 1000.0, 0.075);
    }

    #[test]
    fn test_calculate_expense_ratio() {
        let asset_configs = create_asset_configs();

        let df = DataFrame::new(vec![
            Series::new("Expense Ratio", &[0.01, 0.02]),
        ]).unwrap();

        let particle = &mut initialize_particles(1, &asset_configs)[0];

        particle.set_position(Array1::from(vec![0.5, 0.5]));
        particle.set_best_position(Array1::from(vec![0.5, 0.5]));
        particle.set_best_score(0.0);
        particle.set_asset_types(vec![AssetType::Stock, AssetType::ETF]);

        let expense_ratio = calculate_expense_ratio(&particle, &df);
        assert_eq!((expense_ratio * 1000.0).round() / 1000.0, 0.015);
    }

    #[test]
    fn test_calculate_yield() {
        let asset_configs = create_asset_configs();

        let df = DataFrame::new(vec![
            Series::new("Yield", &[0.02, 0.03]),
        ]).unwrap();

        let particle = &mut initialize_particles(1, &asset_configs)[0];

        particle.set_position(Array1::from(vec![0.5, 0.5]));
        particle.set_best_position(Array1::from(vec![0.5, 0.5]));
        particle.set_best_score(0.0);
        particle.set_asset_types(vec![AssetType::Stock, AssetType::ETF]);

        let div_yield = calculate_yield(&particle, &df);
        assert_eq!((div_yield * 1000.0).round() / 1000.0, 0.025);
    }

    #[test]
    fn test_calculate_diversity_penalty() {
        let asset_configs = create_asset_configs();

        let df = DataFrame::new(vec![
            Series::new("Sector 1", &[0.1, 0.2]),
            Series::new("Sector 2", &[0.3, 0.4]),
        ]).unwrap();

        let particle = &mut initialize_particles(1, &asset_configs)[0];

        particle.set_position(Array1::from(vec![0.5, 0.5]));
        particle.set_best_position(Array1::from(vec![0.5, 0.5]));
        particle.set_best_score(0.0);
        particle.set_asset_types(vec![AssetType::Stock, AssetType::ETF]);

        let diversity_penalty = calculate_diversity_penalty(&particle, &df);
        assert_eq!((diversity_penalty * 1000.0).round() / 1000.0, 160.0);
    }

    #[test]
    fn test_objective_function() {
        let asset_configs = create_asset_configs();

        let df = DataFrame::new(vec![
            Series::new("5 Yr CAGR", &[0.10, 0.05]),
            Series::new("5 Yr Dividend Growth", &[0.10, 0.05]),
            Series::new("Expense Ratio", &[0.01, 0.02]),
            Series::new("Yield", &[0.02, 0.03]),
            Series::new("Sector 1", &[0.1, 0.2]),
            Series::new("Sector 2", &[0.3, 0.4]),
            Series::new("Qualified", &[true, false])
        ]).unwrap();

        let particle = &mut initialize_particles(1, &asset_configs)[0];

        particle.set_position(Array1::from(vec![0.5, 0.5]));
        particle.set_best_position(Array1::from(vec![0.5, 0.5]));
        particle.set_best_score(0.0);
        particle.set_asset_types(vec![AssetType::Stock, AssetType::ETF]);

        let qualified_brackets = ORDINARY_TAX_BRACKETS.get("Single").unwrap();
        let non_qualified_brackets = ORDINARY_TAX_BRACKETS.get("Single").unwrap();

        let objective_value = objective_function(
            &particle,
            &df,
            0.05,
            0.07,
            0.02,
            50000.0,
            100000.0,
            0.5,
            0.3,
            0.2,
            50000.0,
            qualified_brackets,
            non_qualified_brackets,
        );

        assert_eq!((objective_value * 1.0).round() / 1.0, -1136.0);
    }
}