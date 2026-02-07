# bpipe

A Rust toolkit for the [Betfair Exchange API](https://docs.developer.betfair.com/) — library, CLI, real-time WebSocket relay, React frontend, and football predictions scraper.

## Overview

**bpipe** provides everything you need to interact with the Betfair Exchange programmatically:

| Component | Description |
|---|---|
| **Library** (`src/lib.rs`) | Async Rust client for Betfair REST + Streaming APIs |
| **CLI** (`bpipe`) | Command-line tool for betting, querying markets, streaming, and event matching |
| **WebSocket Relay** (`bpipe-ws`) | Bridges Betfair's TCP streaming to WebSocket clients with order book management |
| **React Frontend** (`web/`) | Live depth charts and price ladders via Recharts |
| **Scraper** (`bpipe-scrape`) | Standalone statarea.com predictions scraper (no Betfair credentials needed) |

## Prerequisites

- **Rust** (edition 2024)
- **Node.js** ≥ 18 (for the React frontend)
- A **Betfair Exchange** account with an [Application Key](https://docs.developer.betfair.com/display/1smk3cen4v3lu3yomq5qye0ni/Application+Keys)
- *(Optional)* Client certificate + key for [certificate-based authentication](https://docs.developer.betfair.com/display/1smk3cen4v3lu3yomq5qye0ni/Non-Interactive+%28bot%29+login)

## Building

```bash
cargo build --release
```

This produces three binaries in `target/release/`:
- `bpipe` — main CLI
- `bpipe-ws` — WebSocket relay server
- `bpipe-scrape` — statarea scraper

## Configuration

Set your Betfair credentials via environment variables or CLI flags:

```bash
export BETFAIR_APP_KEY="your-app-key"
export BETFAIR_USERNAME="your-username"
export BETFAIR_PASSWORD="your-password"

# Optional: certificate-based auth (required for bot/non-interactive login)
export BETFAIR_CERT_PATH="/path/to/client-cert.pem"
export BETFAIR_KEY_PATH="/path/to/client-key.pem"
```

## CLI Usage (`bpipe`)

### List Sports

```bash
bpipe sports
```

### List Events

```bash
# Soccer events (event type ID "1")
bpipe events --sport 1

# Only in-play events
bpipe events --sport 1 --inplay
```

### List Markets for an Event

```bash
bpipe markets --event 35188753
```

### View Market Prices

```bash
bpipe prices --market 1.234567890

# With deeper order book (default: 3 levels)
bpipe prices --market 1.234567890 --depth 10
```

### Place a Bet

```bash
# Back £10 on selection 12345 at odds 2.50
bpipe bet --market 1.234567890 --selection 12345 --side back --stake 10 --price 2.50

# Lay with SP persistence
bpipe bet --market 1.234567890 --selection 12345 --side lay --stake 5 --price 3.00 --persistence sp
```

### Cancel a Bet

```bash
bpipe cancel --market 1.234567890 --bet-id 12345678

# Partial cancellation
bpipe cancel --market 1.234567890 --bet-id 12345678 --amount 5.00
```

### View Open Orders

```bash
bpipe orders

# Filter by market
bpipe orders --market 1.234567890
```

### Account Details

```bash
bpipe account
```

### Stream Live Market Data

```bash
# Stream one or more markets
bpipe stream --markets 1.234567890

# Stream with order updates
bpipe stream --markets 1.234567890,1.234567891 --orders
```

### Scrape & Match (Combined)

```bash
# Scrape statarea predictions and match against Betfair events in one step
bpipe scrape --upcoming
```

### Match Events (from JSON)

```bash
# Read previously scraped matches and match against Betfair
bpipe match-events

# Custom input/output, upcoming only
bpipe match-events --input matches.json --output matched.txt --upcoming
```

## Standalone Scraper (`bpipe-scrape`)

Scrapes football predictions from [statarea.com](https://www.statarea.com/predictions) and saves them as JSON. **No Betfair credentials required.**

```bash
# Default: scrape all predictions → matches.json
bpipe-scrape

# Only upcoming matches, custom output
bpipe-scrape --upcoming --output /tmp/matches.json
```

Output format (`matches.json`):

```json
[
  {
    "competition": "SPAIN - LALIGA",
    "home_team": "Barcelona",
    "away_team": "Mallorca",
    "datetime": "2026-02-07 18:15",
    "tip": "1",
    "prob_home": 75,
    "prob_draw": 17,
    "prob_away": 8,
    "has_result": false
  }
]
```

## WebSocket Relay (`bpipe-ws`)

Bridges Betfair's TCP/TLS streaming protocol to a WebSocket server, maintaining a full order book in memory and broadcasting updates to all connected clients.

```bash
# Stream a market on ws://localhost:9001
bpipe-ws --markets 1.234567890

# Custom port
bpipe-ws --markets 1.234567890 --port 8080
```

### React Frontend

The `web/` directory contains a React + TypeScript + Vite app that connects to `bpipe-ws` and displays:

- **Depth charts** — cumulative back/lay volume at each price level (via Recharts)
- **Price ladders** — best available prices with volumes

```bash
cd web
npm install
npm run dev
```

Then open [http://localhost:5173](http://localhost:5173).

## Event Matching

The `match-events` and `scrape` commands use fuzzy matching to link statarea predictions to Betfair events:

1. **Normalize** team names — lowercase, strip suffixes like `FC`, `CF`, `SC`, `AFC`, `(w)`, etc.
2. **Exact substring match** — check if both normalized team names appear in the Betfair event name (e.g. `"barcelona"` and `"mallorca"` both found in `"Barcelona v Mallorca"`)
3. **Short-name fallback** — if no exact match, try the first word of each team name (≥3 chars) to handle abbreviated names on Betfair

Results are sorted by kick-off time and saved to `matched.txt`.

## Project Structure

```
bpipe/
├── Cargo.toml                  # Workspace config, dependencies, binary targets
├── src/
│   ├── lib.rs                  # Library root — re-exports all modules
│   ├── auth.rs                 # Authentication (interactive + certificate)
│   ├── client.rs               # BetfairClient — REST API wrapper
│   ├── config.rs               # BetfairConfig, ApiUrls
│   ├── error.rs                # BpipeError, Result type
│   ├── betting.rs              # Betting API operations
│   ├── account.rs              # Account API operations
│   ├── streaming.rs            # StreamClient — TCP/TLS streaming
│   ├── orderbook.rs            # OrderBook state from streaming deltas
│   ├── scraper.rs              # Statarea HTML scraper
│   ├── types/
│   │   ├── mod.rs
│   │   ├── common.rs           # MarketFilter, shared types
│   │   ├── betting.rs          # Event, Market, Runner, Order types
│   │   ├── account.rs          # Account details/funds types
│   │   └── streaming.rs        # Streaming protocol message types
│   └── bin/
│       ├── bpipe.rs            # Main CLI binary
│       ├── bpipe_ws.rs         # WebSocket relay binary
│       └── bpipe_scrape.rs     # Standalone scraper binary
└── web/
    ├── package.json
    ├── vite.config.ts
    ├── tsconfig.json
    ├── index.html
    └── src/
        ├── main.tsx
        ├── App.tsx             # Main app component
        ├── RunnerDepthChart.tsx # Recharts depth chart
        ├── RunnerLadder.tsx    # Price ladder component
        ├── useOrderBookSocket.ts # WebSocket hook
        ├── types.ts            # TypeScript types
        └── index.css           # Dark theme styles
```

## License

MIT
