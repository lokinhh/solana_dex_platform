/**
 * In-memory store when USE_MEMORY_DB=true (tests / quick dev).
 */
import { randomUUID } from 'crypto';

export class MemoryStore {
  constructor() {
    this.users = new Map();
    this.wallets = new Map();
    this.trades = [];
    this.copySubs = new Map();
    this.autoRules = new Map();
    this.sentiment = new Map();
  }

  createUser(data) {
    const id = randomUUID();
    const row = { id, createdAt: Date.now(), ...data };
    this.users.set(id, row);
    return row;
  }

  findUserByEmail(email) {
    return [...this.users.values()].find((u) => u.email === email) || null;
  }

  createWallet(data) {
    const id = randomUUID();
    const row = { id, createdAt: Date.now(), ...data };
    this.wallets.set(id, row);
    return row;
  }

  listWallets(userId) {
    return [...this.wallets.values()].filter((w) => w.userId === userId);
  }

  addTrade(data) {
    const row = { id: randomUUID(), ts: Date.now(), ...data };
    this.trades.push(row);
    return row;
  }

  listTrades(limit = 50) {
    return [...this.trades].reverse().slice(0, limit);
  }

  upsertCopySub(id, data) {
    const row = { id, updatedAt: Date.now(), ...data };
    this.copySubs.set(id, row);
    return row;
  }

  listCopySubs(userId) {
    return [...this.copySubs.values()].filter((s) => s.userId === userId);
  }

  listAllCopySubs() {
    return [...this.copySubs.values()];
  }

  upsertAutoRule(id, data) {
    const row = { id, updatedAt: Date.now(), ...data };
    this.autoRules.set(id, row);
    return row;
  }

  listAutoRules(userId) {
    return [...this.autoRules.values()].filter((r) => r.userId === userId);
  }

  listAllAutoRules() {
    return [...this.autoRules.values()];
  }

  setSentiment(mint, data) {
    this.sentiment.set(mint, { mint, updatedAt: Date.now(), ...data });
    return this.sentiment.get(mint);
  }

  listSentiment(limit = 20) {
    return [...this.sentiment.values()]
      .sort((a, b) => b.score - a.score)
      .slice(0, limit);
  }
}

let singleton = null;

export function getStore() {
  if (!singleton) singleton = new MemoryStore();
  return singleton;
}
