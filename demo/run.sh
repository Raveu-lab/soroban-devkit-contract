#!/usr/bin/env bash
# Reproduces demo/README.md: calls the live testnet deployments and shows
# their validation rejecting bad input.
#
# Read-only — every call is a simulation, nothing is submitted, no key or
# funded account is needed. If the CLI isn't on your PATH, point SDEV_JS at
# its built entry point:
#
#   SDEV_JS=../soroban-devkit-cli/dist/cli.js ./demo/run.sh
set -euo pipefail

# SDEV_JS holds a path, never a command string: a command string containing
# spaces (and this repo can sit under one) word-splits when expanded.
sdev() {
  if [[ -n "${SDEV_JS:-}" ]]; then node "$SDEV_JS" "$@"; else command sdev "$@"; fi
}
NETWORK="${NETWORK:-testnet}"

if ! sdev --version >/dev/null 2>&1; then
  echo "sdev not found. Install @soroban-devkit/cli, or set SDEV_JS to its dist/cli.js." >&2
  exit 1
fi

# From ADDRESSES.md.
VESTING=CB3DUVIYDN3AJOKHSX6YIXGDVEA3ZPXHHXHTURFP2VYYVHJYHWGONUDL
ESCROW=CB26WWGMKZKREMIRQ5AGKMRY27YEQ3Y4J6MKAVKOARRBQCX5XOZ6VYOC
TOKEN=CCEBWHXYP3Y7BGLI7DUZWZJXGXIRHLSZJR7PGCDD3ZKINCV4Y4S55DLJ

# Any three existing testnet accounts; only their public keys are used.
A=GC3BJ52UL6CBAJBSZRX5DIPHXUW6OGJFVWMQKV3GLBMXHEZ66BWLXZN6
B=GCTQDPY6S7OZDBIM7MYXFQVTLOGGIXAGKKF7LYYUBYXWOBCCKIEBSPH2
C=GDKYA6PQEDTGNBN7EVYVMLDL6UNZFQDP7YKYRUOOJDVK2JYQREZYIOHS

step() { printf '\n\033[1m── %s\033[0m\n\n' "$1"; }

vest() { # cliff, duration
  sdev simulate --contract "$VESTING" --method create_vesting --caller "$A" \
    --network "$NETWORK" \
    --args "[\"$A\",\"$B\",\"$TOKEN\",\"1000\",{\"\$u64\":\"0\"},{\"\$u64\":\"$1\"},{\"\$u64\":\"$2\"}]"
}

escrow() { # arbiter
  sdev simulate --contract "$ESCROW" --method deposit --caller "$A" \
    --network "$NETWORK" \
    --args "[\"$A\",\"$B\",\"$1\",\"$TOKEN\",\"1000\",{\"\$u64\":\"4000000000\"}]"
}

# A guard firing makes sdev exit non-zero, which is the expected result for
# half of these, so the exit code is ignored — but empty output means the
# call never reached the network, which is a real failure.
show() { # pattern, output
  local matched
  matched=$(printf '%s\n' "$2" | grep -E "$1" | head -3 || true)
  if [[ -z "$matched" ]]; then
    echo "no matching output — the call did not reach the network:" >&2
    printf '%s\n' "$2" | head -5 >&2
    exit 1
  fi
  printf '%s\n' "$matched"
}

step "vesting — cliff (1000) outlasts duration (100): expect Error(Contract, #7)"
show "✖|Error\(Contract" "$(vest 1000 100 2>&1 || true)"

step "vesting — cliff (100) inside duration (1000): expect success"
show "✔|Return Value" "$(vest 100 1000 2>&1 || true)"

step "escrow — arbiter is also the depositor: expect rejection"
show "✖|VM call trapped" "$(escrow "$A" 2>&1 || true)"

step "escrow — neutral third-party arbiter: expect success"
show "✔|Return Value" "$(escrow "$C" 2>&1 || true)"
