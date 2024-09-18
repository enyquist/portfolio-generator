wrk.method = "POST"
wrk.headers["Content-Type"] = "application/json"

-- Function to generate a request payload with a given dimension size
function generate_payload(dimension)
    local lower_bounds = {}
    local upper_bounds = {}
    local div_growth_rates = {}
    local cagr_rates = {}
    local yields = {}
    local expense_ratios = {}
    local sector = {}
    local qualified = {}

    for i = 1, dimension do
        table.insert(lower_bounds, string.format("%.1f", 0.0))  -- Ensure 0.0 is formatted as a float
        table.insert(upper_bounds, string.format("%.1f", 1.0))  -- Ensure 1.0 is formatted as a float
        table.insert(div_growth_rates, string.format("%.4f", 0.04 + math.random() * 0.04))
        table.insert(cagr_rates, string.format("%.4f", 0.06 + math.random() * 0.04))
        table.insert(yields, string.format("%.4f", 0.02 + math.random() * 0.04))
        table.insert(expense_ratios, string.format("%.4f", 0.001 + math.random() * 0.003))
        
        -- Explicitly format sector and qualified as floats
        table.insert(sector, string.format("%.1f", math.random(1, 3) * 1.0))  -- Float (1.0, 2.0, 3.0)
        table.insert(qualified, string.format("%.1f", math.random(0, 1) * 1.0))  -- Float (0.0, 1.0)
    end

    return string.format([[
    {
        "dimension": %d,
        "lower_bounds": [%s],
        "upper_bounds": [%s],
        "initial_capital": 100000.0,
        "salary": 50000.0,
        "required_income": 20000.0,
        "min_div_growth": 0.05,
        "min_cagr": 0.07,
        "min_yield": 0.03,
        "div_preference": 0.5,
        "cagr_preference": 0.3,
        "yield_preference": 0.2,
        "filing_status": "single",
        "redistribution_threshold": 0.01,
        "columns": {
            "dividend_growth_rates": [%s],
            "cagr_rates": [%s],
            "yields": [%s],
            "expense_ratios": [%s],
            "sector": [%s],
            "qualified": [%s]
        }
    }
    ]], 
    dimension,
    table.concat(lower_bounds, ", "),
    table.concat(upper_bounds, ", "),
    table.concat(div_growth_rates, ", "),
    table.concat(cagr_rates, ", "),
    table.concat(yields, ", "),
    table.concat(expense_ratios, ", "),
    table.concat(sector, ", "),
    table.concat(qualified, ", "))
end

-- Request distribution
function request_distribution()
    local rand = math.random()

    if rand <= 0.2 then
        return 10  -- 20% of requests will have a dimension of 10
    elseif rand <= 0.9 then
        return 25  -- 70% of requests will have a dimension of 25
    else
        return 50  -- 10% of requests will have a dimension of 50
    end
end

-- Function to assign a payload based on the distribution
request = function()
    local dimension = request_distribution()
    wrk.body = generate_payload(dimension)
    return wrk.format("POST", nil, wrk.headers, wrk.body)
end
