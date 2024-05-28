# third party libraries
import pandas as pd

# portfoliogenerator libraries
from portfoliogenerator.models.pso import pso_optimize


def main() -> None:
    """
    Script to profile the PSO optimization function.
    """
    df = pd.read_csv("~/repos/portfolio-generator/tests/data/portfolio.csv")
    df["ETF"] = df["ETF"].map({"TRUE": 1.0, "FALSE": 0.0, "1": 1.0, "0": 0.0})
    df_dict = df.to_dict(orient="list")
    configs = [
        {"stock": {"min": 0.00, "max": 0.05}},
        {"etf": {"min": 0.00, "max": 0.35}},
    ]

    pso_optimize(
        num_particles=50,
        asset_configs=configs,
        inertia=0.2,
        cognitive=0.2,
        social=0.2,
        num_iterations=50,
        df_dict=df_dict,
        salary=180_000,
        min_div_growth=0.1,
        min_cagr=0.1,
        min_yield=0.1,
        required_income=78_000,
        initial_capital=1.5e6,
        div_preference=0.5,
        cagr_preference=0.3,
        yield_preference=0.2,
        filing_status="Married Filling Jointly",
    )


if __name__ == "__main__":
    main()
