# standard libraries
from concurrent.futures import ProcessPoolExecutor
from typing import List

# third party libraries
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
from scipy.optimize import minimize
from tqdm import tqdm

# portfoliogenerator libraries
from portfoliogenerator.utils import _tax_non_qualified, _tax_qualified


class Allocator:
    """
    A class to optimize a portfolio using the Sequential Least Squares Programming (SLSQP) algorithm.
    """

    MIN_INVESTMENT: float = 0.01

    def __init__(
        self,
        df: pd.DataFrame,
        initial_capital: float,
        required_income: float,
        special_tickers: List[str],
        etf_list: List[str],
        stock_list: List[str],
        required_div_growth: float = 0.10,
        required_cagr: float = 0.10,
        desired_yield: float = 0.10,
        runs: int = 10,
        etf_lower_bound: float = 0.0,
        etf_upper_bound: float = 0.5,
        special_lower_bound: float = 0.0,
        special_upper_bound: float = 0.025,
        stock_lower_bound: float = 0.0,
        stock_upper_bound: float = 0.05,
        salary: float = 180_000,
    ):
        """Initialize the Allocator with the given parameters."""
        self.df = df
        self.initial_capital = initial_capital
        self.required_income = required_income
        self.special_tickers = special_tickers
        self.etf_list = etf_list
        self.stock_list = stock_list
        self.required_div_growth = required_div_growth
        self.required_cagr = required_cagr
        self.desired_yield = desired_yield
        self.runs = runs
        self.etf_lower_bound = etf_lower_bound
        self.etf_upper_bound = etf_upper_bound
        self.special_lower_bound = special_lower_bound
        self.special_upper_bound = special_upper_bound
        self.stock_lower_bound = stock_lower_bound
        self.stock_upper_bound = stock_upper_bound
        self.salary = salary

    def optimize(
        self,
        weights: List[float],
        div_growth_weight: float,
        cagr_weight: float,
        income_weight: float,
        diversity_weight: float,
    ) -> List[float]:
        """Optimize the portfolio using the SLSQP algorithm."""
        raw_weights = self._optimize(weights, div_growth_weight, cagr_weight, income_weight, diversity_weight)
        return self._adjust_weights(raw_weights)

    def objective_function(self, weights: List[float], alpha: float, beta: float, gamma: float, delta: float) -> float:
        """Objective function to minimize for the SLSQP optimization."""
        assert (
            round(sum([alpha, beta, gamma, delta]), 2) == 1
        ), f"The sum of Alpha ({alpha}), Beta ({beta}), Gamma ({gamma}), and Delta ({delta}) must be 1."
        weighted_yield = np.dot(weights, self.df["Yield"])
        net_income = weighted_yield * self.initial_capital - self._calculate_taxes(weights)
        weighted_div_growth = np.dot(weights, self.df["5 Yr Dividend Growth"])
        weighted_cagr = np.dot(weights, self.df["5 Yr CAGR"])

        # Calculate penalties for not meeting required metrics
        income_penalty = 0 if net_income >= self.required_income else (self.required_income - net_income) * 1e3
        div_growth_penalty = (
            0
            if weighted_div_growth >= self.required_div_growth
            else (self.required_div_growth - weighted_div_growth) * 1e3
        )
        cagr_penalty = 0 if weighted_cagr >= self.required_cagr else (self.required_cagr - weighted_cagr) * 1e3
        expense_penalty = np.dot(weights, self.df["Expense Ratio"]) * 1e3
        yield_penalty = 0 if weighted_yield >= self.desired_yield else (self.desired_yield - weighted_yield) * 1e3

        # Calculate diversity Penalty
        diversity_penalty = delta * self._calculate_diversity_penalty(weights)

        # Modify the impact of the diversity penalty based on the satisfaction of other metrics
        # Reduce diversity penalty influence when other metrics are met
        if (
            weighted_yield >= self.desired_yield
            and weighted_div_growth >= self.required_div_growth
            and weighted_cagr >= self.required_cagr
        ):
            diversity_impact = delta * 0.5  # Reduce impact by 50% when all metrics are met
        else:
            diversity_impact = delta

        return (
            -(alpha * weighted_div_growth + beta * weighted_cagr + gamma * weighted_yield)
            + (diversity_impact * diversity_penalty)
            + income_penalty
            + div_growth_penalty
            + cagr_penalty
            + expense_penalty
            + yield_penalty
        )

    def _calculate_taxes(self, weights: List[float], capital: float = None) -> float:
        if not capital:
            capital = self.initial_capital
        qualified_income = np.dot(weights * self.df["Qualified"], self.df["Yield"]) * capital
        non_qualified_income = np.dot(weights * ~self.df["Qualified"], self.df["Yield"]) * capital

        return _tax_qualified(qualified_income, self.salary) + _tax_non_qualified(non_qualified_income, self.salary)

    def constraint_sum_to_one(self, weights):
        """Constraint function to ensure the sum of the weights equals 1.0."""
        return np.sum(weights) - 1.0

    def run_optimization(self, _):
        """Run the optimization process."""
        result = minimize(
            self.objective_function,
            self.initial_weights,
            args=(self.alpha, self.beta, self.gamma, self.delta),
            method=self.method,
            bounds=self.bounds,
            constraints={"type": "eq", "fun": self.constraint_sum_to_one},
        )
        return result

    def _optimize(self, weights, alpha, beta, gamma, delta, method="SLSQP"):
        self.initial_weights, self.alpha, self.beta, self.gamma, self.delta, self.method = (
            weights,
            alpha,
            beta,
            gamma,
            delta,
            method,
        )
        self.bounds = [
            (
                (self.etf_lower_bound, self.etf_upper_bound)
                if ticker in self.etf_list
                else (
                    (self.special_lower_bound, self.special_upper_bound)
                    if ticker in self.special_tickers
                    else (self.stock_lower_bound, self.stock_upper_bound) if ticker in self.stock_list else (0, 0)
                )
            )
            for ticker in self.df["Ticker"]
        ]

        self.constraints = {"type": "eq", "fun": self.constraint_sum_to_one}

        best_solution = None
        with ProcessPoolExecutor() as executor:
            results = list(
                tqdm(
                    executor.map(self.run_optimization, range(self.runs)),
                    total=self.runs,
                    desc=f"Optimizing with {method}",
                )
            )

        for result in results:
            if best_solution is None or (result.success and result.fun < best_solution.fun):
                best_solution = result

        self.best_weights = best_solution.x if best_solution else None
        return self.best_weights

    def _generate_weights(self):
        num_assets = len(self.df)
        random_allocations = np.random.rand(num_assets)
        normalized_allocations = random_allocations / sum(random_allocations)
        return normalized_allocations

    def _adjust_weights(self, weights):
        # Round weights less than the minimum investment down to 0
        adjusted_weights = np.where(weights >= self.MIN_INVESTMENT, weights, 0)

        # Calculate the shortfall to redistribute
        shortfall = 1 - adjusted_weights.sum()

        # Redistribute the shortfall proportionally to the weights above the minimum investment
        if shortfall > 0:
            above_min_mask = adjusted_weights >= self.MIN_INVESTMENT
            redistribution = (adjusted_weights[above_min_mask] / adjusted_weights[above_min_mask].sum()) * shortfall
            adjusted_weights[above_min_mask] += redistribution

        # Normalize the weights to sum to 1 again
        adjusted_weights /= adjusted_weights.sum()

        return adjusted_weights

    def _calculate_diversity_penalty(self, weights):
        # extract sectors from dataframe
        sectors = self.df.columns[self.df.columns.str.contains("Sector")]

        # Calculate the total capital invested in each sector
        sector_allocations = np.dot(weights, self.df[sectors])

        # Calculate the proportion of total capital in each sector
        sector_proportions = sector_allocations / sector_allocations.sum()

        # Calculate the Herfindahl-Hirschman Index (HHI) for the portfolio
        hhi = np.sum(sector_proportions**2)

        # Normalize the HHI to a range of 0 to 1 (0 being the most diverse and 1 being the least)
        hhi_normalized = (hhi - 1 / len(sectors)) / (1 - 1 / len(sectors))

        return hhi_normalized * 1e3

    def print_distribution(self, weights: List[float]) -> None:
        """Print the sector distribution of the portfolio."""

        sectors = self.df.columns[self.df.columns.str.contains("Sector")]

        # Calculate sector distribution
        self.df[sectors] = self.df[sectors].apply(pd.to_numeric, errors="coerce")
        sector_allocations = np.dot(weights, self.df[sectors].values)

        # Custom color palette for the pie chart
        colors = [
            "#ff9999",
            "#66b3ff",
            "#99ff99",
            "#ffcc99",
            "#c2c2f0",
            "#ffb3e6",
            "#c4e17f",
            "#76d7c4",
            "#f7c6c7",
            "#f7b7a3",
            "#ffcc00",
            "#c4e17f",
        ]

        # Calculate and display the pie chart for sector distribution
        fig, ax = plt.subplots()
        ax.pie(
            sector_allocations,
            labels=[sector.lstrip("Sector_") for sector in sectors],
            autopct="%1.1f%%",
            startangle=90,
            colors=colors,
        )
        ax.axis("equal")  # Equal aspect ratio ensures that pie is drawn as a circle.
        plt.title("Portfolio Sector Distribution")
        plt.show()

    def print_allocations(self, weights: List[float]) -> None:
        """Print the allocations of the portfolio in a markdown table."""
        tickers = self.df["Ticker"]
        initial_capital = self.initial_capital
        yields = self.df["Yield"].values

        # Precompute content lengths to determine column widths
        max_ticker_length = max(len(ticker) for ticker in tickers)
        max_weight_length = max(len(f"{weight:.3%}") for weight in weights)
        max_amount_length = max(len(f"${weight * initial_capital:,.2f}") for weight in weights)
        max_income_length = max(
            len(f"${weight * initial_capital * yield_:,.2f}") for weight, yield_ in zip(weights, yields)
        )

        max_length = max(max_ticker_length, max_weight_length, max_amount_length, max_income_length)

        # Headers with adjusted column widths
        headers = [
            "Ticker".ljust(max_length),
            "Weight".ljust(max_length),
            "Amount".ljust(max_length),
            "Income".ljust(max_length),
        ]
        table = f"| {' | '.join(headers)} |\n"

        # Separator line
        separator = ["-" * max_length, "-" * max_length, "-" * max_length, "-" * max_length]
        table += f"| {' | '.join(separator)} |\n"

        # Table rows
        for ticker, weight, yield_value in zip(tickers, weights, yields):
            amount = weight * initial_capital
            income = amount * yield_value
            row = [
                ticker.ljust(max_length),
                f"{weight:.3%}".ljust(max_length),
                f"${amount:,.2f}".ljust(max_length),
                f"${income:,.2f}".ljust(max_length),
            ]
            table += f"| {' | '.join(row)} |\n"

        # Print the markdown table
        print(table)

    def print_summary(self, weights: List[float]):
        """Print a summary of the portfolio."""
        initial_capital = self.initial_capital
        yields = self.df["Yield"].values

        # Calculate the portfolio"s overall metrics
        portfolio_yield = np.dot(weights, yields)
        portfolio_weighted_div_growth = np.dot(weights, self.df["5 Yr Dividend Growth"].values)
        portfolio_weighted_cagr = np.dot(weights, self.df["5 Yr CAGR"].values)
        portfolio_expense_ratio = np.dot(weights, self.df["Expense Ratio"].values)
        portfolio_weighted_beta = np.dot(weights, self.df["Beta"].values)

        # Display the results
        print("\nSummary:")
        print(f"Total Holdings: {len([weight for weight in weights if weight > 0])}")
        print(f"Portfolio Yield: {portfolio_yield:.2%}")
        print(f"Weighted 5-Year Dividend Growth: {portfolio_weighted_div_growth:.2%}")
        print(f"Weighted 5-Year CAGR: {portfolio_weighted_cagr:0.2%}")
        print(f"Portfolio Gross Income: ${portfolio_yield * initial_capital:,.2f}")
        print(f"Portfolio Tax Burden: ${self._calculate_taxes(weights):,.2f}")
        print(
            f"Portfolio Effective Tax Rate: {self._calculate_taxes(weights) / (portfolio_yield * initial_capital):0.2%}"
        )
        print(f"Portfolio Net Income: ${portfolio_yield * initial_capital - self._calculate_taxes(weights):,.2f}")
        print(f"Expense Ratio: {portfolio_expense_ratio:0.2%}")
        print(f"Weighted Beta: {portfolio_weighted_beta:0.2f}")

    def _generate_financial_projection(
        self,
        weights: List[float],
        start_year: int = 2026,
        initial_cost_of_living: float = 78000.0,
        col_growth_rate: float = 0.033,
        initial_age: int = 33,
        num_years: int = 30,
    ):
        initial_capital = self.initial_capital
        yields = self.df["Yield"].values

        # Calculate the portfolio"s overall metrics
        portfolio_yield = np.dot(weights, yields)
        portfolio_weighted_div_growth = np.dot(weights, self.df["5 Yr Dividend Growth"].values)
        portfolio_weighted_cagr = np.dot(weights, self.df["5 Yr CAGR"].values)
        initial_gross_income = portfolio_yield * initial_capital
        initial_capital = self.initial_capital

        years = list(range(start_year, start_year + num_years))
        ages = list(range(initial_age, initial_age + num_years))
        incomes = []
        costs_of_living = []
        capital = []

        current_income = initial_gross_income
        current_cost_of_living = initial_cost_of_living
        current_capital = initial_capital

        for _ in range(num_years):
            incomes.append(current_income)
            costs_of_living.append(current_cost_of_living)
            capital.append(current_capital)
            current_income *= 1 + portfolio_weighted_div_growth
            current_cost_of_living *= 1 + col_growth_rate
            current_capital *= 1 + portfolio_weighted_cagr

        data = {
            "Year": years,
            "Age": ages,
            "Capital": capital,
            "Income": incomes,
            "Cost of Living": costs_of_living,
            "Difference": [
                income - self._calculate_taxes(weights, cap) - col
                for income, col, cap in zip(incomes, costs_of_living, capital)
            ],
        }

        df = pd.DataFrame(data)
        return df

    def print_projection(self, weights: List[float]) -> None:
        """Print a financial projection of the portfolio."""
        df = self._generate_financial_projection(weights)

        # Precompute content lengths to determine column widths
        max_year_length = max(len(str(year)) for year in df["Year"])
        max_age_length = max(len(str(age)) for age in df["Age"])
        max_income_length = max(len(f"${income:,.2f}") for income in df["Income"])
        max_capital = max(len(f"${capital:,.2f}") for capital in df["Capital"])
        max_col_length = max(len(f"${col:,.2f}") for col in df["Cost of Living"])
        max_diff_length = max(len(f"${diff:,.2f}") for diff in df["Difference"])

        max_length = max(
            max_year_length, max_age_length, max_income_length, max_col_length, max_diff_length, max_capital
        )

        # Headers with adjusted column widths
        headers = [
            "Year".ljust(max_length),
            "Age".ljust(max_length),
            "Capital".ljust(max_length),
            "Gross Income".ljust(max_length),
            "Net Income".ljust(max_length),
            "Cost of Living".ljust(max_length),
            "Difference".ljust(max_length),
        ]
        table = f"| {' | '.join(headers)} |\n"

        # Separator line
        separator = ["-" * max_length for _ in headers]
        table += f"| {' | '.join(separator)} |\n"

        # Table rows
        for _, row in df.iterrows():
            year = (f"{row['Year']:.0f}").ljust(max_length)
            age = (f"{row['Age']:.0f}").ljust(max_length)
            capital = f"${row['Capital']:,.2f}".ljust(max_length)
            gross_income = f"${row['Income']:,.2f}".ljust(max_length)
            income = f"${row['Income'] - self._calculate_taxes(weights, row['Capital']):,.2f}".ljust(max_length)
            cost_of_living = f"${row['Cost of Living']:,.2f}".ljust(max_length)
            difference = f"${row['Difference']:,.2f}".ljust(max_length)

            # Format each row as a markdown table row
            table += f"| {year} | {age} | {capital} | {gross_income} | {income} | {cost_of_living} | {difference} |\n"

        # Print the markdown table
        print(table)
