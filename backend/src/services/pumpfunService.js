import { logger } from '../lib/logger.js';

const MOCK_TOKENS = [
  {
    mint: 'So11111111111111111111111111111111111111112',
    symbol: 'SOL',
    name: 'Wrapped SOL',
    priceUsd: 145.2,
    marketCap: 68_000_000_000,
    bondingCurvePct: 100,
    source: 'native',
  },
  {
    mint: 'PumpFunDemoMint1111111111111111111111111111',
    symbol: 'PEPE2',
    name: 'Pepe 2.0',
    priceUsd: 0.000042,
    marketCap: 420_000,
    bondingCurvePct: 78,
    holders: 1240,
    source: 'pump.fun',
  },
  {
    mint: 'PumpFunDemoMint2222222222222222222222222222',
    symbol: 'BONKAI',
    name: 'Bonk AI',
    priceUsd: 0.000018,
    marketCap: 180_000,
    bondingCurvePct: 45,
    holders: 890,
    source: 'pump.fun',
  },
  {
    mint: 'PumpFunDemoMint3333333333333333333333333333',
    symbol: 'WIF2',
    name: 'Wif Sequel',
    priceUsd: 0.000095,
    marketCap: 950_000,
    bondingCurvePct: 92,
    holders: 3100,
    source: 'pump.fun',
  },
];

export function createPumpfunService() {
  const paper = process.env.PAPER_TRADING !== 'false';
  const baseUrl = process.env.PUMPFUN_API_URL || 'https://frontend-api.pump.fun';

  return {
    async listTrending(limit = 20) {
      if (paper) {
        return MOCK_TOKENS.slice(0, limit).map((t) => ({
          ...t,
          volume24h: Math.floor(Math.random() * 500_000) + 10_000,
          createdAt: Date.now() - Math.floor(Math.random() * 86400000),
        }));
      }
      try {
        const res = await fetch(`${baseUrl}/coins/trending?limit=${limit}`);
        if (!res.ok) throw new Error(`pumpfun_http_${res.status}`);
        return await res.json();
      } catch (err) {
        logger.warn('pumpfun_fetch_failed', { err: String(err) });
        return MOCK_TOKENS;
      }
    },

    async getToken(mint) {
      const list = await this.listTrending(50);
      return list.find((t) => t.mint === mint) || null;
    },
  };
}
