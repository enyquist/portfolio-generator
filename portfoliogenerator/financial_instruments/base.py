# standard libraries
from typing import Optional

# third party libraries
from pydantic import BaseModel, Field


class Sectors(BaseModel):
    """The sectors allocation of the financial instrument"""

    financials: Optional[float] = Field(None, description="Percentage in Financials sector")
    healthcare: Optional[float] = Field(None, description="Percentage in Healthcare sector")
    consumer_defensive: Optional[float] = Field(None, description="Percentage in Consumer Defensive sector")
    industrials: Optional[float] = Field(None, description="Percentage in Industrials sector")
    energy: Optional[float] = Field(None, description="Percentage in Energy sector")
    consumer_cyclical: Optional[float] = Field(None, description="Percentage in Consumer Cyclical sector")
    technology: Optional[float] = Field(None, description="Percentage in Technology sector")
    communication: Optional[float] = Field(None, description="Percentage in Communication sector")
    basic_material: Optional[float] = Field(None, description="Percentage in Basic Material sector")
    utilities: Optional[float] = Field(None, description="Percentage in Utilities sector")
    real_estate: Optional[float] = Field(None, description="Percentage in Real Estate sector")
    non_traditional_equity: Optional[float] = Field(None, description="Percentage in Non-Traditional Equity sector")


class FinancialInstrumentBase(BaseModel):
    """Base class for financial instruments"""

    ticker: str = Field(..., description="The ticker symbol of the financial instrument")
    yield_: Optional[float] = Field(None, alias="yield", description="The dividend yield of the financial instrument")
    dividend_growth_5yr: Optional[float] = Field(None, description="5 Year Dividend Growth")
    etf: Optional[bool] = Field(None, description="Whether the instrument is an ETF")
    beta: Optional[float] = Field(None, description="The beta value of the financial instrument")
    qualified: Optional[bool] = Field(None, description="Whether the dividend is qualified")
    capital_appreciation_5yr: Optional[float] = Field(None, description="5 Year Capital Appreciation")
    cagr_5yr: Optional[float] = Field(None, description="5 Year Compound Annual Growth Rate")
    expense_ratio: Optional[float] = Field(None, description="Expense Ratio for ETFs")
    sectors: Optional[Sectors] = Field(None, description="Sectors allocation")

    class Config:
        """Allow population by field name for the base class"""

        allow_population_by_field_name = True


class Stock(FinancialInstrumentBase):
    """Base class for stocks"""

    etf: bool = Field(False, description="A stock is not an ETF by default")


class ETF(FinancialInstrumentBase):
    """Base class for ETFs"""

    etf: bool = Field(True, description="An ETF is marked as True")
    expense_ratio: float = Field(..., description="Expense Ratio is required for ETFs")
