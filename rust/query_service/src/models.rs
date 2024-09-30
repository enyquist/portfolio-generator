// src/models.rs

use chrono::{NaiveDate, Utc, Duration};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use thiserror::Error;
use validator::Validate;

#[derive(Debug, Error)]
pub enum TickerDataError {
    #[error("Invalid date format encountered: {0}")]
    InvalidDateFormat(String),
}

// Struct to represent stock data
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct TickerData {
    #[validate(length(min = 1, max = 5))]
    pub ticker: String,
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(range(min = 0.0, max = 1.0))]
    pub dividend_yield: f64,
    pub dividend_history: Vec<(String, f64)>,  // Date, Dividend
    #[validate(range(min = 0.0, max = 1.0))]
    pub dividend_growth_rate: f64, // computed from dividend_history
    pub is_etf: bool,
    #[validate(range(min = -1.0, max = 1.0))]
    pub beta: f64,
    pub is_qualified: bool,
    pub price_history: Vec<(String, f64)>,  // Date, Price
    pub cagr: f64,  // computed from price_history
    #[validate(range(min = 0.0, max = 1.0))]
    pub expense_ratio: f64,
    pub sector: HashMap<String, f64>,  // Sector, Weight
}

impl TickerData {
    // Constructor (new) that computes dividend growth and CAGR automatically
    pub fn new(
        ticker: String,
        name: String,
        dividend_yield: f64,
        dividend_history: Vec<(String, f64)>,
        is_etf: bool,
        beta: f64,
        is_qualified: bool,
        price_history: Vec<(String, f64)>,
        expense_ratio: f64,
        sector: HashMap<String, f64>,
        current_date: Option<NaiveDate>,  // New parameter for mockable date
    ) -> Result<Self, TickerDataError> {
        let mut stock_data = TickerData {
            ticker,
            name,
            dividend_yield,
            dividend_history,
            dividend_growth_rate: 0.0,  // Placeholder, will be computed
            is_etf,
            beta,
            is_qualified,
            price_history,
            cagr: 0.0,  // Placeholder, will be computed
            expense_ratio,
            sector,
        };

        // Automatically compute dividend growth rate and CAGR
        stock_data.compute_dividend_growth(current_date)?;
        stock_data.compute_cagr(current_date)?;

        Ok(stock_data)
    }

    // Helper function to filter data from the last 5 years
    pub fn filter_last_5_years(
        history: &Vec<(String, f64)>,
        current_date: Option<NaiveDate>,
    ) -> Result<Vec<(NaiveDate, f64)>, TickerDataError> {
        let now = current_date.unwrap_or_else(|| Utc::now().naive_utc().date());
        let five_years_ago = now - Duration::days(365 * 5);

        let mut filtered_history = Vec::new();

        for (date_str, value) in history {
            match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                Ok(date) => {
                    if date >= five_years_ago {
                        filtered_history.push((date, *value));
                    }
                }
                Err(_) => {
                    return Err(TickerDataError::InvalidDateFormat(date_str.clone()));
                }
            }
        }

        Ok(filtered_history)
    }

    // Method to compute dividend growth rate (CAGR) from dividend history (last 5 years)
    pub fn compute_dividend_growth(
        &mut self,
        current_date: Option<NaiveDate>,
    ) -> Result<(), TickerDataError> {
        let filtered_history = Self::filter_last_5_years(&self.dividend_history, current_date)?;

        if let (Some((first_date, first_payout)), Some((last_date, last_payout))) =
            (filtered_history.first(), filtered_history.last())
        {
            let years = (last_date.signed_duration_since(*first_date).num_days() as f64) / 365.25;
            self.dividend_growth_rate = (last_payout / first_payout).powf(1.0 / years) - 1.0;
        }

        Ok(())
    }

    // Method to compute CAGR from price history (last 5 years)
    pub fn compute_cagr(
        &mut self,
        current_date: Option<NaiveDate>,
    ) -> Result<(), TickerDataError> {
        let filtered_history = Self::filter_last_5_years(&self.price_history, current_date)?;

        if let (Some((first_date, first_price)), Some((last_date, last_price))) =
            (filtered_history.first(), filtered_history.last())
        {
            let years = (last_date.signed_duration_since(*first_date).num_days() as f64) / 365.25;
            self.cagr = (last_price / first_price).powf(1.0 / years) - 1.0;
        }

        Ok(())
    }
}

// Custom function to convert a JSON string to f64
fn string_to_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse::<f64>().map_err(serde::de::Error::custom)
}

// Custom function to convert a JSON string to i64
fn string_to_i64<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse::<i64>().map_err(serde::de::Error::custom)
}

// Custom function to convert a JSON string to a NaiveDate
fn string_to_date<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    NaiveDate::parse_from_str(&s, "%Y-%m-%d").map_err(serde::de::Error::custom)
}

// Define Overview API structure
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct OverviewResponse {
    pub symbol: String,
    pub asset_type: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "CIK", deserialize_with = "string_to_i64")]
    pub cik: i64,
    pub exchange: String,
    pub currency: String,
    pub country: String,
    pub sector: String,
    pub industry: String,
    pub address: String,
    pub official_site: String,
    pub fiscal_year_end: String,
    #[serde(deserialize_with = "string_to_date")]
    pub latest_quarter: NaiveDate,
    #[serde(deserialize_with = "string_to_i64")]
    pub market_capitalization: i64,
    #[serde(rename = "EBITDA", deserialize_with = "string_to_i64")]
    pub ebitda: i64,
    #[serde(rename = "PERatio", deserialize_with = "string_to_f64")]
    pub pe_ratio: f64,
    #[serde(rename = "PEGRatio", deserialize_with = "string_to_f64")]
    pub peg_ratio: f64,
    #[serde(deserialize_with = "string_to_f64")]
    pub book_value: f64,
    #[serde(deserialize_with = "string_to_f64")]
    pub dividend_per_share: f64,
    #[serde(deserialize_with = "string_to_f64")]
    pub dividend_yield: f64,
    #[serde(rename = "EPS", deserialize_with = "string_to_f64")]
    pub eps: f64,
    #[serde(rename = "RevenuePerShareTTM", deserialize_with = "string_to_f64")]
    pub revenue_per_share_ttm: f64,
    #[serde(deserialize_with = "string_to_f64")]
    pub profit_margin: f64,
    #[serde(rename = "OperatingMarginTTM", deserialize_with = "string_to_f64")]
    pub operating_margin_ttm: f64,
    #[serde(rename = "ReturnOnAssetsTTM", deserialize_with = "string_to_f64")]
    pub return_on_assets_ttm: f64,
    #[serde(rename = "ReturnOnEquityTTM", deserialize_with = "string_to_f64")]
    pub return_on_equity_ttm: f64,
    #[serde(rename = "RevenueTTM", deserialize_with = "string_to_i64")]
    pub revenue_ttm: i64,
    #[serde(rename = "GrossProfitTTM", deserialize_with = "string_to_i64")]
    pub gross_profit_ttm: i64,
    #[serde(rename = "DilutedEPSTTM", deserialize_with = "string_to_f64")]
    pub diluted_eps_ttm: f64,
    #[serde(rename = "QuarterlyEarningsGrowthYOY", deserialize_with = "string_to_f64")]
    pub quarterly_earnings_growth_yoy: f64,
    #[serde(rename = "QuarterlyRevenueGrowthYOY", deserialize_with = "string_to_f64")]
    pub quarterly_revenue_growth_yoy: f64,
    #[serde(deserialize_with = "string_to_f64")]
    pub analyst_target_price: f64,
    #[serde(deserialize_with = "string_to_i64")]
    pub analyst_rating_strong_buy: i64,
    #[serde(deserialize_with = "string_to_i64")]
    pub analyst_rating_buy: i64,
    #[serde(deserialize_with = "string_to_i64")]
    pub analyst_rating_hold: i64,
    #[serde(deserialize_with = "string_to_i64")]
    pub analyst_rating_sell: i64,
    #[serde(deserialize_with = "string_to_i64")]
    pub analyst_rating_strong_sell: i64,
    #[serde(rename = "TrailingPE", deserialize_with = "string_to_f64")]
    pub trailing_pe: f64,
    #[serde(rename = "ForwardPE", deserialize_with = "string_to_f64")]
    pub forward_pe: f64,
    #[serde(rename = "PriceToSalesRatioTTM", deserialize_with = "string_to_f64")]
    pub price_to_sales_ratio_ttm: f64,
    #[serde(deserialize_with = "string_to_f64")]
    pub price_to_book_ratio: f64,
    #[serde(rename = "EVToRevenue", deserialize_with = "string_to_f64")]
    pub ev_to_revenue: f64,
    #[serde(rename = "EVToEBITDA", deserialize_with = "string_to_f64")]
    pub ev_to_ebitda: f64,
    #[serde(deserialize_with = "string_to_f64")]
    pub beta: f64,
    #[serde(rename = "52WeekHigh", deserialize_with = "string_to_f64")]
    pub week52_high: f64,
    #[serde(rename = "52WeekLow", deserialize_with = "string_to_f64")]
    pub week52_low: f64,
    #[serde(rename = "50DayMovingAverage", deserialize_with = "string_to_f64")]
    pub moving_average_50_day: f64,
    #[serde(rename = "200DayMovingAverage", deserialize_with = "string_to_f64")]
    pub moving_average_200_day: f64,
    #[serde(deserialize_with = "string_to_i64")]
    pub shares_outstanding: i64,
    #[serde(deserialize_with = "string_to_date")]
    pub dividend_date: NaiveDate,
    #[serde(deserialize_with = "string_to_date")]
    pub ex_dividend_date: NaiveDate,
}

// Define the Dividend History API structure
#[derive(Deserialize, Debug)]
pub struct DividendHistory {
    #[serde(deserialize_with = "string_to_date")]
    pub ex_dividend_date: NaiveDate,
    #[serde(deserialize_with = "string_to_date")]
    pub declaration_date: NaiveDate,
    #[serde(deserialize_with = "string_to_date")]
    pub record_date: NaiveDate,
    #[serde(deserialize_with = "string_to_date")]
    pub payment_date: NaiveDate,
    #[serde(deserialize_with = "string_to_f64")]
    pub amount: f64,
}

// Struct for overall API response
#[derive(Deserialize, Debug)]
pub struct DividendHistoryResponse {
    pub symbol: String,
    pub data: Vec<DividendHistory>,
}

// Define the NAV API structure
// Struct for the Meta Data
#[derive(Debug, Deserialize)]
pub struct MetaData {
    #[serde(rename = "1. Information")]
    pub information: String,

    #[serde(rename = "2. Symbol")]
    pub symbol: String,

    #[serde(rename = "3. Last Refreshed", deserialize_with = "string_to_date")]
    pub last_refreshed: NaiveDate,

    #[serde(rename = "4. Time Zone")]
    pub time_zone: String,
}

// Struct for the stock prices for each date
#[derive(Debug, Deserialize)]
pub struct TimeSeriesData {
    #[serde(rename = "1. open", deserialize_with = "string_to_f64")]
    pub open: f64,

    #[serde(rename = "2. high", deserialize_with = "string_to_f64")]
    pub high: f64,

    #[serde(rename = "3. low", deserialize_with = "string_to_f64")]
    pub low: f64,

    #[serde(rename = "4. close", deserialize_with = "string_to_f64")]
    pub close: f64,

    #[serde(rename = "5. volume", deserialize_with = "string_to_i64")]
    pub volume: i64,
}

// Struct for the overall response
#[derive(Debug, Deserialize)]
pub struct PriceHistoryResponse {
    #[serde(rename = "Meta Data")]
    pub meta_data: MetaData,

    #[serde(rename = "Monthly Time Series")]
    pub monthly_time_series: HashMap<String, TimeSeriesData>,  // Date -> TimeSeriesData
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_compute_dividend_growth() {
        let stock_data = TickerData::new(
            "AAPL".to_string(),  // Ticker
            "Apple Inc.".to_string(),  // Name
            0.02,  // Dividend Yield
            vec![
                ("2016-01-01".to_string(), 0.5),
                ("2017-01-01".to_string(), 0.6),
                ("2018-01-01".to_string(), 0.7),
                ("2019-01-01".to_string(), 0.8),
                ("2020-01-01".to_string(), 0.9),
            ],  // Dividend History
            false,  // ETF
            1.0,  // Beta
            true,  // Qualified Dividend
            vec![
                ("2016-01-01".to_string(), 100.0),
                ("2017-01-01".to_string(), 110.0),
                ("2018-01-01".to_string(), 120.0),
                ("2019-01-01".to_string(), 130.0),
                ("2020-01-01".to_string(), 140.0),
            ],  // Price History
            0.01,  // Expense Ratio
            HashMap::from([
                ("Technology".to_string(), 1.00),
            ]),  // Sector
            Some(NaiveDate::from_ymd_opt(2020, 1, 1).expect("REASON"))  // Mock the current date to be 2020
        ).unwrap();

        // Check if dividend growth rate is computed correctly with epsilon precision
        let epsilon = 0.0001;
        assert!((stock_data.dividend_growth_rate - 0.158292).abs() < epsilon);
    }

    #[test]
    fn test_compute_cagr_growth() {
        let stock_data = TickerData::new(
            "AAPL".to_string(),  // Ticker
            "Apple Inc.".to_string(),  // Name
            0.02,  // Dividend Yield
            vec![
                ("2016-01-01".to_string(), 0.5),
                ("2017-01-01".to_string(), 0.6),
                ("2018-01-01".to_string(), 0.7),
                ("2019-01-01".to_string(), 0.8),
                ("2020-01-01".to_string(), 0.9),
            ],  // Dividend History
            false,  // ETF
            1.0,  // Beta
            true,  // Qualified Dividend
            vec![
                ("2016-01-01".to_string(), 100.0),
                ("2017-01-01".to_string(), 110.0),
                ("2018-01-01".to_string(), 120.0),
                ("2019-01-01".to_string(), 130.0),
                ("2020-01-01".to_string(), 140.0),
            ],  // Price History
            0.01,  // Expense Ratio
            HashMap::from([
                ("Technology".to_string(), 1.00),
            ]),  // Sector
            Some(NaiveDate::from_ymd_opt(2020, 1, 1).expect("REASON"))  // Mock the current date to be 2020
        ).unwrap();

        // Check if dividend growth rate is computed correctly with epsilon precision
        let epsilon = 0.0001;
        assert!((stock_data.cagr - 0.087757).abs() < epsilon);
    }

    #[test]
    fn test_invalid_date_in_price_history() {
        let stock_data = TickerData::new(
            "AAPL".to_string(),  // Ticker
            "Apple Inc.".to_string(),  // Name
            0.02,  // Dividend Yield
            vec![
                ("2016-01-01".to_string(), 0.5),
                ("2017-01-01".to_string(), 0.6),
                ("2018-01-01".to_string(), 0.7),
                ("2019-01-01".to_string(), 0.8),
                ("2020-01-01".to_string(), 0.9),
            ],  // Dividend History
            false,  // ETF
            1.0,  // Beta
            true,  // Qualified Dividend
            vec![
                ("2016-01-01".to_string(), 100.0),
                ("2017-01-01".to_string(), 110.0),
                ("2018-01-01".to_string(), 120.0),
                ("2019-01-01".to_string(), 130.0),
                ("2020-01-01".to_string(), 140.0),
                ("invalid-date".to_string(), 150.0),  // Invalid date
            ],  // Price History
            0.01,  // Expense Ratio
            HashMap::from([
                ("Technology".to_string(), 1.00),
            ]),  // Sector
            Some(NaiveDate::from_ymd_opt(2020, 1, 1).expect("REASON"))  // Mock the current date to be 2020
        );

        assert!(stock_data.is_err());
        if let Err(TickerDataError::InvalidDateFormat(date_str)) = stock_data {
            assert_eq!(date_str, "invalid-date");
        } else {
            panic!("Expected InvalidDateFormat error");
        }
    }

    #[test]
    fn test_invalid_date_in_dividend_history() {
        let stock_data = TickerData::new(
            "AAPL".to_string(),  // Ticker
            "Apple Inc.".to_string(),  // Name
            0.02,  // Dividend Yield
            vec![
                ("2016-01-01".to_string(), 0.5),
                ("2017-01-01".to_string(), 0.6),
                ("2018-01-01".to_string(), 0.7),
                ("2019-01-01".to_string(), 0.8),
                ("invalid-date".to_string(), 0.9),
            ],  // Dividend History
            false,  // ETF
            1.0,  // Beta
            true,  // Qualified Dividend
            vec![
                ("2016-01-01".to_string(), 100.0),
                ("2017-01-01".to_string(), 110.0),
                ("2018-01-01".to_string(), 120.0),
                ("2019-01-01".to_string(), 130.0),
                ("2020-01-01".to_string(), 140.0),
            ],  // Price History
            0.01,  // Expense Ratio
            HashMap::from([
                ("Technology".to_string(), 1.00),
            ]),  // Sector
            Some(NaiveDate::from_ymd_opt(2020, 1, 1).expect("REASON"))  // Mock the current date to be 2020
        );

        assert!(stock_data.is_err());
        if let Err(TickerDataError::InvalidDateFormat(date_str)) = stock_data {
            assert_eq!(date_str, "invalid-date");
        } else {
            panic!("Expected InvalidDateFormat error");
        }
    }

    #[test]
    fn test_filter_last_5_years() {
        // History is already in the last 5 years
        let history = vec![
            ("2016-01-01".to_string(), 100.0),
            ("2017-01-01".to_string(), 110.0),
            ("2018-01-01".to_string(), 120.0),
            ("2019-01-01".to_string(), 130.0),
            ("2020-01-01".to_string(), 140.0),
        ];

        let filtered_history = TickerData::filter_last_5_years(&history, Some(NaiveDate::from_ymd_opt(2020, 1, 1).expect("REASON"))).unwrap();

        assert_eq!(filtered_history.len(), 5);
        assert_eq!(filtered_history[0].0, NaiveDate::from_ymd_opt(2016, 1, 1).expect("REASON"));
        assert_eq!(filtered_history[1].0, NaiveDate::from_ymd_opt(2017, 1, 1).expect("REASON"));
        assert_eq!(filtered_history[2].0, NaiveDate::from_ymd_opt(2018, 1, 1).expect("REASON"));
        assert_eq!(filtered_history[3].0, NaiveDate::from_ymd_opt(2019, 1, 1).expect("REASON"));
        assert_eq!(filtered_history[4].0, NaiveDate::from_ymd_opt(2020, 1, 1).expect("REASON"));

        // History is more than 5 years
        let history = vec![
            ("2015-01-01".to_string(), 90.0),
            ("2016-01-01".to_string(), 100.0),
            ("2017-01-01".to_string(), 110.0),
            ("2018-01-01".to_string(), 120.0),
            ("2019-01-01".to_string(), 130.0),
            ("2020-01-01".to_string(), 140.0),
        ];

        let filtered_history = TickerData::filter_last_5_years(&history, Some(NaiveDate::from_ymd_opt(2020, 1, 1).expect("REASON"))).unwrap();

        assert_eq!(filtered_history.len(), 5);
        assert_eq!(filtered_history[0].0, NaiveDate::from_ymd_opt(2016, 1, 1).expect("REASON"));
        assert_eq!(filtered_history[1].0, NaiveDate::from_ymd_opt(2017, 1, 1).expect("REASON"));
        assert_eq!(filtered_history[2].0, NaiveDate::from_ymd_opt(2018, 1, 1).expect("REASON"));
        assert_eq!(filtered_history[3].0, NaiveDate::from_ymd_opt(2019, 1, 1).expect("REASON"));
        assert_eq!(filtered_history[4].0, NaiveDate::from_ymd_opt(2020, 1, 1).expect("REASON"));

        // History is less than 5 years
        let history = vec![
            ("2018-01-01".to_string(), 120.0),
            ("2019-01-01".to_string(), 130.0),
            ("2020-01-01".to_string(), 140.0),
        ];

        let filtered_history = TickerData::filter_last_5_years(&history, Some(NaiveDate::from_ymd_opt(2020, 1, 1).expect("REASON"))).unwrap();

        assert_eq!(filtered_history.len(), 3);
        assert_eq!(filtered_history[0].0, NaiveDate::from_ymd_opt(2018, 1, 1).expect("REASON"));
        assert_eq!(filtered_history[1].0, NaiveDate::from_ymd_opt(2019, 1, 1).expect("REASON"));
        assert_eq!(filtered_history[2].0, NaiveDate::from_ymd_opt(2020, 1, 1).expect("REASON"));

        // History is empty
        let history = vec![];

        let filtered_history = TickerData::filter_last_5_years(&history, Some(NaiveDate::from_ymd_opt(2020, 1, 1).expect("REASON"))).unwrap();

        assert_eq!(filtered_history.len(), 0);
    }

    #[test]
    fn test_empty_dividend_history() {
        let stock_data = TickerData::new(
            "AAPL".to_string(),  // Ticker
            "Apple Inc.".to_string(),  // Name
            0.02,  // Dividend Yield
            vec![],  // Dividend History
            false,  // ETF
            1.0,  // Beta
            true,  // Qualified Dividend
            vec![
                ("2016-01-01".to_string(), 100.0),
                ("2017-01-01".to_string(), 110.0),
                ("2018-01-01".to_string(), 120.0),
                ("2019-01-01".to_string(), 130.0),
                ("2020-01-01".to_string(), 140.0),
            ],  // Price History
            0.01,  // Expense Ratio
            HashMap::from([
                ("Technology".to_string(), 1.00),
            ]),  // Sector
            Some(NaiveDate::from_ymd_opt(2020, 1, 1).expect("REASON"))  // Mock the current date to be 2020
        ).unwrap();

        // Check if dividend growth rate is computed correctly with epsilon precision
        let epsilon = 0.0001;
        assert!((stock_data.dividend_growth_rate - 0.0).abs() < epsilon);
    }

    #[test]
    fn test_empty_price_history() {
        let stock_data = TickerData::new(
            "AAPL".to_string(),  // Ticker
            "Apple Inc.".to_string(),  // Name
            0.02,  // Dividend Yield
            vec![
                ("2016-01-01".to_string(), 0.5),
                ("2017-01-01".to_string(), 0.6),
                ("2018-01-01".to_string(), 0.7),
                ("2019-01-01".to_string(), 0.8),
                ("2020-01-01".to_string(), 0.9),
            ],  // Dividend History
            false,  // ETF
            1.0,  // Beta
            true,  // Qualified Dividend
            vec![],  // Price History
            0.01,  // Expense Ratio
            HashMap::from([
                ("Technology".to_string(), 1.00),
            ]),  // Sector
            Some(NaiveDate::from_ymd_opt(2020, 1, 1).expect("REASON"))  // Mock the current date to be 2020
        ).unwrap();

        // Check if dividend growth rate is computed correctly with epsilon precision
        let epsilon = 0.0001;
        assert!((stock_data.cagr - 0.0).abs() < epsilon);
    }

    #[test]
    fn test_validate_tickerdata() {
        // Valid TickerData
        let stock_data = TickerData {
            ticker: "AAPL".to_string(),
            name: "Apple Inc.".to_string(),
            dividend_yield: 0.02,
            dividend_history: vec![
                ("2016-01-01".to_string(), 0.5),
                ("2017-01-01".to_string(), 0.6),
                ("2018-01-01".to_string(), 0.7),
                ("2019-01-01".to_string(), 0.8),
                ("2020-01-01".to_string(), 0.9),
            ],
            dividend_growth_rate: 0.0,
            is_etf: false,
            beta: 1.0,
            is_qualified: true,
            price_history: vec![
                ("2016-01-01".to_string(), 100.0),
                ("2017-01-01".to_string(), 110.0),
                ("2018-01-01".to_string(), 120.0),
                ("2019-01-01".to_string(), 130.0),
                ("2020-01-01".to_string(), 140.0),
            ],
            cagr: 0.0,
            expense_ratio: 0.01,
            sector: HashMap::from([
                ("Technology".to_string(), 1.00),
            ]),
        };

        assert!(stock_data.validate().is_ok());

        // Invalid TickerData
        let stock_data = TickerData {
            ticker: "".to_string(),
            name: "".to_string(),
            dividend_yield: -0.1,
            dividend_history: vec![],
            dividend_growth_rate: 0.0,
            is_etf: false,
            beta: 1.1,
            is_qualified: true,
            price_history: vec![],
            cagr: 0.0,
            expense_ratio: 1.1,
            sector: HashMap::new(),
        };

        assert!(stock_data.validate().is_err());
    }
}