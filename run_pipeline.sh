#!/usr/bin/env bash
set -euo pipefail

# ─── Configuration ────────────────────────────────────────────────────────────
# Set these env vars or edit the defaults below.
# They can also be set in ~/.bpipe.env (sourced automatically if it exists).

BPIPE_DIR="${BPIPE_DIR:-$(cd "$(dirname "$0")" && pwd)}"
BPIPE_BIN="${BPIPE_BIN:-$BPIPE_DIR/target/release}"
BPIPE_DATA="${BPIPE_DATA:-$BPIPE_DIR/data}"
BPIPE_LOG="${BPIPE_LOG:-$BPIPE_DATA/logs}"

# Everything this script creates (logs, data files) is private to the owner.
umask 077

# Source optional env file for credentials, warning if others can read it.
for envfile in "$HOME/.bpipe.env" "$BPIPE_DIR/.env"; do
    [[ -f "$envfile" ]] || continue
    perms=$(stat -c %a "$envfile" 2>/dev/null || stat -f %Lp "$envfile")
    if [[ "${perms: -2}" != "00" ]]; then
        echo "WARNING: $envfile is readable by other users (mode $perms). Run: chmod 600 $envfile" >&2
    fi
    source "$envfile"
done

# Betfair credentials (must be set via env vars or .env file).
# They are exported and read by the binaries from the environment, never
# passed as command-line arguments (which any local user can see via ps).
: "${BETFAIR_APP_KEY:?Set BETFAIR_APP_KEY}"
: "${BETFAIR_USERNAME:?Set BETFAIR_USERNAME}"
: "${BETFAIR_PASSWORD:?Set BETFAIR_PASSWORD}"
export BETFAIR_APP_KEY BETFAIR_USERNAME BETFAIR_PASSWORD

# Optional: cert-based auth (only used when both paths are set)
if [[ -n "${BETFAIR_CERT_PATH:-}" && -n "${BETFAIR_KEY_PATH:-}" ]]; then
    export BETFAIR_CERT_PATH BETFAIR_KEY_PATH
else
    unset BETFAIR_CERT_PATH BETFAIR_KEY_PATH
fi

# Betting parameters
STAKE="${BPIPE_STAKE:-2.0}"
MIN_PROB="${BPIPE_MIN_PROB:-50}"
MAX_ODDS="${BPIPE_MAX_ODDS:-}"
PERSISTENCE="${BPIPE_PERSISTENCE:-lapse}"
DRY_RUN="${BPIPE_DRY_RUN:-false}"

# ─── Setup ────────────────────────────────────────────────────────────────────
mkdir -p "$BPIPE_DATA" "$BPIPE_LOG"

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
LOGFILE="$BPIPE_LOG/pipeline_${TIMESTAMP}.log"

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*" | tee -a "$LOGFILE"
}

die() {
    log "ERROR: $*"
    exit 1
}

log "═══════════════════════════════════════════════════════════════"
log "bpipe pipeline started"
log "═══════════════════════════════════════════════════════════════"

# ─── Step 1: Scrape predictions ──────────────────────────────────────────────
SCRAPE_OUTPUT="$BPIPE_DATA/matches_${TIMESTAMP}.json"

log "Step 1/3: Scraping predictions from Statarea..."
"$BPIPE_BIN/bpipe-scrape" \
    --upcoming \
    --output "$SCRAPE_OUTPUT" \
    >> "$LOGFILE" 2>&1 \
    || die "Scraping failed"

COUNT=$(python3 -c "import json; print(len(json.load(open('$SCRAPE_OUTPUT'))))" 2>/dev/null || echo "?")
log "  Scraped $COUNT predictions → $SCRAPE_OUTPUT"

# ─── Step 2: Match against Betfair events ────────────────────────────────────
MATCHED_OUTPUT="$BPIPE_DATA/matched_${TIMESTAMP}.txt"

log "Step 2/3: Matching predictions to Betfair events..."
"$BPIPE_BIN/bpipe" \
    match-events \
    --input "$SCRAPE_OUTPUT" \
    --upcoming \
    --output "$MATCHED_OUTPUT" \
    >> "$LOGFILE" 2>&1 \
    || die "Matching failed"

MATCHED_COUNT=$(grep -c "^[✓ ]" "$MATCHED_OUTPUT" 2>/dev/null | head -1 || echo "?")
log "  Matched events → $MATCHED_OUTPUT ($MATCHED_COUNT lines)"

# ─── Step 3: Place bets ─────────────────────────────────────────────────────
log "Step 3/3: Placing bets..."

BET_ARGS=(
    --input "$MATCHED_OUTPUT"
    --stake "$STAKE"
    --min-prob "$MIN_PROB"
    --upcoming-only
    --yes
    --persistence "$PERSISTENCE"
)

[[ -n "$MAX_ODDS" ]] && BET_ARGS+=(--max-odds "$MAX_ODDS")
[[ "$DRY_RUN" == "true" ]] && BET_ARGS+=(--dry-run)

"$BPIPE_BIN/bpipe-bet" \
    "${BET_ARGS[@]}" \
    2>&1 | tee -a "$LOGFILE"

log "═══════════════════════════════════════════════════════════════"
log "Pipeline complete"
log "═══════════════════════════════════════════════════════════════"
