import { Router } from 'express';
import { z } from 'zod';
import { requireAuth, verifyWebhookSecret } from '../middleware/auth.js';
import {
  encodeLogCopyIntent,
  findLeaderPda,
  findRegistryPda,
  findSubscriptionPda,
  getTradeRegistryProgramId,
  isTradeRegistryEnabled,
} from '../services/tradeRegistryClient.js';

const tradeSchema = z.object({
  walletId: z.string().min(1).optional(),
  walletPublicKey: z.string().min(32).optional(),
  mint: z.string().min(32),
  symbol: z.string().optional(),
  side: z.enum(['buy', 'sell']),
  amountSol: z.number().positive(),
});

const quoteSchema = z.object({
  mint: z.string().min(32),
  side: z.enum(['buy', 'sell']),
  amountSol: z.number().positive(),
  walletPublicKey: z.string().min(32).optional(),
});

const submitSchema = tradeSchema.extend({
  signedTransaction: z.string().min(1),
  walletPublicKey: z.string().min(32),
  walletId: z.string().optional(),
});

const copySchema = z.object({
  leaderAddress: z.string().min(32),
  followerWalletId: z.string().min(1).optional(),
  followerPublicKey: z.string().min(32).optional(),
  sizePct: z.number().min(1).max(100).default(100),
});

const autoSchema = z.object({
  walletId: z.string().min(1),
  mint: z.string().min(32),
  minSentiment: z.number().min(0).max(100).default(70),
  maxTradeSol: z.number().positive().default(0.1),
});

const linkWalletSchema = z.object({
  publicKey: z.string().min(32),
  label: z.string().default('Phantom Wallet'),
});

export function createApiRouter(deps) {
  const router = Router();
  router.use(requireAuth);

  router.get('/tokens/trending', async (_req, res) => {
    const tokens = await deps.pumpfun.listTrending();
    res.json({ ok: true, tokens });
  });

  router.get('/tokens/:mint', async (req, res) => {
    const token = await deps.pumpfun.getToken(req.params.mint);
    if (!token) return res.status(404).json({ error: 'token_not_found' });
    res.json({ ok: true, token });
  });

  router.get('/sentiment', async (_req, res) => {
    res.json({ ok: true, scores: await deps.sentiment.listTop() });
  });

  router.post('/sentiment/ingest', (req, res) => {
    const row = deps.sentiment.ingestMention(req.body);
    res.status(201).json({ ok: true, ...row });
  });

  router.post('/wallets', async (req, res) => {
    const { label } = req.body || {};
    const kp = deps.solana.generateWallet();
    const wallet = await deps.repo.createWallet({
      userId: req.userId,
      publicKey: kp.publicKey,
      label: label || 'Platform Wallet',
      encryptedSecret: deps.solana.encryptSecret(kp.secretKey),
      isPlatform: true,
    });
    res.status(201).json({
      ok: true,
      wallet: { id: wallet.id, publicKey: wallet.publicKey, label: wallet.label },
    });
  });

  router.post('/wallets/link', async (req, res) => {
    try {
      const body = linkWalletSchema.parse(req.body);
      const wallet = await deps.repo.createWallet({
        userId: req.userId,
        publicKey: body.publicKey,
        label: body.label,
        isPlatform: false,
        isExternal: true,
      });
      res.status(201).json({ ok: true, wallet: { id: wallet.id, publicKey: wallet.publicKey, label: wallet.label } });
    } catch (err) {
      res.status(400).json({ error: err.message });
    }
  });

  router.get('/wallets', async (req, res) => {
    const wallets = await deps.repo.listWallets(req.userId);
    res.json({
      ok: true,
      wallets: wallets.map((w) => ({
        id: w.id,
        publicKey: w.publicKey,
        label: w.label,
        isExternal: w.isExternal,
      })),
    });
  });

  router.get('/wallets/:id/balance', async (req, res) => {
    const w = await deps.repo.findWallet(req.userId, req.params.id);
    if (!w) return res.status(404).json({ error: 'wallet_not_found' });
    const balance = await deps.solana.getBalanceSol(w.publicKey);
    res.json({ ok: true, publicKey: w.publicKey, balanceSol: balance });
  });

  router.get('/balance/:publicKey', async (req, res) => {
    const balance = await deps.solana.getBalanceSol(req.params.publicKey);
    res.json({ ok: true, publicKey: req.params.publicKey, balanceSol: balance });
  });

  router.post('/trade/quote', async (req, res) => {
    try {
      const body = quoteSchema.parse(req.body);
      const result = await deps.executor.quote(body);
      res.json({ ok: true, ...result });
    } catch (err) {
      res.status(400).json({ error: err.message });
    }
  });

  router.post('/trade/build', async (req, res) => {
    try {
      const { quote, walletPublicKey } = req.body;
      if (!quote || !walletPublicKey) throw new Error('quote_and_wallet_required');
      const built = await deps.executor.build({ quote, walletPublicKey });
      res.json({ ok: true, ...built });
    } catch (err) {
      res.status(400).json({ error: err.message });
    }
  });

  router.post('/trade/submit', async (req, res) => {
    try {
      const body = submitSchema.parse(req.body);
      const idempotencyKey = req.headers['idempotency-key'] || req.headers['x-idempotency-key'];
      const trade = await deps.executor.submit({
        userId: req.userId,
        ...body,
        idempotencyKey: idempotencyKey ? String(idempotencyKey) : undefined,
      });
      const status = trade.duplicate ? 200 : 201;
      res.status(status).json({ ok: true, trade });
    } catch (err) {
      res.status(400).json({ error: err.message });
    }
  });

  router.post('/trade', async (req, res) => {
    try {
      const body = tradeSchema.parse(req.body);
      const trade = await deps.executor.execute({ userId: req.userId, ...body });
      res.status(201).json({ ok: true, trade });
    } catch (err) {
      res.status(400).json({ error: err.message });
    }
  });

  router.get('/trades', async (req, res) => {
    const trades = await deps.repo.listTrades(req.userId);
    res.json({ ok: true, trades });
  });

  router.post('/copy/subscribe', async (req, res) => {
    try {
      const body = copySchema.parse(req.body);
      const sub = await deps.copyTrade.subscribe({ userId: req.userId, ...body });
      res.status(201).json({ ok: true, subscription: sub });
    } catch (err) {
      res.status(400).json({ error: err.message });
    }
  });

  router.get('/copy/subscriptions', async (req, res) => {
    res.json({ ok: true, subscriptions: await deps.copyTrade.list(req.userId) });
  });

  router.post('/copy/simulate', async (req, res) => {
    const { leaderAddress, mint, symbol, side, amountSol } = req.body;
    const results = await deps.copyTrade.simulateLeaderTrade(leaderAddress, {
      mint,
      symbol,
      side: side || 'buy',
      amountSol: amountSol || 0.05,
    });
    res.json({ ok: true, mirrored: results.length, trades: results });
  });

  router.post('/auto/rules', async (req, res) => {
    try {
      const body = autoSchema.parse(req.body);
      const rule = await deps.autoTrade.createRule({ userId: req.userId, ...body });
      res.status(201).json({ ok: true, rule });
    } catch (err) {
      res.status(400).json({ error: err.message });
    }
  });

  router.get('/auto/rules', async (req, res) => {
    res.json({ ok: true, rules: await deps.autoTrade.list(req.userId) });
  });

  router.get('/registry/status', (_req, res) => {
    res.json({
      ok: true,
      enabled: isTradeRegistryEnabled(),
      programId: getTradeRegistryProgramId().toBase58(),
    });
  });

  router.get('/registry/pdas', (req, res) => {
    const { leader, follower } = req.query;
    const [registry, registryBump] = findRegistryPda();
    const payload = {
      registry: registry.toBase58(),
      registryBump,
      programId: getTradeRegistryProgramId().toBase58(),
    };

    if (typeof leader === 'string' && leader.length >= 32) {
      const [leaderPda, leaderBump] = findLeaderPda(leader);
      payload.leaderPda = leaderPda.toBase58();
      payload.leaderBump = leaderBump;
    }

    if (
      typeof leader === 'string' &&
      typeof follower === 'string' &&
      leader.length >= 32 &&
      follower.length >= 32
    ) {
      const [subscriptionPda, subscriptionBump] = findSubscriptionPda(follower, leader);
      payload.subscriptionPda = subscriptionPda.toBase58();
      payload.subscriptionBump = subscriptionBump;
      payload.logCopyIntentData = encodeLogCopyIntent({
        action: 'buy',
        mint: 'PumpFunDemoMint1111111111111111111111111111',
        amountLamports: 50_000_000,
        referenceSig: Buffer.alloc(64),
      }).toString('hex');
    }

    res.json({ ok: true, ...payload });
  });

  return router;
}

export function createWebhookRouter(deps) {
  const router = Router();
  router.post('/helius', verifyWebhookSecret, async (req, res) => {
    const events = Array.isArray(req.body) ? req.body : [req.body];
    const results = [];
    for (const event of events) {
      const trades = await deps.onchainWatcher.handleWebhookEvent(event);
      results.push(...trades);
    }
    res.json({ ok: true, processed: events.length, mirrored: results.length });
  });
  return router;
}
