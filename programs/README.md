# Rust workspace

Solana on-chain programs and Rust services for the DEX copy-trade platform.

## Crates

| Crate | Role |
|---|---|
| `trade-registry` | Solana BPF program — registry, leaders, subscriptions, intent logs |
| `dex-core` | Shared Borsh layouts, PDAs, instruction codec, tx builders |
| `dex-solana` | RPC client, paper ledger, balance and signature polling |
| `jupiter-client` | Jupiter v6 quote + swap transaction builder |
| `pumpfun-client` | Pump.fun trending tokens with paper fixtures |
| `sentiment-core` | Social mention buffer + token sentiment scoring |
| `onchain-indexer` | Leader signature polling + Helius webhook normalization |
| `dex-api` | Axum HTTP API — sentiment, trending, registry PDAs, copy simulate |
| `dex-worker` | Background worker binary (sentiment tick, on-chain poll) |
| `dex-cli` | CLI to derive PDAs and hex-encode instructions |

## Instructions (on-chain)

| Instruction | Description |
|---|---|
| `InitializeRegistry` | Create global registry PDA (authority signer) |
| `RegisterLeader` | Register a leader wallet with follower cap |
| `UpdateLeader` | Pause leader or change follower cap |
| `Subscribe` | Follower mirrors a leader (`size_bps` = 0.01% units) |
| `Unsubscribe` | Deactivate a subscription |
| `LogCopyIntent` | On-chain audit log for mirrored buy/sell |

## Commands

```bash
cd programs
cargo test --workspace
cargo build --workspace

# Worker (JSON output)
cargo run -p dex-worker -- sentiment-tick --limit 10
cargo run -p dex-worker -- onchain-poll --leaders LeaderWallet111111111111111111111111111111

# CLI helpers
cargo run -p dex-cli -- pda-registry
cargo run -p dex-cli -- encode-subscribe --size-bps 5000
```

## PDAs

- Registry: `["registry"]`
- Leader profile: `["leader", leader_pubkey]`
- Subscription: `["sub", follower_pubkey, leader_pubkey]`

## Node.js integration

- `backend/src/services/tradeRegistryClient.js` mirrors `dex-core` instruction layouts
- Set `TRADE_REGISTRY_ENABLED=true` to log copy intents
- Optional: run `dex-worker` as a sidecar for Rust-native sentiment / indexer jobs
