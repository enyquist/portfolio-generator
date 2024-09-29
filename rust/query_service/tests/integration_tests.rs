// tests/integration_tests.rs

use reqwest::Client;
use mockito::{mock, Matcher};
use std::error::Error;
use query_service::models::{OverviewResponse, DividendHistoryResponse, PriceHistoryResponse};
use chrono::NaiveDate;

#[tokio::test]
async fn test_overview_response() -> Result<(), Box<dyn Error>> {
    // Define mock server response
    let mock_server_response = r#"
    {
        "Symbol": "IBM",
        "AssetType": "Common Stock",
        "Name": "International Business Machines",
        "Description": "International Business Machines Corporation (IBM) is an American multinational technology company",
        "CIK": "51143",
        "Exchange": "NYSE",
        "Currency": "USD",
        "Country": "USA",
        "Sector": "TECHNOLOGY",
        "Industry": "COMPUTER & OFFICE EQUIPMENT",
        "Address": "1 NEW ORCHARD ROAD, ARMONK, NY, US",
        "OfficialSite": "https://www.ibm.com",
        "FiscalYearEnd": "December",
        "LatestQuarter": "2021-06-30",
        "MarketCapitalization": "197991563000",
        "EBITDA": "14625000000",
        "PERatio": "23.7",
        "PEGRatio": "4.173",
        "BookValue": "26.08",
        "DividendPerShare": "6.65",
        "DividendYield": "0.0311",
        "EPS": "9.07",
        "RevenuePerShareTTM": "68.06",
        "ProfitMargin": "0.135",
        "OperatingMarginTTM": "0.149",
        "ReturnOnAssetsTTM": "0.047",
        "ReturnOnEquityTTM": "0.362",
        "RevenueTTM": "62363001000",
        "GrossProfitTTM": "32688000000",
        "DilutedEPSTTM": "9.07",
        "QuarterlyEarningsGrowthYOY": "0.141",
        "QuarterlyRevenueGrowthYOY": "0.019",
        "AnalystTargetPrice": "194.43",
        "AnalystRatingStrongBuy": "4",
        "AnalystRatingBuy": "5",
        "AnalystRatingHold": "10",
        "AnalystRatingSell": "3",
        "AnalystRatingStrongSell": "0",
        "TrailingPE": "23.7",
        "ForwardPE": "20.04",
        "PriceToSalesRatioTTM": "3.175",
        "PriceToBookRatio": "8.24",
        "EVToRevenue": "3.915",
        "EVToEBITDA": "16.13",
        "Beta": "0.693",
        "52WeekHigh": "218.84",
        "52WeekLow": "130.68",
        "50DayMovingAverage": "194.62",
        "200DayMovingAverage": "180.54",
        "SharesOutstanding": "921148000",
        "DividendDate": "2024-09-10",
        "ExDividendDate": "2024-08-09"
    }"#;

    // Set up mock server with mockito
    let _mock = mock("GET", "/query")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("function".into(), "OVERVIEW".into()),
            Matcher::UrlEncoded("symbol".into(), "IBM".into()),
            Matcher::UrlEncoded("apikey".into(), "demo".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_server_response)
        .create();

    // Use mockito's server URL
    let mock_url = &mockito::server_url();

    // Make a request using reqwest to the mock server
    let client = Client::new();
    let url = format!("{}/query?function=OVERVIEW&symbol=IBM&apikey=demo", mock_url);
    let response = client.get(&url).send().await?;

    // Deserialize the response into OverviewResponse
    let overview: OverviewResponse = response.json().await?;

    // Assertions to ensure the data was correctly parsed
    assert_eq!(overview.symbol, "IBM");
    assert_eq!(overview.asset_type, "Common Stock");
    assert_eq!(overview.name, "International Business Machines");
    assert_eq!(overview.description, "International Business Machines Corporation (IBM) is an American multinational technology company");
    assert_eq!(overview.cik, 51143);
    assert_eq!(overview.exchange, "NYSE");
    assert_eq!(overview.currency, "USD");
    assert_eq!(overview.country, "USA");
    assert_eq!(overview.sector, "TECHNOLOGY");
    assert_eq!(overview.industry, "COMPUTER & OFFICE EQUIPMENT");
    assert_eq!(overview.address, "1 NEW ORCHARD ROAD, ARMONK, NY, US");
    assert_eq!(overview.official_site, "https://www.ibm.com");
    assert_eq!(overview.fiscal_year_end, "December");
    assert_eq!(overview.latest_quarter, NaiveDate::parse_from_str("2021-06-30", "%Y-%m-%d").map_err(|_| "Failed to parse date")?);
    assert_eq!(overview.market_capitalization, 197991563000);
    assert_eq!(overview.ebitda, 14625000000);
    assert_eq!(overview.pe_ratio, 23.7);
    assert_eq!(overview.peg_ratio, 4.173);
    assert_eq!(overview.book_value, 26.08);
    assert_eq!(overview.dividend_per_share, 6.65);
    assert_eq!(overview.dividend_yield, 0.0311);
    assert_eq!(overview.eps, 9.07);
    assert_eq!(overview.revenue_per_share_ttm, 68.06);
    assert_eq!(overview.profit_margin, 0.135);
    assert_eq!(overview.operating_margin_ttm, 0.149);
    assert_eq!(overview.return_on_assets_ttm, 0.047);
    assert_eq!(overview.return_on_equity_ttm, 0.362);
    assert_eq!(overview.revenue_ttm, 62363001000);
    assert_eq!(overview.gross_profit_ttm, 32688000000);
    assert_eq!(overview.diluted_eps_ttm, 9.07);
    assert_eq!(overview.quarterly_earnings_growth_yoy, 0.141);
    assert_eq!(overview.quarterly_revenue_growth_yoy, 0.019);
    assert_eq!(overview.analyst_target_price, 194.43);
    assert_eq!(overview.analyst_rating_strong_buy, 4);
    assert_eq!(overview.analyst_rating_buy, 5);
    assert_eq!(overview.analyst_rating_hold, 10);
    assert_eq!(overview.analyst_rating_sell, 3);
    assert_eq!(overview.analyst_rating_strong_sell, 0);
    assert_eq!(overview.trailing_pe, 23.7);
    assert_eq!(overview.forward_pe, 20.04);
    assert_eq!(overview.price_to_sales_ratio_ttm, 3.175);
    assert_eq!(overview.price_to_book_ratio, 8.24);
    assert_eq!(overview.ev_to_revenue, 3.915);
    assert_eq!(overview.ev_to_ebitda, 16.13);
    assert_eq!(overview.beta, 0.693);
    assert_eq!(overview.week52_high, 218.84);
    assert_eq!(overview.week52_low, 130.68);
    assert_eq!(overview.moving_average_50_day, 194.62);
    assert_eq!(overview.moving_average_200_day, 180.54);
    assert_eq!(overview.shares_outstanding, 921148000);
    assert_eq!(overview.dividend_date, NaiveDate::parse_from_str("2024-09-10", "%Y-%m-%d").map_err(|_| "Failed to parse date")?);
    assert_eq!(overview.ex_dividend_date, NaiveDate::parse_from_str("2024-08-09", "%Y-%m-%d").map_err(|_| "Failed to parse date")?);

    Ok(())
}

#[tokio::test]
async fn test_dividend_response() -> Result<(), Box<dyn Error>> {
    // Define Mock Server Response
    let mock_server_response = r#"
    {
        "symbol": "AAPL",
        "data": [
            {
                "ex_dividend_date": "2021-08-06",
                "declaration_date": "2021-07-27",
                "record_date": "2021-08-09",
                "payment_date": "2021-08-12",
                "amount": "0.22"
            },
            {
                "ex_dividend_date": "2021-05-07",
                "declaration_date": "2021-04-28",
                "record_date": "2021-05-10",
                "payment_date": "2021-05-13",
                "amount": "0.22"
            }
        ]
    }"#;

    // Set up mock server with mockito
    let _mock = mock("GET", "/query")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("function".into(), "OVERVIEW".into()),
            Matcher::UrlEncoded("symbol".into(), "AAPL".into()),
            Matcher::UrlEncoded("apikey".into(), "demo".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_server_response)
        .create();

    // Use mockito's server URL
    let mock_url = &mockito::server_url();

    // Make a request using reqwest to the mock server
    let client = Client::new();
    let url = format!("{}/query?function=OVERVIEW&symbol=AAPL&apikey=demo", mock_url);
    let response = client.get(&url).send().await.unwrap();

    // Deserialize the response into DividendHistoryResponse
    let dividend_response: DividendHistoryResponse = response.json().await.unwrap();

    // Assertions to ensure the data was correctly parsed
    assert_eq!(dividend_response.symbol, "AAPL");
    assert_eq!(dividend_response.data.len(), 2);
    assert_eq!(dividend_response.data[0].ex_dividend_date, NaiveDate::parse_from_str("2021-08-06", "%Y-%m-%d").map_err(|_| "Failed to parse date")?);
    assert_eq!(dividend_response.data[0].amount, 0.22);
    assert_eq!(dividend_response.data[1].ex_dividend_date, NaiveDate::parse_from_str("2021-05-07", "%Y-%m-%d").map_err(|_| "Failed to parse date")?);
    assert_eq!(dividend_response.data[1].amount, 0.22);

    Ok(())
}

#[tokio::test]
async fn test_price_history_response() -> Result<(), Box<dyn Error>> {
    // Define Mock Server Response
    let mock_server_response = r#"
    {
        "Meta Data": {
            "1. Information": "Monthly Prices (open, high, low, close) and Volumes",
            "2. Symbol": "IBM",
            "3. Last Refreshed": "2024-09-18",
            "4. Time Zone": "US/Eastern"
        },
        "Monthly Time Series": {
            "2024-09-18": {
                "1. open": "201.9100",
                "2. high": "218.8400",
                "3. low": "199.3350",
                "4. close": "214.9400",
                "5. volume": "48332843"
            },
            "2024-08-31": {
                "1. open": "200.0000",
                "2. high": "204.0000",
                "3. low": "198.0000",
                "4. close": "202.0000",
                "5. volume": "12345678"
            }
        }
    }"#;

    // Set up mock server with mockito
    let _mock = mock("GET", "/query")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("function".into(), "TIME_SERIES_MONTHLY".into()),
            Matcher::UrlEncoded("symbol".into(), "IBM".into()),
            Matcher::UrlEncoded("apikey".into(), "demo".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_server_response)
        .create();

    // Use mockito's server URL
    let mock_url = &mockito::server_url();

    // Make a request using reqwest to the mock server
    let client = Client::new();
    let url = format!("{}/query?function=TIME_SERIES_MONTHLY&symbol=IBM&apikey=demo", mock_url);
    let response = client.get(&url).send().await.unwrap();

    // Deserialize the response into PriceHistoryResponse
    let price_history_response: PriceHistoryResponse = response.json().await.unwrap();

    // Assertions to ensure the data was correctly parsed
    assert_eq!(price_history_response.meta_data.symbol, "IBM");
    assert_eq!(price_history_response.meta_data.information, "Monthly Prices (open, high, low, close) and Volumes");
    assert_eq!(price_history_response.meta_data.last_refreshed, NaiveDate::parse_from_str("2024-09-18", "%Y-%m-%d").map_err(|_| "Failed to parse date")?);
    assert_eq!(price_history_response.meta_data.time_zone, "US/Eastern");
    assert_eq!(price_history_response.monthly_time_series.len(), 2);
    assert_eq!(price_history_response.monthly_time_series["2024-09-18"].open, 201.9100);
    assert_eq!(price_history_response.monthly_time_series["2024-08-31"].volume, 12345678);

    Ok(())
}