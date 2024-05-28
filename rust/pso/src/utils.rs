use ndarray::Array1;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString};
use polars::frame::DataFrame;
use std::collections::HashMap;
use lazy_static::lazy_static; // For global static initialization


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AssetType {
    Stock,
    ETF,
}


impl ToPyObject for AssetType {
    fn to_object(&self, py: Python) -> PyObject {
        match self {
            AssetType::Stock => PyString::new_bound(py, "stock").to_object(py),
            AssetType::ETF => PyString::new_bound(py, "etf").to_object(py),
        }
    }
}


#[derive(Debug, Clone, Copy)]
pub struct AssetRange {
    pub min: f64,
    pub max: f64,
}

impl AssetRange {
    pub fn min(&self) -> f64 {
        self.min
    }

    pub fn max(&self) -> f64 {
        self.max
    }
}


#[derive(Debug, Clone, Copy)]
pub struct AssetConfig {
    pub asset_type: AssetType,
    pub range: AssetRange,
}


impl AssetConfig {
    pub fn asset_type(&self) -> AssetType {
        self.asset_type
    }
    pub fn range(&self) -> AssetRange {
        self.range
    }
}


impl<'source> FromPyObject<'source> for AssetConfig {
    fn extract(obj: &'source PyAny) -> PyResult<Self> {
        let dict = obj.downcast::<PyDict>()?;
        for (key, value) in dict.iter() {
            let key: String = key.extract()?;
            let value_dict: &PyDict = value.downcast()?;

            // Handle Result from get_item and then Option for actual item presence
            let min: f64 = value_dict.get_item("min")
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Error accessing 'min': {}", e)))?
                .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("min key not found in asset configuration"))?
                .extract()?;

            let max: f64 = value_dict.get_item("max")
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Error accessing 'max': {}", e)))?
                .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("max key not found in asset configuration"))?
                .extract()?;


            let asset_type = match key.as_str() {
                "stock" => AssetType::Stock,
                "etf" => AssetType::ETF,
                _ => return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Unknown asset type",
                )),
            };

            return Ok(AssetConfig {
                asset_type,
                range: AssetRange { min, max },
            });
        }

        Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Expected a dictionary with asset configuration",
        ))
    }
}


pub type TaxBracket = (Option<f64>, f64);  // (income, tax rate)


lazy_static! {
    pub static ref QUALIFIED_TAX_BRACKETS: HashMap<&'static str, Vec<TaxBracket>> = {
        let mut m = HashMap::new();
        m.insert("Single", vec![(Some(47025.0), 0.0), (Some(518900.0), 0.15), (None, 0.20)]);
        m.insert("Married Filling Jointly", vec![(Some(94050.0), 0.0), (Some(583750.0), 0.15), (None, 0.20)]);
        m.insert("Married Filling Separately", vec![(Some(47025.0), 0.0), (Some(291850.0), 0.15), (None, 0.20)]);
        m.insert("Head of Household", vec![(Some(63000.0), 0.0), (Some(551350.0), 0.15), (None, 0.20)]);
        m
    };
}


lazy_static! {
    pub static ref ORDINARY_TAX_BRACKETS: HashMap<&'static str, Vec<TaxBracket>> = {
        let mut m = HashMap::new();
        m.insert("Single", vec![
            (Some(11600.0), 0.1),
            (Some(47150.0), 0.12),
            (Some(100525.0), 0.22),
            (Some(191950.0), 0.24),
            (Some(243725.0), 0.32),
            (Some(609350.0), 0.35),
            (None, 0.37),
        ]);
        m.insert("Married Filling Jointly", vec![
            (Some(23200.0), 0.1),
            (Some(94300.0), 0.12),
            (Some(201050.0), 0.22),
            (Some(383900.0), 0.24),
            (Some(487450.0), 0.32),
            (Some(731200.0), 0.35),
            (None, 0.37),
        ]);
        m.insert("Married Filling Separately", vec![
            (Some(11600.0), 0.1),
            (Some(47150.0), 0.12),
            (Some(100525.0), 0.22),
            (Some(191950.0), 0.24),
            (Some(243725.0), 0.32),
            (Some(365600.0), 0.35),
            (None, 0.37),
        ]);
        m.insert("Head of Household", vec![
            (Some(16550.0), 0.1),
            (Some(63100.0), 0.12),
            (Some(100500.0), 0.22),
            (Some(191950.0), 0.24),
            (Some(243700.0), 0.32),
            (Some(609350.0), 0.35),
            (None, 0.37),
        ]);
        m
    };
}


pub fn calculate_taxes(
    weights: &Array1<f64>,
    capital: f64,
    df: &DataFrame,
    salary: f64,
    qualified_brackets: &[TaxBracket],
    non_qualified_brackets: &[TaxBracket],
) -> f64 {
    let qualified = df.column("ETF").unwrap().f64().unwrap().clone();
    let yield_values = df.column("Yield").unwrap().f64().unwrap();

    let qualified_income = weights.iter()
        .zip(qualified.into_iter())
        .zip(yield_values.into_iter())
        .map(|((weight, qualified), yield_value)| match (qualified, yield_value) {
            (Some(1.0), Some(yield_value)) => weight * yield_value,  // Check both Option values
            _ => 0.0,  // Handle all other cases including None
        })
        .sum::<f64>() * capital;  // Calculate total qualified income

    let non_qualified_income = weights.iter()
        .zip(qualified.into_iter())
        .zip(yield_values.into_iter())
        .map(|((weight, qualified), yield_value)| {
            match (qualified, yield_value) {
                (Some(0.0), Some(yield_val)) => weight * yield_val,  // Only calculate if qualified is false and yield_val is Some
                _ => 0.0,  // Handle all other cases including None values
            }
        })
        .sum::<f64>() * capital;  // Calculate total non-qualified income

    tax_qualified(qualified_income, salary, qualified_brackets) + tax_non_qualified(non_qualified_income, salary, non_qualified_brackets)
}


fn tax_qualified(income: f64, salary: f64, brackets: &[TaxBracket]) -> f64 {
    let mut tax_rate = 0.0;
    let total_income = income + salary;

    for (limit, rate) in brackets {
        match limit {
            Some(l) if total_income <= *l => {
                tax_rate = *rate;
                break; // Break the loop once the correct bracket is found
            },
            None => {
                tax_rate = *rate;
                break; // Break the loop if there is no upper limit
            },
            _ => continue, // Skip to the next bracket if the current one doesn't fit
        }
    }

    income * tax_rate
}


fn calculate_tax(income: f64, brackets: &[TaxBracket]) -> f64 {
    let mut tax_owed = 0.0;
    let mut previous_limit = 0.0;

    for (limit, rate) in brackets {
        // Check if theres an upper limit or assume the given income if None
        let upper_limit = limit.unwrap_or(f64::INFINITY);

        if income > previous_limit {
            let taxable_income = (income.min(upper_limit) - previous_limit).max(0.0);
            tax_owed += taxable_income * rate;
            previous_limit = upper_limit;
        } else {
            break;
        }
    }

    tax_owed
}


fn tax_non_qualified(income: f64, salary: f64, brackets: &[TaxBracket]) -> f64 {
    let total_income = salary + income;
    let total_tax_owed = calculate_tax(total_income, brackets);
    let salary_tax_owed = calculate_tax(salary, brackets);
    let tax_on_dividends = total_tax_owed - salary_tax_owed;

    tax_on_dividends
}


#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;
    use ndarray::array;

    // Helper functions to create a DataFrame
    fn create_test_dataframe() -> DataFrame {
        let qualified = BooleanChunked::from_slice("Qualified", &[true, false]);
        let etf = Float64Chunked::from_slice("ETF", &[1.0, 0.0]);
        let yields = Float64Chunked::from_slice("Yield", &[0.02, 0.03]);
        DataFrame::new(vec![
            qualified.into_series(),
            yields.into_series(),
            etf.into_series(),
        ]).unwrap()
    }

    #[test]
    fn test_calculate_tax() {
        let filing_status = "Single";

        let non_qualified_brackets = ORDINARY_TAX_BRACKETS.get(filing_status)
            .expect("Filing status not found in ordinary tax brackets");

        let income = 100000.0;

        let tax = calculate_tax(income, non_qualified_brackets);
        let expected_tax = 11600.0 * 0.10 + (47150.0 - 11600.0) * 0.12 + (income - 47150.0) * 0.22;
        assert_eq!(tax, expected_tax);
    }

    #[test]
    fn test_calculate_taxes() {
        let weights = array![0.5, 0.5];
        let df = create_test_dataframe();
        let capital = 100000.0;
        let salary = 50000.0;
        let filing_status = "Single";

        let qualified_brackets = QUALIFIED_TAX_BRACKETS.get(filing_status)
            .expect("Filing status not found in qualified tax brackets");

        let non_qualified_brackets = ORDINARY_TAX_BRACKETS.get(filing_status)
            .expect("Filing status not found in ordinary tax brackets");
            
        let taxes = calculate_taxes(&weights, capital, &df, salary, &qualified_brackets, &non_qualified_brackets);
        
        let expected_taxes = 150.0 + 330.0;  // 150 from qualified and 330 from non-qualified
        assert_eq!(taxes, expected_taxes);
    }
}