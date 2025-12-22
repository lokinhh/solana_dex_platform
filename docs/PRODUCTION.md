# Production deployment

## Checklist

- [ ] Set `JWT_SECRET` and `API_SECRET` (32+ random chars each)
- [ ] Set `USE_MEMORY_DB=false` + `MONGODB_URI` with persistent volume
- [ ] Configure `SOLANA_RPC_URL` (Helius / QuickNode — not public RPC)
- [ ] Set `HELIUS_WEBHOOK_SECRET` for copy-trade webhooks
- [ ] Start with `PAPER_TRADING=true`, then `PAPER_TRADING=false` on devnet
- [ ] Frontend: `NEXT_PUBLIC_PAPER_TRADING=false` for live sign flow
- [ ] HTTPS reverse proxy for API + WebSocket
- [ ] **Non-custodial**: prefer Phantom `wallet/link` over platform wallet secrets

## E2E flows (implemented)

| Flow | Endpoint |
|---|---|
| Register / Login | `POST /api/v1/auth/register`, `/login` → JWT |
| Link Phantom | `POST /api/v1/wallets/link` |
| Quote → Build → Sign → Submit | `/trade/quote`, `/trade/build`, `/trade/submit` |
| Copy trade webhook | `POST /webhooks/helius` |
| On-chain leader poll | `ONCHAIN_POLL_MS` when not paper |
| Sentiment realtime | WebSocket `sentiment:update` |

## Modes

| Env | Description |
|---|---|
| `PAPER_TRADING=true` | Simulated swaps — safe demo |
| `PAPER_TRADING=false` + devnet | Jupiter quote/build + Phantom sign + RPC submit |
| `mainnet-beta` | Production — audit required |

## Docker

```bash
export API_SECRET="$(openssl rand -hex 24)"
export JWT_SECRET="$(openssl rand -hex 32)"
docker compose up -d --build
```

## MongoDB

```env
USE_MEMORY_DB=false
MONGODB_URI=mongodb://mongo:27017/solana_dex
```

## Helius copy-trade webhook

```bash
curl -X POST https://your-api/webhooks/helius \
  -H "Authorization: Bearer $HELIUS_WEBHOOK_SECRET" \
  -d '{"leaderAddress":"...","signature":"...","mint":"...","side":"buy","amountSol":0.05}'
```
