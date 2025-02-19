import { logger } from '../lib/logger.js';

export class AutoTradeEngine {
  constructor({ repo, executor, sentiment, io }) {
    this.repo = repo;
    this.executor = executor;
    this.sentiment = sentiment;
    this.io = io;
    this.timer = null;
    this.dailyCount = 0;
    this.dayKey = new Date().toISOString().slice(0, 10);
  }

  start() {
    this.timer = setInterval(() => this.evaluate(), 20000);
    logger.info('auto_trade_engine_started');
  }

  stop() {
    if (this.timer) clearInterval(this.timer);
  }

  async createRule({ userId, walletId, mint, minSentiment = 70, maxTradeSol = 0.1 }) {
    const id = `${userId}:${mint}`;
    return this.repo.upsertAutoRule(id, {
      userId,
      walletId,
      mint,
      minSentiment,
      maxTradeSol,
      active: true,
    });
  }

  async list(userId) {
    return this.repo.listAutoRules(userId);
  }

  _syncDay() {
    const key = new Date().toISOString().slice(0, 10);
    if (key !== this.dayKey) {
      this.dayKey = key;
      this.dailyCount = 0;
    }
  }

  async evaluate() {
    this._syncDay();
    const maxDaily = Number(process.env.MAX_DAILY_TRADES || 50);
    if (this.dailyCount >= maxDaily) return;

    const rules = await this.repo.listAllAutoRules();
    const scores = await this.sentiment.listTop(20);
    const scoreMap = new Map(scores.map((s) => [s.mint, s]));

    for (const rule of rules) {
      const sent = scoreMap.get(rule.mint);
      if (!sent || sent.score < rule.minSentiment) continue;

      try {
        const wallet = await this.repo.findWallet(rule.userId, rule.walletId);
        if (!wallet) continue;

        const trade = await this.executor.submit({
          userId: rule.userId,
          walletId: rule.walletId,
          walletPublicKey: wallet.publicKey,
          mint: rule.mint,
          symbol: sent.symbol,
          side: 'buy',
          amountSol: rule.maxTradeSol,
          source: 'auto',
          signedTransaction: 'paper',
          idempotencyKey: `auto:${rule.id}:${this.dayKey}`,
        });
        this.dailyCount += 1;
        this.io?.emit('auto:trade', { rule, trade, sentiment: sent });
      } catch (err) {
        if (!String(err).includes('duplicate')) {
          logger.warn('auto_trade_failed', { mint: rule.mint, err: String(err) });
        }
      }
    }
  }
}
