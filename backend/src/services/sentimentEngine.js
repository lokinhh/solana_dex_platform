import { logger } from '../lib/logger.js';

const MOCK_MENTIONS = [
  { source: 'twitter', text: '$PEPE2 going parabolic on pump.fun', mint: 'PumpFunDemoMint1111111111111111111111111111' },
  { source: 'twitter', text: 'aped BONKAI early LFG', mint: 'PumpFunDemoMint2222222222222222222222222222' },
  { source: 'telegram', text: 'WIF2 bonding curve almost done', mint: 'PumpFunDemoMint3333333333333333333333333333' },
];

export class SentimentEngine {
  constructor({ repo, pumpfun, io }) {
    this.repo = repo;
    this.pumpfun = pumpfun;
    this.io = io;
    this.mentionBuffer = [];
    this.timer = null;
  }

  start() {
    const ms = Number(process.env.SENTIMENT_POLL_MS || 15000);
    this.tick();
    this.timer = setInterval(() => this.tick(), ms);
    logger.info('sentiment_engine_started', { pollMs: ms });
  }

  stop() {
    if (this.timer) clearInterval(this.timer);
  }

  ingestMention({ source, text, mint }) {
    this.mentionBuffer.push({ source, text, mint, ts: Date.now() });
    return { ok: true };
  }

  scoreToken(token, mentions) {
    const recent = mentions.filter((m) => m.mint === token.mint);
    const velocity = recent.filter((m) => Date.now() - m.ts < 300_000).length;
    const mentionScore = Math.min(recent.length * 8, 40);
    const velocityScore = Math.min(velocity * 12, 35);
    const holderScore = Math.min((token.holders || 0) / 100, 25);
    const score = Math.round(Math.min(100, mentionScore + velocityScore + holderScore));
    return {
      mint: token.mint,
      symbol: token.symbol,
      score,
      mentions: recent.length,
      velocity,
      sources: {
        twitter: recent.filter((m) => m.source === 'twitter').length,
        telegram: recent.filter((m) => m.source === 'telegram').length,
      },
    };
  }

  async tick() {
    const tokens = await this.pumpfun.listTrending(10);
    const mentions = [...MOCK_MENTIONS, ...this.mentionBuffer].slice(-200);

    for (const token of tokens) {
      const row = this.scoreToken(token, mentions);
      await this.repo.setSentiment(token.mint, row);
      this.io?.emit('sentiment:update', row);
    }
  }

  async listTop(limit = 10) {
    return this.repo.listSentiment(limit);
  }
}
