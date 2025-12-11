/**
 * Platform unit tests
 */
import test from 'node:test';
import assert from 'node:assert/strict';
import { MemoryStore } from '../backend/src/services/memoryStore.js';
import { MemoryRepository } from '../backend/src/services/repository.js';
import { createSolanaClient } from '../backend/src/services/solanaClient.js';
import { createPumpfunService } from '../backend/src/services/pumpfunService.js';
import { createJupiterService } from '../backend/src/services/jupiterService.js';
import { createTradeExecutor } from '../backend/src/services/tradeExecutor.js';
import { SentimentEngine } from '../backend/src/services/sentimentEngine.js';
import { hashPassword, verifyPassword, signToken, verifyToken } from '../backend/src/lib/jwt.js';

process.env.PAPER_TRADING = 'true';
process.env.JWT_SECRET = 'test-secret-key-32-characters!!';

function makeRepo() {
  return new MemoryRepository(new MemoryStore());
}

test('JWT sign and verify', () => {
  const token = signToken({ uid: 'u1', email: 'a@b.com' });
  const payload = verifyToken(token);
  assert.equal(payload.uid, 'u1');
});

test('password hash verifies', () => {
  const h = hashPassword('password123');
  assert.equal(verifyPassword('password123', h), true);
  assert.equal(verifyPassword('wrong', h), false);
});

test('repository creates user and wallet', async () => {
  const repo = makeRepo();
  const user = await repo.createUser({ email: 't@t.com', passwordHash: 'x', name: 'T' });
  const w = await repo.createWallet({ userId: user.id, publicKey: 'pk1', label: 'W' });
  assert.equal((await repo.listWallets(user.id)).length, 1);
  assert.equal(w.publicKey, 'pk1');
});

test('trade executor paper buy', async () => {
  const repo = makeRepo();
  const sol = createSolanaClient();
  const pf = createPumpfunService();
  const jup = createJupiterService(sol);
  const executor = createTradeExecutor({ repo, solana: sol, jupiter: jup, pumpfun: pf });

  const user = await repo.createUser({ email: 'x@x.com', passwordHash: 'h' });
  const w = await repo.createWallet({
    userId: user.id,
    publicKey: sol.generateWallet().publicKey,
    label: 'test',
  });

  const trade = await executor.execute({
    userId: user.id,
    walletId: w.id,
    mint: 'PumpFunDemoMint1111111111111111111111111111',
    symbol: 'PEPE2',
    side: 'buy',
    amountSol: 0.1,
  });

  assert.equal(trade.side, 'buy');
  assert.equal((await repo.listTrades(user.id)).length, 1);
});

test('trade quote build submit flow', async () => {
  const repo = makeRepo();
  const sol = createSolanaClient();
  const jup = createJupiterService(sol);
  const pf = createPumpfunService();
  const executor = createTradeExecutor({ repo, solana: sol, jupiter: jup, pumpfun: pf });
  const user = await repo.createUser({ email: 'q@q.com', passwordHash: 'h' });
  const pk = sol.generateWallet().publicKey;

  const { quote } = await executor.quote({
    side: 'buy',
    mint: 'PumpFunDemoMint1111111111111111111111111111',
    amountSol: 0.05,
  });
  const built = await executor.build({ quote, walletPublicKey: pk });
  assert.ok(built.swapTransaction);

  const trade = await executor.submit({
    userId: user.id,
    walletPublicKey: pk,
    mint: 'PumpFunDemoMint1111111111111111111111111111',
    symbol: 'PEPE2',
    side: 'buy',
    amountSol: 0.05,
    signedTransaction: built.swapTransaction,
    idempotencyKey: 'test-idem-1',
  });
  assert.equal(trade.status, 'confirmed');
});

test('sentiment engine scores tokens', async () => {
  const repo = makeRepo();
  const pf = createPumpfunService();
  const engine = new SentimentEngine({ repo, pumpfun: pf, io: null });
  await engine.tick();
  const scores = await engine.listTop(5);
  assert.ok(scores.length > 0);
});
