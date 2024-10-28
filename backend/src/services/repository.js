/**
 * Unified data layer — MongoDB in production, in-memory for tests.
 */
import { randomUUID } from 'crypto';
import { connectDb } from '../lib/db.js';
import { Trade, Wallet, CopySub, AutoRule, Sentiment, User } from '../models/index.js';
import { MemoryStore } from './memoryStore.js';

function docId(doc) {
  const id = doc.id || doc._id?.toString();
  return { ...doc.toObject?.() || doc, id };
}

let repo = null;
let memoryStore = null;

export class MemoryRepository {
  constructor(store) {
    this.store = store;
  }

  async createUser(data) {
    return this.store.createUser(data);
  }

  async findUserByEmail(email) {
    return this.store.findUserByEmail(email);
  }

  async createWallet(data) {
    return this.store.createWallet(data);
  }

  async listWallets(userId) {
    return this.store.listWallets(userId);
  }

  async findWallet(userId, walletId) {
    return this.store.listWallets(userId).find((w) => w.id === walletId) || null;
  }

  async addTrade(data) {
    return this.store.addTrade(data);
  }

  async listTrades(userId, limit = 100) {
    return this.store.listTrades(limit).filter((t) => t.userId === userId);
  }

  async listAllTrades(limit = 100) {
    return this.store.listTrades(limit);
  }

  async upsertCopySub(id, data) {
    return this.store.upsertCopySub(id, data);
  }

  async listCopySubs(userId) {
    return this.store.listCopySubs(userId);
  }

  async listAllCopySubs() {
    return this.store.listAllCopySubs();
  }

  async upsertAutoRule(id, data) {
    return this.store.upsertAutoRule(id, data);
  }

  async listAutoRules(userId) {
    return this.store.listAutoRules(userId);
  }

  async listAllAutoRules() {
    return this.store.listAllAutoRules();
  }

  async setSentiment(mint, data) {
    return this.store.setSentiment(mint, data);
  }

  async listSentiment(limit = 20) {
    return this.store.listSentiment(limit);
  }

  async getIdempotency(key) {
    return this.store.idempotency?.get(key) || null;
  }

  async setIdempotency(key, result) {
    if (!this.store.idempotency) this.store.idempotency = new Map();
    this.store.idempotency.set(key, { result, expiresAt: Date.now() + 48 * 3600_000 });
    return result;
  }

  async getLeaderCursor(leader) {
    return this.store.leaderCursors?.get(leader) || null;
  }

  async setLeaderCursor(leader, signature) {
    if (!this.store.leaderCursors) this.store.leaderCursors = new Map();
    this.store.leaderCursors.set(leader, signature);
  }
}

class MongoRepository {
  async createUser(data) {
    const row = await User.create(data);
    return docId(row);
  }

  async findUserByEmail(email) {
    const row = await User.findOne({ email });
    return row ? docId(row) : null;
  }

  async createWallet(data) {
    const row = await Wallet.create(data);
    return docId(row);
  }

  async listWallets(userId) {
    const rows = await Wallet.find({ userId }).sort({ createdAt: -1 });
    return rows.map(docId);
  }

  async findWallet(userId, walletId) {
    const row = await Wallet.findOne({ _id: walletId, userId });
    return row ? docId(row) : null;
  }

  async addTrade(data) {
    const row = await Trade.create(data);
    return docId(row);
  }

  async listTrades(userId, limit = 100) {
    const rows = await Trade.find({ userId }).sort({ createdAt: -1 }).limit(limit);
    return rows.map(docId);
  }

  async listAllTrades(limit = 100) {
    const rows = await Trade.find().sort({ createdAt: -1 }).limit(limit);
    return rows.map(docId);
  }

  async upsertCopySub(id, data) {
    const row = await CopySub.findOneAndUpdate(
      { _id: id },
      { ...data, _id: id },
      { upsert: true, new: true, setDefaultsOnInsert: true },
    );
    return docId(row);
  }

  async listCopySubs(userId) {
    const rows = await CopySub.find({ userId });
    return rows.map(docId);
  }

  async listAllCopySubs() {
    const rows = await CopySub.find({ active: true });
    return rows.map(docId);
  }

  async upsertAutoRule(id, data) {
    const row = await AutoRule.findOneAndUpdate(
      { _id: id },
      { ...data, _id: id },
      { upsert: true, new: true, setDefaultsOnInsert: true },
    );
    return docId(row);
  }

  async listAutoRules(userId) {
    const rows = await AutoRule.find({ userId });
    return rows.map(docId);
  }

  async listAllAutoRules() {
    const rows = await AutoRule.find({ active: true });
    return rows.map(docId);
  }

  async setSentiment(mint, data) {
    const row = await Sentiment.findOneAndUpdate({ mint }, data, { upsert: true, new: true });
    return docId(row);
  }

  async listSentiment(limit = 20) {
    const rows = await Sentiment.find().sort({ score: -1 }).limit(limit);
    return rows.map(docId);
  }

  async getIdempotency(key) {
    const row = await Trade.findOne({ idempotencyKey: key });
    return row ? docId(row) : null;
  }

  async setIdempotency(key, result) {
    return result;
  }

  async getLeaderCursor(leader) {
    const row = await CopySub.findOne({ leaderAddress: leader });
    return row?.lastSignature || null;
  }

  async setLeaderCursor(leader, signature) {
    await CopySub.updateMany({ leaderAddress: leader }, { lastSignature: signature });
  }
}

export function createRepository() {
  if (repo) return repo;
  if (process.env.USE_MEMORY_DB === 'true') {
    memoryStore = new MemoryStore();
    repo = new MemoryRepository(memoryStore);
  } else {
    repo = new MongoRepository();
  }
  return repo;
}

export function resetRepository() {
  repo = null;
  memoryStore = null;
}
