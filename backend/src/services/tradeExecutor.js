import { logger } from '../lib/logger.js';

export function createTradeExecutor({ repo, solana, jupiter, pumpfun }) {
  const maxTradeSol = Number(process.env.MAX_TRADE_SOL || 1);

  return {
    async quote({ side, mint, amountSol }) {
      if (amountSol <= 0 || amountSol > maxTradeSol) throw new Error('invalid_amount');
      const amountLamports = Math.floor(amountSol * 1e9);
      const quote = await jupiter.getQuote({ side, mint, amountLamports });
      const token = await pumpfun.getToken(mint);
      return { quote, token, amountSol };
    },

    async build({ quote, walletPublicKey }) {
      return jupiter.buildSwapTransaction(quote, walletPublicKey);
    },

    async submit({
      userId,
      walletId,
      walletPublicKey,
      mint,
      symbol,
      side,
      amountSol,
      source = 'manual',
      leaderWallet,
      signedTransaction,
      idempotencyKey,
    }) {
      if (idempotencyKey) {
        const cached = await repo.getIdempotency(idempotencyKey);
        if (cached) return { ...cached, duplicate: true };
      }

      let signature;
      if (solana.isPaper()) {
        signature = await jupiter.executePaperSwap(walletPublicKey, amountSol, side);
      } else if (signedTransaction) {
        const result = await jupiter.submitSignedTransaction(signedTransaction);
        signature = result.signature;
      } else {
        throw new Error('signed_transaction_required');
      }

      const token = await pumpfun.getToken(mint);
      const trade = await repo.addTrade({
        userId,
        walletId,
        walletPublicKey,
        mint,
        symbol: symbol || token?.symbol || 'UNKNOWN',
        side,
        amountSol,
        tokenAmount: 0,
        priceUsd: token?.priceUsd || 0,
        mode: solana.mode,
        source,
        leaderWallet,
        txSignature: signature,
        status: 'confirmed',
        idempotencyKey,
      });

      if (idempotencyKey) await repo.setIdempotency(idempotencyKey, trade);

      logger.info('trade_executed', { side, mint, amountSol, source, mode: solana.mode, signature });
      return trade;
    },

    /** Legacy one-shot paper execute */
    async execute(params) {
      const wallet = await repo.findWallet(params.userId, params.walletId);
      if (!wallet) throw new Error('wallet_not_found');
      const balance = await solana.getBalanceSol(wallet.publicKey);
      if (params.side === 'buy' && balance < params.amountSol) throw new Error('insufficient_sol');

      if (!solana.isPaper()) {
        throw new Error('use_quote_build_submit_for_live');
      }

      return this.submit({
        ...params,
        walletPublicKey: wallet.publicKey,
        signedTransaction: 'paper',
      });
    },
  };
}
