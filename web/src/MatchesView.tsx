import { useState, useEffect, useCallback, useRef } from 'react'
import type { MatchesResponse, FileListResponse, MatchEntry } from './matchTypes'

const API_BASE = ''
const AUTO_REFRESH_MS = 30_000 // refresh data every 30s

type SortField = 'match_name' | 'date' | 'tip' | 'prob_home' | 'prob_draw' | 'prob_away' | 'bf_event_name' | 'live'
type SortDir = 'asc' | 'desc'

export function MatchesView() {
  const [files, setFiles] = useState<string[]>([])
  const [selectedFile, setSelectedFile] = useState<string | null>(null)
  const [data, setData] = useState<MatchesResponse | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [filter, setFilter] = useState('')
  const [tipFilter, setTipFilter] = useState<string>('all')
  const [statusFilter, setStatusFilter] = useState<string>('all')
  const [sortField, setSortField] = useState<SortField>('date')
  const [sortDir, setSortDir] = useState<SortDir>('asc')
  const refreshTimer = useRef<ReturnType<typeof setInterval> | null>(null)

  // Fetch file list
  useEffect(() => {
    fetch(`${API_BASE}/api/files`)
      .then(res => res.json())
      .then((body: FileListResponse) => {
        setFiles(body.files)
        if (body.files.length > 0 && !selectedFile) {
          setSelectedFile(body.files[0])
        }
      })
      .catch(e => setError(e.message))
  }, [])

  // Fetch matches for selected file
  const fetchMatches = useCallback(() => {
    if (!selectedFile) return
    setLoading(true)
    setError(null)
    fetch(`${API_BASE}/api/matches?file=${encodeURIComponent(selectedFile)}`)
      .then(res => {
        if (!res.ok) throw new Error(`HTTP ${res.status}`)
        return res.json()
      })
      .then((body: MatchesResponse) => {
        setData(body)
        setLoading(false)
      })
      .catch(e => {
        setError(e.message)
        setLoading(false)
      })
  }, [selectedFile])

  useEffect(() => {
    fetchMatches()
  }, [fetchMatches])

  // Auto-refresh every 30 seconds to pick up live status changes
  useEffect(() => {
    if (refreshTimer.current) clearInterval(refreshTimer.current)
    refreshTimer.current = setInterval(() => {
      fetchMatches()
    }, AUTO_REFRESH_MS)
    return () => {
      if (refreshTimer.current) clearInterval(refreshTimer.current)
    }
  }, [fetchMatches])

  // Sort handler
  const handleSort = (field: SortField) => {
    if (sortField === field) {
      setSortDir(d => (d === 'asc' ? 'desc' : 'asc'))
    } else {
      setSortField(field)
      setSortDir('asc')
    }
  }

  const sortIndicator = (field: SortField) => {
    if (sortField !== field) return ''
    return sortDir === 'asc' ? ' ▲' : ' ▼'
  }

  // Filter and sort matches
  const filteredMatches = (data?.matches ?? [])
    .filter((m: MatchEntry) => {
      if (filter) {
        const q = filter.toLowerCase()
        if (
          !m.match_name.toLowerCase().includes(q) &&
          !m.bf_event_name.toLowerCase().includes(q) &&
          !m.bf_event_id.includes(q)
        ) {
          return false
        }
      }
      if (tipFilter !== 'all' && m.tip !== tipFilter) return false
      if (statusFilter === 'result' && !m.has_result) return false
      if (statusFilter === 'pending' && m.has_result) return false
      if (statusFilter === 'live' && !m.live) return false
      if (statusFilter === 'pre-match' && (m.has_result || m.live)) return false
      if (statusFilter === 'closed' && m.market_status !== 'CLOSED') return false
      return true
    })
    .sort((a: MatchEntry, b: MatchEntry) => {
      let cmp = 0
      switch (sortField) {
        case 'match_name':
          cmp = a.match_name.localeCompare(b.match_name)
          break
        case 'date':
          cmp = a.date.localeCompare(b.date)
          break
        case 'tip':
          cmp = a.tip.localeCompare(b.tip)
          break
        case 'prob_home':
          cmp = (a.prob_home ?? 0) - (b.prob_home ?? 0)
          break
        case 'prob_draw':
          cmp = (a.prob_draw ?? 0) - (b.prob_draw ?? 0)
          break
        case 'prob_away':
          cmp = (a.prob_away ?? 0) - (b.prob_away ?? 0)
          break
        case 'bf_event_name':
          cmp = a.bf_event_name.localeCompare(b.bf_event_name)
          break
        case 'live':
          cmp = (a.live ? 1 : 0) - (b.live ? 1 : 0)
          break
      }
      return sortDir === 'asc' ? cmp : -cmp
    })

  // Unique tips for filter dropdown
  const uniqueTips = [...new Set((data?.matches ?? []).map(m => m.tip))].sort()

  // Probability bar component
  const ProbBar = ({ home, draw, away }: { home: number; draw: number; away: number }) => (
    <div className="prob-bar">
      <div className="prob-segment prob-home" style={{ width: `${home}%` }} title={`Home ${home}%`}>
        {home > 15 ? `${home}` : ''}
      </div>
      <div className="prob-segment prob-draw" style={{ width: `${draw}%` }} title={`Draw ${draw}%`}>
        {draw > 15 ? `${draw}` : ''}
      </div>
      <div className="prob-segment prob-away" style={{ width: `${away}%` }} title={`Away ${away}%`}>
        {away > 15 ? `${away}` : ''}
      </div>
    </div>
  )

  const tipBadgeClass = (tip: string) => {
    switch (tip) {
      case '1': return 'tip-badge home'
      case '2': return 'tip-badge away'
      case 'X': return 'tip-badge draw'
      case '1X': return 'tip-badge home-draw'
      case 'X2': return 'tip-badge draw-away'
      case '12': return 'tip-badge home-away'
      default: return 'tip-badge'
    }
  }

  if (loading && !data) {
    return (
      <div className="app">
        <div className="loading">Loading matches…</div>
      </div>
    )
  }

  if (error && !data) {
    return (
      <div className="app">
        <div className="error-msg">Error: {error}</div>
      </div>
    )
  }

  return (
    <div className="app matches-app">
      <div className="header">
        <h1>bpipe — Matched Events</h1>
        {data && (
          <div className="status">
            {data.matches.some(m => m.live) && (
              <span className="stat-pill live-pill">
                <span className="live-dot-pulse" /> {data.matches.filter(m => m.live).length} LIVE
              </span>
            )}
            <span className="stat-pill">{data.total_matched} matched</span>
            <span className="stat-pill dim">{data.total_unmatched} unmatched</span>
            <span className="stat-pill dim">{data.total_scraped} scraped</span>
          </div>
        )}
      </div>

      <div className="controls">
        <div className="controls-left">
          <select
            className="file-select"
            value={selectedFile ?? ''}
            onChange={e => setSelectedFile(e.target.value)}
          >
            {files.map(f => (
              <option key={f} value={f}>
                {f.replace('matched_', '').replace('.txt', '').replace('_', ' ')}
              </option>
            ))}
          </select>

          <input
            className="search-input"
            type="text"
            placeholder="Search matches…"
            value={filter}
            onChange={e => setFilter(e.target.value)}
          />

          <select
            className="filter-select"
            value={tipFilter}
            onChange={e => setTipFilter(e.target.value)}
          >
            <option value="all">All Tips</option>
            {uniqueTips.map(t => (
              <option key={t} value={t}>{t}</option>
            ))}
          </select>

          <select
            className="filter-select"
            value={statusFilter}
            onChange={e => setStatusFilter(e.target.value)}
          >
            <option value="all">All Status</option>
            <option value="live">🔴 Live</option>
            <option value="pre-match">Pre-match</option>
            <option value="result">Has Result</option>
            <option value="closed">Closed</option>
            <option value="pending">Pending</option>
          </select>
        </div>

        <div className="controls-right">
          <button className="refresh-btn" onClick={fetchMatches} disabled={loading}>
            {loading ? '↻' : '↻'} Refresh
          </button>
          <span className="match-count">{filteredMatches.length} shown</span>
        </div>
      </div>

      {data && (data.generated_at || data.last_poll) && (
        <div className="generated-at">
          {data.generated_at && <>Generated: {data.generated_at}</>}
          {data.last_poll && <> · Last Betfair poll: {data.last_poll}</>}
        </div>
      )}

      <div className="table-wrap">
        <table className="matches-table">
          <thead>
            <tr>
              <th className="th-status">St</th>
              <th className="th-sortable th-center" onClick={() => handleSort('live')}>
                Live{sortIndicator('live')}
              </th>
              <th className="th-sortable" onClick={() => handleSort('match_name')}>
                Match{sortIndicator('match_name')}
              </th>
              <th className="th-sortable" onClick={() => handleSort('date')}>
                Date{sortIndicator('date')}
              </th>
              <th className="th-sortable th-center" onClick={() => handleSort('tip')}>
                Tip{sortIndicator('tip')}
              </th>
              <th className="th-prob">Probabilities</th>
              <th className="th-sortable" onClick={() => handleSort('bf_event_name')}>
                Betfair Event{sortIndicator('bf_event_name')}
              </th>
              <th className="th-id">Event ID</th>
            </tr>
          </thead>
          <tbody>
            {filteredMatches.map((m, i) => (
              <tr key={i} className={`${m.has_result ? 'row-result' : ''} ${m.live ? 'row-live' : ''}`}>
                <td className="td-status">
                  {m.has_result ? <span className="result-check">✓</span> : <span className="pending-dot">●</span>}
                </td>
                <td className="td-live">
                  {m.live ? (
                    <span className="live-badge"><span className="live-dot" />LIVE</span>
                  ) : m.market_status === 'CLOSED' ? (
                    <span className="closed-badge">END</span>
                  ) : (
                    <span className="pre-badge">—</span>
                  )}
                </td>
                <td className="td-match">{m.match_name}</td>
                <td className="td-date">{m.date}</td>
                <td className="td-tip">
                  <span className={tipBadgeClass(m.tip)}>{m.tip}</span>
                </td>
                <td className="td-prob">
                  {m.prob_home != null && m.prob_draw != null && m.prob_away != null ? (
                    <ProbBar home={m.prob_home} draw={m.prob_draw} away={m.prob_away} />
                  ) : '—'}
                </td>
                <td className="td-bf-name">{m.bf_event_name}</td>
                <td className="td-bf-id">
                  <code>{m.bf_event_id}</code>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {filteredMatches.length === 0 && (
        <div className="empty-state">No matches found</div>
      )}
    </div>
  )
}
