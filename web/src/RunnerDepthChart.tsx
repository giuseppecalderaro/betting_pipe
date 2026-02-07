import { useMemo } from 'react'
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  ReferenceLine,
} from 'recharts'
import type { RunnerBook, DepthPoint } from './types'

interface Props {
  runner: RunnerBook
}

/**
 * Depth chart showing cumulative back and lay liquidity.
 *
 * - Back side (green): cumulative size available to back, from highest price down
 * - Lay side (red): cumulative size available to lay, from lowest price up
 *
 * This creates the classic "depth chart" view used in exchanges.
 */
export function RunnerDepthChart({ runner }: Props) {
  const { data, midPrice } = useMemo(() => {
    // Build cumulative depth from back prices (sorted descending)
    // Back: buyers willing to back at price or higher
    const backs = [...runner.back].sort((a, b) => b.price - a.price)
    const backCumulative: { price: number; depth: number }[] = []
    let cumBack = 0
    for (const level of backs) {
      cumBack += level.size
      backCumulative.push({ price: level.price, depth: cumBack })
    }
    // Reverse so it goes low→high for the chart
    backCumulative.reverse()

    // Lay: sellers willing to lay at price or lower
    const lays = [...runner.lay].sort((a, b) => a.price - b.price)
    const layCumulative: { price: number; depth: number }[] = []
    let cumLay = 0
    for (const level of lays) {
      cumLay += level.size
      layCumulative.push({ price: level.price, depth: cumLay })
    }

    // Merge into a single array with all price points
    const priceSet = new Set<number>()
    backCumulative.forEach((p) => priceSet.add(p.price))
    layCumulative.forEach((p) => priceSet.add(p.price))

    const allPrices = [...priceSet].sort((a, b) => a - b)

    const data: DepthPoint[] = allPrices.map((price) => {
      // Find back depth at this price (last entry ≤ price)
      const backEntry = backCumulative.find((b) => b.price === price)
      // Find lay depth at this price
      const layEntry = layCumulative.find((l) => l.price === price)

      return {
        price,
        backDepth: backEntry?.depth ?? 0,
        layDepth: layEntry?.depth ?? 0,
      }
    })

    // Separate back and lay data points to avoid overlap
    // Back: only show depth on back side (prices ≤ best back)
    // Lay: only show depth on lay side (prices ≥ best lay)
    const bestBack = backs.length > 0 ? backs[0].price : null
    const bestLay = lays.length > 0 ? lays[0].price : null

    const merged: DepthPoint[] = allPrices.map((price) => {
      const backEntry = backCumulative.find((b) => b.price === price)
      const layEntry = layCumulative.find((l) => l.price === price)

      return {
        price,
        backDepth: bestBack != null && price <= bestBack ? (backEntry?.depth ?? 0) : 0,
        layDepth: bestLay != null && price >= bestLay ? (layEntry?.depth ?? 0) : 0,
      }
    })

    const midPrice = runner.last_traded_price ??
      (bestBack != null && bestLay != null ? (bestBack + bestLay) / 2 : null)

    return { data: merged, midPrice }
  }, [runner])

  if (data.length === 0) {
    return (
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%', color: 'var(--text-dim)' }}>
        Waiting for price data...
      </div>
    )
  }

  return (
    <>
      <ResponsiveContainer width="100%" height="100%">
        <AreaChart data={data} margin={{ top: 10, right: 20, left: 10, bottom: 5 }}>
          <CartesianGrid strokeDasharray="3 3" stroke="var(--border)" />
          <XAxis
            dataKey="price"
            type="number"
            domain={['dataMin', 'dataMax']}
            tick={{ fill: 'var(--text-dim)', fontSize: 11 }}
            tickFormatter={(v: number) => v.toFixed(2)}
            stroke="var(--border)"
          />
          <YAxis
            tick={{ fill: 'var(--text-dim)', fontSize: 11 }}
            tickFormatter={(v: number) => `£${v.toFixed(0)}`}
            stroke="var(--border)"
            width={60}
          />
          <Tooltip
            contentStyle={{
              background: 'var(--surface)',
              border: '1px solid var(--border)',
              borderRadius: 6,
              fontSize: 13,
              color: 'var(--text)',
            }}
            formatter={(value: number, name: string) => [
              `£${value.toFixed(2)}`,
              name === 'backDepth' ? 'Back Depth' : 'Lay Depth',
            ]}
            labelFormatter={(v: number) => `Price: ${v.toFixed(2)}`}
          />
          <Area
            type="stepAfter"
            dataKey="backDepth"
            stroke="#2ea043"
            fill="rgba(46, 160, 67, 0.3)"
            strokeWidth={2}
            isAnimationActive={false}
          />
          <Area
            type="stepBefore"
            dataKey="layDepth"
            stroke="#f85149"
            fill="rgba(248, 81, 73, 0.3)"
            strokeWidth={2}
            isAnimationActive={false}
          />
          {midPrice != null && (
            <ReferenceLine
              x={midPrice}
              stroke="var(--accent)"
              strokeDasharray="4 4"
              strokeWidth={1}
            />
          )}
        </AreaChart>
      </ResponsiveContainer>
      <div className="legend">
        <div className="legend-item">
          <div className="legend-swatch back" />
          <span>Back (cumulative)</span>
        </div>
        <div className="legend-item">
          <div className="legend-swatch lay" />
          <span>Lay (cumulative)</span>
        </div>
      </div>
    </>
  )
}
