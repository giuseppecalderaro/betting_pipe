export interface MatchEntry {
  has_result: boolean
  match_name: string
  date: string
  tip: string
  prob_home: number | null
  prob_draw: number | null
  prob_away: number | null
  bf_event_id: string
  bf_event_name: string
  live?: boolean
  market_status?: string
  score?: string
}

export interface MatchesResponse {
  file_name: string
  generated_at: string
  total_scraped: number
  total_matched: number
  total_unmatched: number
  last_poll: string | null
  matches: MatchEntry[]
}

export interface FileListResponse {
  files: string[]
}
