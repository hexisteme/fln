"""Market-data sources for the oracle."""

from __future__ import annotations

from abc import ABC, abstractmethod
from datetime import UTC, datetime, timedelta

import pandas as pd


class MarketSource(ABC):
    @abstractmethod
    def history(self, ticker: str, days: int) -> pd.DataFrame:
        """Return a DataFrame indexed by datetime (UTC) with at least Open/High/Low/Close columns."""


class InMemorySource(MarketSource):
    """Deterministic source for unit tests."""

    def __init__(self, store: dict[str, pd.DataFrame]):
        self._store = store

    def history(self, ticker: str, days: int) -> pd.DataFrame:
        if ticker not in self._store:
            raise KeyError(f"no in-memory series for {ticker}")
        df = self._store[ticker]
        cutoff = df.index.max() - pd.Timedelta(days=days)
        return df[df.index >= cutoff].copy()


class YFinanceSource(MarketSource):
    """Live yfinance adapter — no API key required."""

    def history(self, ticker: str, days: int) -> pd.DataFrame:
        import yfinance as yf

        end = datetime.now(UTC).date()
        start = end - timedelta(days=days + 7)  # small buffer for non-trading days
        df = yf.download(
            ticker,
            start=start.isoformat(),
            end=(end + timedelta(days=1)).isoformat(),
            progress=False,
            auto_adjust=False,
        )
        if df.empty:
            raise RuntimeError(f"yfinance returned no data for {ticker}")
        if isinstance(df.columns, pd.MultiIndex):
            df.columns = df.columns.get_level_values(0)
        df.index = pd.to_datetime(df.index, utc=True)
        return df.tail(days + 1)
