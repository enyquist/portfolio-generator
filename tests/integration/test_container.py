# third party libraries
import requests


def test_health_check():
    response = requests.get("http://localhost:8080/health")
    assert response.status_code == 200
    assert response.text == "OK"


def test_optimize():
    payload = {
        "dimension": 3,
        "lower_bounds": [0.0, 0.0, 0.0],
        "upper_bounds": [1.0, 1.0, 1.0],
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
        "redistribution_threshold": 0.1,
        "columns": {
            "dividend_growth_rates": [0.04, 0.05, 0.06],
            "cagr_rates": [0.06, 0.07, 0.08],
            "yields": [0.02, 0.03, 0.04],
            "expense_ratios": [0.001, 0.002, 0.003],
            "sector": [1.0, 2.0, 1.0],
            "qualified": [1.0, 0.0, 1.0],
        },
    }

    response = requests.post("http://localhost:8080/optimize", json=payload)
    assert response.status_code == 200
    assert response.json().get("success") is True
    assert response.json().get("x") is not None
