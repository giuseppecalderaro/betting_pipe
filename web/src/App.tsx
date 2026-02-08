import { useState } from 'react'
import { useOrderBookSocket } from './useOrderBookSocket'
import { RunnerDepthChart } from './RunnerDepthChart'
import { RunnerLadder } from './RunnerLadder'
import { MatchesView } from './MatchesView'

type View = 'matches' | 'orderbook'

export default function App() {
  const [view, setView] = useState<View>('matches')
  const [wsUrl, setWsUrl] = useState<string | null>(null)
  const [inputUrl, setInputUrl] = useState('ws://127.0.0.1:9001')

  const { orderBook, status, error, updateCount } = useOrderBookSocket(wsUrl)

  // Navigation tabs
  const navBar = (
    <div className="nav-bar">
      <button
        className={`nav-tab ${view === 'matches' ? 'active' : ''}`}
        onClick={() => setView('matches')}
      >
        Matches
      </button>
      <button
        className={`nav-tab ${view === 'orderbook' ? 'active' : ''}`}
        onClick={() => setView('orderbook')}
      >
        Order Book
      </button>
    </div>
  )

  if (view === 'matches') {
    return (
      <>
        {navBar}
        <MatchesView />
      </>
    )
  }

  if (!wsUrl) {
    return (
      <>
        {navBar}
        <div className="app">
          <div className="connect-form">
            <h2>bpipe — Live Order Book</h2>
            <p>
              Connect to a running <code>bpipe-ws</code> server to view real-time
              Betfair market data.
            </p>
            <input
              type="text"
              value={inputUrl}
              onChange={(e) => setInputUrl(e.target.value)}
              placeholder="ws://127.0.0.1:9001"
              onKeyDown={(e) => e.key === 'Enter' && setWsUrl(inputUrl)}
            />
            <button onClick={() => setWsUrl(inputUrl)}>Connect</button>
          </div>
        </div>
      </>
    )
  }

  return (
    <>
      {navBar}
      <div className="app">
        <div className="header">
          <h1>
            bpipe — {orderBook?.market_id ?? 'Connecting...'}
          </h1>
        <div className="status">
          <div
            className={`status-dot ${
              status === 'connected' ? 'connected' : status === 'error' ? 'error' : ''
            }`}
          />
          {status === 'connected'
            ? `Live — ${updateCount} updates`
            : status === 'connecting'
            ? 'Connecting...'
            : status === 'error'
            ? error ?? 'Error'
            : 'Disconnected'}
        </div>
      </div>

      {orderBook && (
        <>
          <div className="market-info">
            <span>
              Status: <span className="value">{orderBook.status ?? '—'}</span>
            </span>
            <span>
              In-play: <span className="value">{orderBook.inplay ? 'Yes' : 'No'}</span>
            </span>
            <span>
              Total volume:{' '}
              <span className="value">
                £{orderBook.total_volume.toLocaleString(undefined, { minimumFractionDigits: 0 })}
              </span>
            </span>
            {orderBook.last_update && (
              <span>
                Last update:{' '}
                <span className="value">
                  {new Date(orderBook.last_update).toLocaleTimeString()}
                </span>
              </span>
            )}
          </div>

          <div className="runner-grid">
            {orderBook.runners.map((runner) => (
              <div key={runner.selection_id} className="runner-card">
                <div className="runner-header">
                  <span className="runner-name">
                    Selection {runner.selection_id}
                  </span>
                  <div className="runner-stats">
                    {runner.last_traded_price != null && (
                      <span>
                        LTP: <span className="value">{runner.last_traded_price.toFixed(2)}</span>
                      </span>
                    )}
                    {runner.traded_volume != null && (
                      <span>
                        Volume:{' '}
                        <span className="value">
                          £{runner.traded_volume.toLocaleString(undefined, { minimumFractionDigits: 0 })}
                        </span>
                      </span>
                    )}
                  </div>
                </div>

                <div className="chart-container">
                  <RunnerDepthChart runner={runner} />
                </div>

                <RunnerLadder runner={runner} />
              </div>
            ))}
          </div>
        </>
      )}
    </div>
    </>
  )
}
