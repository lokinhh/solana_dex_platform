import 'dotenv/config';
import express from 'express';
import cors from 'cors';
import { createServer } from 'http';
import { Server } from 'socket.io';
import { connectDb } from './lib/db.js';
import { logger } from './lib/logger.js';
import { createRepository } from './services/repository.js';
import { createSolanaClient } from './services/solanaClient.js';
import { createPumpfunService } from './services/pumpfunService.js';
import { createJupiterService } from './services/jupiterService.js';
import { createTradeExecutor } from './services/tradeExecutor.js';
import { SentimentEngine } from './services/sentimentEngine.js';
import { CopyTradeEngine } from './services/copyTradeEngine.js';
import { AutoTradeEngine } from './services/autoTradeEngine.js';
import { OnchainWatcher } from './services/onchainWatcher.js';
import { createApiRouter, createWebhookRouter } from './routes/api.js';
import { createAuthRouter } from './routes/auth.js';

const PORT = Number(process.env.PORT || 8091);
const HOST = process.env.HOST || '0.0.0.0';

await connectDb();
const repo = createRepository();
const solana = createSolanaClient();
const pumpfun = createPumpfunService();
const jupiter = createJupiterService(solana);
const executor = createTradeExecutor({ repo, solana, jupiter, pumpfun });

const app = express();
const httpServer = createServer(app);
const io = new Server(httpServer, {
  cors: { origin: process.env.CORS_ORIGIN || '*' },
});

const sentiment = new SentimentEngine({ repo, pumpfun, io });
const copyTrade = new CopyTradeEngine({ repo, executor, io });
const autoTrade = new AutoTradeEngine({ repo, executor, sentiment, io });
const onchainWatcher = new OnchainWatcher({ repo, solana, copyTrade });

const deps = {
  repo,
  solana,
  pumpfun,
  jupiter,
  executor,
  sentiment,
  copyTrade,
  autoTrade,
  onchainWatcher,
  io,
};

app.disable('x-powered-by');
app.use(cors({ origin: process.env.CORS_ORIGIN || '*' }));
app.use(express.json({ limit: '512kb' }));

app.get('/health', async (_req, res) => {
  const trades = process.env.USE_MEMORY_DB === 'true'
    ? (await repo.listAllTrades(5))
  : await repo.listAllTrades(5);
  res.json({
    ok: true,
    mode: solana.mode,
    cluster: solana.cluster,
    db: process.env.USE_MEMORY_DB === 'true' ? 'memory' : 'mongodb',
    trades: trades.length,
    sentiment: await sentiment.listTop(3),
    uptimeSec: Math.floor(process.uptime()),
  });
});

app.use('/api/v1/auth', createAuthRouter(repo));
app.use('/api/v1', createApiRouter(deps));
app.use('/webhooks', createWebhookRouter(deps));

io.on('connection', async (socket) => {
  socket.emit('sentiment:snapshot', await sentiment.listTop());
  socket.on('disconnect', () => {});
});

sentiment.start();
copyTrade.start();
autoTrade.start();
onchainWatcher.start();

httpServer.listen(PORT, HOST, () => {
  logger.info('api_started', {
    host: HOST,
    port: PORT,
    mode: solana.mode,
    db: process.env.USE_MEMORY_DB === 'true' ? 'memory' : 'mongodb',
  });
});

function shutdown(sig) {
  logger.info('shutdown', { sig });
  sentiment.stop();
  copyTrade.stop();
  autoTrade.stop();
  onchainWatcher.stop();
  httpServer.close(() => process.exit(0));
}
process.on('SIGTERM', () => shutdown('SIGTERM'));
process.on('SIGINT', () => shutdown('SIGINT'));

export { app, deps, httpServer };
