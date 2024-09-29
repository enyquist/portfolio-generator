// src/api_client.rs

use serde::Deserialize;
use std::collections::HashMap;
use chrono::NaiveDate;

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
