# standard libraries
from typing import Dict, List

# third party libraries
import numpy as np
import pandas as pd
from rspso import optimize as rs_optimize

# portfoliogenerator libraries
from portfoliogenerator.models.slsqp import Allocator

AssetConfig = Dict[str, Dict[str, float]]
PortfolioData = Dict[str, List[float | str]]


def pso_optimize(
    df: pd.DataFrame,
    asset_configs: List[AssetConfig],
    salary: float,
    num_particles: int = 50,
    inertia: float = 0.5,
    decay_rate: float = 0.1,
    cognitive: float = 0.5,
    social: float = 0.5,
    num_iterations: int = 100,
    min_div_growth: float = 0.10,
    min_cagr: float = 0.10,
    min_yield: float = 0.10,
    required_income: float = 78_000,
    initial_capital: float = 1_500_000,
    div_preference: float = 0.50,
    cagr_preference: float = 0.30,
    yield_preference: float = 0.20,
    filing_status: str = "Single",
) -> np.ndarray:
    """
    Optimize the portfolio using the Particle Swarm Optimization algorithm.

    Args:
        df_dict (PortfolioData): _description_
        asset_configs (List[AssetConfig]): _description_
        salary (float): _description_
        num_particles (int, optional): _description_. Defaults to 50.
        inertia (float, optional): _description_. Defaults to 0.5.
        cognitive (float, optional): _description_. Defaults to 0.5.
        social (float, optional): _description_. Defaults to 0.5.
        num_iterations (int, optional): _description_. Defaults to 100.
        min_div_growth (float, optional): _description_. Defaults to 0.10.
        min_cagr (float, optional): _description_. Defaults to 0.10.
        min_yield (float, optional): _description_. Defaults to 0.10.
        required_income (float, optional): _description_. Defaults to 78_000.
        initial_capital (float, optional): _description_. Defaults to 1_500_000.
        div_preference (float, optional): _description_. Defaults to 0.50.
        cagr_preference (float, optional): _description_. Defaults to 0.30.
        yield_preference (float, optional): _description_. Defaults to 0.20.
        filing_status (str, optional): _description_. Defaults to "Single".

    Returns:
        np.ndarray: _description_
    """
    # Prepare Data
    df["ETF"] = df["ETF"].map({"TRUE": 1.0, "FALSE": 0.0, "1": 1.0, "0": 0.0})
    df_dict = df.to_dict(orient="list")

    # Prepare Parameters
    num_assets = len(df_dict.get("Ticker"))
    numeric_data = {k: v for k, v in df_dict.items() if k != "Ticker"}

    weights, iteration_broke, best_score = rs_optimize(
        num_particles,
        asset_configs,
        num_assets,
        inertia,
        decay_rate,
        cognitive,
        social,
        num_iterations,
        numeric_data,
        salary,
        min_div_growth,
        min_cagr,
        min_yield,
        required_income,
        initial_capital,
        div_preference,
        cagr_preference,
        yield_preference,
        filing_status,
    )

    # SLSQP Optimization
    # Define constants
    ETFS = list(df[df["ETF"] == 1.0]["Ticker"])
    ETFS.remove("BST")
    STOCKS = list(df[df["ETF"] == 0.0]["Ticker"])
    # STOCKS.remove("CONY")
    STOCKS.append("BST")
    SPECIAL_TICKERS = ["NVDY"]
    df["Qualified"] = df["Qualified"].map({1.0: True, 0.0: False})

    etf_upper_bound = asset_configs[1].get("max")
    stock_upper_bound = asset_configs[0].get("max")

    brokerage_allocator = Allocator(
        df=df,
        initial_capital=initial_capital,
        required_income=required_income,
        special_tickers=SPECIAL_TICKERS,
        etf_list=ETFS,
        stock_list=STOCKS,
        special_upper_bound=stock_upper_bound,
        etf_upper_bound=etf_upper_bound,
        runs=3,
        salary=salary,
    )

    # Optimize
    weights = brokerage_allocator.optimize(
        weights,
        div_growth_weight=div_preference,
        cagr_weight=cagr_preference,
        income_weight=yield_preference,
        diversity_weight=0.0,
    )

    return weights, iteration_broke, best_score
