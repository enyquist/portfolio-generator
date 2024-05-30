# third party libraries
import numpy as np
import pandas as pd

SALARY_INCOME = 180_000  # TODO: Update with actual salary income


def calculate_diversity_penalty(weights: np.array, df: pd.DataFrame) -> float:
    """
    Calculate the diversity penalty for a given portfolio.

    Args:
        weights (np.array): Portfolio weights
        df (pd.DataFrame): DataFrame containing the sector allocations for each stock

    Returns:
        float: Herfindahl-Hirschman Index (HHI) normalized to a range of 0 to 1
    """
    # extract sectors from dataframe
    sectors = df.columns[df.columns.str.contains("Sector")]

    # Calculate the total capital invested in each sector
    sector_allocations = np.dot(weights, df[sectors].values)

    # Calculate the proportion of total capital in each sector
    sector_proportions = sector_allocations / sector_allocations.sum()

    # Calculate the Herfindahl-Hirschman Index (HHI) for the portfolio
    hhi = np.sum(sector_proportions**2)

    # Normalize the HHI to a range of 0 to 1 (0 being the most diverse and 1 being the least)
    hhi_normalized = (hhi - 1 / len(sectors)) / (1 - 1 / len(sectors))

    return hhi_normalized * 1e3


def calculate_taxes(weights: np.array, capital: float, df: pd.DataFrame) -> float:
    """
    Calculate the total tax liability for a given portfolio.

    Args:
        weights (np.array): Portfolio weights
        capital (float): Initial capital invested in the portfolio
        df (pd.DataFrame): DataFrame containing the qualified status and yield of each stock

    Returns:
        float: Total tax liability for the portfolio
    """
    qualified_income = np.dot(weights * df["Qualified"], df["Yield"]) * capital
    non_qualified_income = np.dot(weights * ~df["Qualified"], df["Yield"]) * capital

    return _tax_qualified(qualified_income) + _tax_non_qualified(non_qualified_income)


def _tax_qualified(income: float, salary: float) -> float:
    brackets = [
        (94_050, 0.0),  # 0% for income up to $94,050
        (583_750, 0.15),  # 15% for income up to $583,750
        (float("inf"), 0.20),  # 20% for income over $583,751
    ]

    total_income = income + salary

    for limit, rate in brackets:
        if total_income <= limit:
            tax_rate = rate
            break

    # Calculate the tax based on the determined rate
    return income * tax_rate


def _tax_non_qualified(income: float, salary: float) -> float:
    # Define the tax brackets and rates for Married Filing Jointly
    brackets = [
        (22000, 0.10),  # 10% for income up to $22,000
        (89450, 0.12),  # 12% for income over $22,000 to $89,450
        (190750, 0.22),  # 22% for income over $89,450 to $190,750
        (364200, 0.24),  # 24% for income over $190,750 to $364,200
        (462500, 0.32),  # 32% for income over $364,200 to $462,500
        (693750, 0.35),  # 35% for income over $462,500 to $693,750
        (None, 0.37),  # 37% for income over $693,750
    ]

    def calculate_tax(total_income: float) -> float:
        tax_owed = 0
        previous_limit = 0

        # Calculate the tax owed
        for limit, rate in brackets:
            if total_income > previous_limit:
                # Apply tax rate to the amount of income within the current bracket
                taxable_income = min(total_income, limit if limit is not None else total_income) - previous_limit
                tax_owed += taxable_income * rate
                previous_limit = limit if limit is not None else total_income
            else:
                break

        return tax_owed

    # Calculate total income (W2 income + unqualified dividends)
    total_income = salary + income

    # Calculate the total tax owed on the combined income
    total_tax_owed = calculate_tax(total_income)

    # Calculate the tax owed on the W2 income alone
    salary_tax_owed = calculate_tax(salary)

    # The tax attributable to the unqualified dividends is the difference
    tax_on_dividends = total_tax_owed - salary_tax_owed

    return tax_on_dividends
