import { Connection, Keypair, LAMPORTS_PER_SOL, PublicKey, VersionedTransaction, clusterApiUrl } from '@solana/web3.js';
import bs58 from 'bs58';
import crypto from 'crypto';
import { logger } from '../lib/logger.js';

const SOL_MINT = 'So11111111111111111111111111111111111111112';

export function createSolanaClient() {
  const paper = process.env.PAPER_TRADING !== 'false';
  const cluster = process.env.SOLANA_CLUSTER || 'devnet';
  const rpcUrl = process.env.SOLANA_RPC_URL || clusterApiUrl(cluster);
  const connection = new Connection(rpcUrl, 'confirmed');

  const paperState = {
    balances: new Map(),
    defaultSol: 10,
  };

  function getPaperBalance(pubkey) {
    if (!paperState.balances.has(pubkey)) {
      paperState.balances.set(pubkey, paperState.defaultSol);
    }
    return paperState.balances.get(pubkey);
  }

  return {
    mode: paper ? 'paper' : cluster,
    cluster,
    rpcUrl,
    SOL_MINT,

    getConnection() {
      return connection;
    },

    isPaper() {
      return paper;
    },

    async getBalanceSol(publicKey) {
      const pk = typeof publicKey === 'string' ? publicKey : publicKey.toBase58();
      if (paper) return getPaperBalance(pk);
      const bal = await connection.getBalance(new PublicKey(pk));
      return bal / LAMPORTS_PER_SOL;
    },

    generateWallet() {
      const kp = Keypair.generate();
      return {
        publicKey: kp.publicKey.toBase58(),
        secretKey: bs58.encode(kp.secretKey),
      };
    },

    encryptSecret(secretKey, passphrase = process.env.JWT_SECRET || 'dev') {
      const iv = crypto.randomBytes(16);
      const key = crypto.scryptSync(passphrase, 'salt', 32);
      const cipher = crypto.createCipheriv('aes-256-gcm', key, iv);
      const enc = Buffer.concat([cipher.update(secretKey, 'utf8'), cipher.final()]);
      const tag = cipher.getAuthTag();
      return `${iv.toString('hex')}:${tag.toString('hex')}:${enc.toString('hex')}`;
    },

    async getRecentSignatures(address, limit = 10) {
      const pubkey = new PublicKey(address);
      return connection.getSignaturesForAddress(pubkey, { limit });
    },

    async sendRawTransaction(serializedTxBase64) {
      const buf = Buffer.from(serializedTxBase64, 'base64');
      const tx = VersionedTransaction.deserialize(buf);
      const sig = await connection.sendRawTransaction(tx.serialize(), {
        skipPreflight: false,
        maxRetries: 3,
      });
      await connection.confirmTransaction(sig, 'confirmed');
      logger.info('tx_confirmed', { sig });
      return sig;
    },

    async paperSwap(walletPubkey, amountSol, side) {
      const bal = getPaperBalance(walletPubkey);
      if (side === 'buy' && bal < amountSol) throw new Error('insufficient_sol');
      if (side === 'buy') paperState.balances.set(walletPubkey, bal - amountSol);
      else paperState.balances.set(walletPubkey, bal + amountSol);
      const sig = `paper-${Date.now()}-${crypto.randomBytes(4).toString('hex')}`;
      logger.info('paper_swap', { wallet: walletPubkey, side, amountSol, sig });
      return sig;
    },
  };
}
