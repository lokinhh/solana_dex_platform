import mongoose from 'mongoose';
import { logger } from './logger.js';

const connected = { value: false };

export async function connectDb() {
  if (process.env.USE_MEMORY_DB === 'true') {
    logger.info('db_connected', { mode: 'in-memory' });
    return null;
  }
  const uri = process.env.MONGODB_URI || 'mongodb://127.0.0.1:27017/solana_dex';
  await mongoose.connect(uri);
  connected.value = true;
  logger.info('db_connected', { mode: 'mongodb' });
  return mongoose.connection;
}

export function useMongo() {
  return connected.value && mongoose.connection.readyState === 1;
}

export async function disconnectDb() {
  if (mongoose.connection.readyState) await mongoose.disconnect();
}
