import type { RunnerBook } from './types'

interface Props {
  runner: RunnerBook
}

/**
 * Price ladder showing individual back/lay levels in a table format.
 */
export function RunnerLadder({ runner }: Props) {
  return (
    <div className="ladder">
      <div className="ladder-side">
        <div className="ladder-title back">Back (buy)</div>
        {runner.back.length === 0 ? (
          <div style={{ color: 'var(--text-dim)', fontSize: 13, padding: '4px 8px' }}>—</div>
        ) : (
          runner.back.map((level, i) => (
            <div key={i} className="ladder-row back">
              <span className="price">{level.price.toFixed(2)}</span>
              <span className="size">£{level.size.toFixed(2)}</span>
            </div>
          ))
        )}
      </div>

      <div className="ladder-side">
        <div className="ladder-title lay">Lay (sell)</div>
        {runner.lay.length === 0 ? (
          <div style={{ color: 'var(--text-dim)', fontSize: 13, padding: '4px 8px' }}>—</div>
        ) : (
          runner.lay.map((level, i) => (
            <div key={i} className="ladder-row lay">
              <span className="price">{level.price.toFixed(2)}</span>
              <span className="size">£{level.size.toFixed(2)}</span>
            </div>
          ))
        )}
      </div>
    </div>
  )
}
