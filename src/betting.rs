use crate::client::BetfairClient;
use crate::error::Result;
use crate::types::betting::*;
use crate::types::common::*;

/// Betting API operations.
impl BetfairClient {
    // ─── Navigation / Discovery ─────────────────────────

    /// List all event types (sports).
    pub async fn list_event_types(&self, filter: &MarketFilter) -> Result<Vec<EventTypeResult>> {
        self.betting_request("listEventTypes", &FilterRequest { filter: filter.clone(), locale: None })
            .await
    }

    /// List competitions matching the filter.
    pub async fn list_competitions(&self, filter: &MarketFilter) -> Result<Vec<CompetitionResult>> {
        self.betting_request("listCompetitions", &FilterRequest { filter: filter.clone(), locale: None })
            .await
    }

    /// List events matching the filter.
    pub async fn list_events(&self, filter: &MarketFilter) -> Result<Vec<EventResult>> {
        self.betting_request("listEvents", &FilterRequest { filter: filter.clone(), locale: None })
            .await
    }

    /// List market types matching the filter.
    pub async fn list_market_types(&self, filter: &MarketFilter) -> Result<Vec<MarketTypeResult>> {
        self.betting_request("listMarketTypes", &FilterRequest { filter: filter.clone(), locale: None })
            .await
    }

    /// List countries matching the filter.
    pub async fn list_countries(&self, filter: &MarketFilter) -> Result<Vec<CountryCodeResult>> {
        self.betting_request("listCountries", &FilterRequest { filter: filter.clone(), locale: None })
            .await
    }

    /// List venues matching the filter.
    pub async fn list_venues(&self, filter: &MarketFilter) -> Result<Vec<VenueResult>> {
        self.betting_request("listVenues", &FilterRequest { filter: filter.clone(), locale: None })
            .await
    }

    // ─── Market Data ────────────────────────────────────

    /// List market catalogues (metadata, runners).
    pub async fn list_market_catalogue(
        &self,
        request: &ListMarketCatalogueRequest,
    ) -> Result<Vec<MarketCatalogue>> {
        self.betting_request("listMarketCatalogue", request).await
    }

    /// List market books (live prices).
    pub async fn list_market_book(
        &self,
        request: &ListMarketBookRequest,
    ) -> Result<Vec<MarketBook>> {
        self.betting_request("listMarketBook", request).await
    }

    // ─── Order Management ───────────────────────────────

    /// Place orders on a market.
    pub async fn place_orders(
        &self,
        request: &PlaceOrdersRequest,
    ) -> Result<PlaceExecutionReport> {
        self.betting_request("placeOrders", request).await
    }

    /// Cancel orders.
    pub async fn cancel_orders(
        &self,
        request: &CancelOrdersRequest,
    ) -> Result<CancelExecutionReport> {
        self.betting_request("cancelOrders", request).await
    }

    /// Replace orders (change price).
    pub async fn replace_orders(
        &self,
        request: &ReplaceOrdersRequest,
    ) -> Result<ReplaceExecutionReport> {
        self.betting_request("replaceOrders", request).await
    }

    /// Update orders (change persistence).
    pub async fn update_orders(
        &self,
        request: &UpdateOrdersRequest,
    ) -> Result<UpdateExecutionReport> {
        self.betting_request("updateOrders", request).await
    }

    /// List current (unmatched/partially matched) orders.
    pub async fn list_current_orders(
        &self,
        request: &ListCurrentOrdersRequest,
    ) -> Result<CurrentOrderSummaryReport> {
        self.betting_request("listCurrentOrders", request).await
    }
}
