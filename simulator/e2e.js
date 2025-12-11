/**
 * E2E: auth → wallet → quote → trade → copy → webhook
 */
import { spawn } from 'child_process';
import path from 'path';
import { fileURLToPath } from 'url';

const ROOT = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
const BASE = process.env.API_BASE || 'http://127.0.0.1:8091';
const API_SECRET = process.env.API_SECRET || 'dev-api-secret';

function sleep(ms) {
  return new Promise((r) => setTimeout(r, ms));
}

async function waitHealth() {
  for (let i = 0; i < 50; i++) {
    try {
      const r = await fetch(`${BASE}/health`);
      if (r.ok) return r.json();
    } catch { /* retry */ }
    await sleep(200);
  }
  throw new Error('health timeout');
}

function startApi() {
  return spawn('node', ['src/server.js'], {
    cwd: path.join(ROOT, 'backend'),
    env: {
      ...process.env,
      PORT: '8091',
      USE_MEMORY_DB: 'true',
      PAPER_TRADING: 'true',
      API_SECRET,
      JWT_SECRET: 'e2e-jwt-secret-32-characters-long',
    },
    stdio: ['ignore', 'pipe', 'pipe'],
  });
}

function authHeaders(token) {
  return {
    'Content-Type': 'application/json',
    Authorization: `Bearer ${token}`,
  };
}

export async function runE2E() {
  const child = startApi();
  child.stdout.on('data', (d) => process.stdout.write(`[api] ${d}`));

  try {
    const health = await waitHealth();
    if (!health.ok) throw new Error('health not ok');

    const reg = await (await fetch(`${BASE}/api/v1/auth/register`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ email: `e2e-${Date.now()}@test.com`, password: 'password123', name: 'E2E' }),
    })).json();
    if (!reg.token) throw new Error('register failed');
    const h = authHeaders(reg.token);

    const wallet = await (await fetch(`${BASE}/api/v1/wallets`, { method: 'POST', headers: h, body: '{}' })).json();
    if (!wallet.wallet?.id) throw new Error('wallet create failed');

    const quote = await (await fetch(`${BASE}/api/v1/trade/quote`, {
      method: 'POST',
      headers: h,
      body: JSON.stringify({
        mint: 'PumpFunDemoMint1111111111111111111111111111',
        side: 'buy',
        amountSol: 0.05,
      }),
    })).json();
    if (!quote.quote) throw new Error('quote failed');

    const built = await (await fetch(`${BASE}/api/v1/trade/build`, {
      method: 'POST',
      headers: h,
      body: JSON.stringify({ quote: quote.quote, walletPublicKey: wallet.wallet.publicKey }),
    })).json();
    if (!built.swapTransaction) throw new Error('build failed');

    const trade = await (await fetch(`${BASE}/api/v1/trade/submit`, {
      method: 'POST',
      headers: { ...h, 'Idempotency-Key': 'e2e-trade-1' },
      body: JSON.stringify({
        mint: 'PumpFunDemoMint1111111111111111111111111111',
        symbol: 'PEPE2',
        side: 'buy',
        amountSol: 0.05,
        walletPublicKey: wallet.wallet.publicKey,
        walletId: wallet.wallet.id,
        signedTransaction: built.swapTransaction,
      }),
    })).json();
    if (!trade.trade?.txSignature) throw new Error('submit failed');

    await fetch(`${BASE}/api/v1/copy/subscribe`, {
      method: 'POST',
      headers: h,
      body: JSON.stringify({
        leaderAddress: 'LeaderWallet1111111111111111111111111111',
        followerWalletId: wallet.wallet.id,
        followerPublicKey: wallet.wallet.publicKey,
        sizePct: 50,
      }),
    });

    const webhook = await (await fetch(`${BASE}/webhooks/helius`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${API_SECRET}` },
      body: JSON.stringify({
        leaderAddress: 'LeaderWallet1111111111111111111111111111',
        signature: `wh-${Date.now()}`,
        mint: 'PumpFunDemoMint1111111111111111111111111111',
        symbol: 'PEPE2',
        side: 'buy',
        amountSol: 0.02,
      }),
    })).json();
    if (!webhook.ok) throw new Error('webhook failed');

    const sentiment = await (await fetch(`${BASE}/api/v1/sentiment`, { headers: h })).json();
    if (!sentiment.scores?.length) throw new Error('no sentiment');

    const noAuth = await fetch(`${BASE}/api/v1/trades`);
    if (noAuth.status !== 401) throw new Error('expected 401');

    return { ok: true, mode: health.mode, db: health.db };
  } finally {
    child.kill('SIGTERM');
    await sleep(300);
  }
}

if (import.meta.url === `file://${process.argv[1]}` || process.argv[1]?.endsWith('e2e.js')) {
  runE2E()
    .then((s) => { console.log('\nE2E OK', s); process.exit(0); })
    .catch((e) => { console.error('\nE2E FAIL', e); process.exit(1); });
}
