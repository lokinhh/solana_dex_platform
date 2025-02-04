import { logger } from '../lib/logger.js';
import { encodeLogCopyIntent, isTradeRegistryEnabled } from './tradeRegistryClient.js';

export class CopyTradeEngine {
  constructor({ repo, executor, io }) {
    this.repo = repo;
    this.executor = executor;
    this.io = io;
    this.timer = null;
  }

  start() {
    if (!this.executor) return;
    this.timer = setInterval(() => this.pollPaperLeaders(), 15000);
    logger.info('copy_trade_engine_started');
  }

  stop() {
    if (this.timer) clearInterval(this.timer);
  }

  async subscribe({ userId, leaderAddress, followerWalletId, followerPublicKey, sizePct = 100 }) {
    const id = `${userId}:${leaderAddress}`;
    const sub = await this.repo.upsertCopySub(id, {
      userId,
      leaderAddress,
      followerWalletId,
      followerPublicKey,
      sizePct,
      active: true,
    });
    return sub;
  }

  async list(userId) {
    return this.repo.listCopySubs(userId);
  }

  async handleLeaderActivity(leaderAddress, { signature, mint, symbol, side, amountSol }) {
    const subs = (await this.repo.listAllCopySubs()).filter(
      (s) => s.active && s.leaderAddress === leaderAddress,
    );
    if (!subs.length) return [];

    const results = [];
    for (const sub of subs) {
      const scaled = (amountSol * sub.sizePct) / 100;
      try {
        const wallet = await this.repo.findWallet(sub.userId, sub.followerWalletId);
        const pubkey = sub.followerPublicKey || wallet?.publicKey;
        if (!pubkey) continue;

        const trade = await this.executor.submit({
          userId: sub.userId,
          walletId: sub.followerWalletId,
          walletPublicKey: pubkey,
          mint: mint || 'PumpFunDemoMint1111111111111111111111111111',
          symbol: symbol || 'TOKEN',
          side: side || 'buy',
          amountSol: scaled,
          source: 'copy',
          leaderWallet: leaderAddress,
          signedTransaction: 'paper',
          idempotencyKey: `copy:${signature}:${sub.userId}`,
        });
        results.push(trade);
        this.io?.emit('copy:trade', { sub, trade, leaderSignature: signature });

        if (isTradeRegistryEnabled()) {
          const registryPayload = encodeLogCopyIntent({
            action: side || 'buy',
            mint: mint || 'PumpFunDemoMint1111111111111111111111111111',
            amountLamports: Math.floor(scaled * 1e9),
            referenceSig: signature,
          });
          logger.info('trade_registry_intent', {
            follower: pubkey,
            leader: leaderAddress,
            instructionHex: registryPayload.toString('hex'),
          });
        }
      } catch (err) {
        logger.warn('copy_mirror_failed', { leader: leaderAddress, err: String(err) });
      }
    }
    return results;
  }

  async simulateLeaderTrade(leaderAddress, payload) {
    return this.handleLeaderActivity(leaderAddress, {
      signature: `sim-${Date.now()}`,
      ...payload,
    });
  }

  async pollPaperLeaders() {
    if (process.env.PAPER_TRADING === 'false') return;
    const subs = await this.repo.listAllCopySubs();
    if (!subs.length || Math.random() > 0.35) return;

    const sub = subs[Math.floor(Math.random() * subs.length)];
    await this.handleLeaderActivity(sub.leaderAddress, {
      signature: `paper-leader-${Date.now()}`,
      mint: 'PumpFunDemoMint1111111111111111111111111111',
      symbol: 'PEPE2',
      side: 'buy',
      amountSol: 0.05,
    });
  }
}
