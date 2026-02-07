export interface PriceLevel {
  price: number
  size: number
}

export interface RunnerBook {
  selection_id: number
  last_traded_price: number | null
  traded_volume: number | null
  back: PriceLevel[]
  lay: PriceLevel[]
}

export interface OrderBook {
  market_id: string
  status: string | null
  inplay: boolean | null
  total_volume: number
  runners: RunnerBook[]
  last_update: number | null
}

/** Data point for the depth chart. */
export interface DepthPoint {
  price: number
  backDepth: number
  layDepth: number
}
