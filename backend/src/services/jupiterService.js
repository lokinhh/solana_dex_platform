import { logger } from '../lib/logger.js';

const SOL_MINT = 'So11111111111111111111111111111111111111112';

export function createJupiterService(solana) {
  const paper = process.env.PAPER_TRADING !== 'false';
  const baseUrl = process.env.JUPITER_API_URL || 'https://quote-api.jup.ag/v6';
  const defaultSlippage = Number(process.env.DEFAULT_SLIPPAGE_BPS || 300);

  async function fetchQuote({ inputMint, outputMint, amount, slippageBps = defaultSlippage }) {
    const params = new URLSearchParams({
      inputMint,
      outputMint,
      amount: String(amount),
      slippageBps: String(slippageBps),
    });
    const res = await fetch(`${baseUrl}/quote?${params}`);
    if (!res.ok) {
      const err = await res.text();
      throw new Error(`jupiter_quote_${res.status}:${err.slice(0, 120)}`);
    }
    return res.json();
  }

  return {
    SOL_MINT,

    resolveMints(side, mint) {
      if (side === 'buy') {
        return { inputMint: SOL_MINT, outputMint: mint };
      }
      return { inputMint: mint, outputMint: SOL_MINT };
    },

    async getQuote({ side, mint, amountLamports, slippageBps }) {
      const { inputMint, outputMint } = this.resolveMints(side, mint);
      if (paper) {
        const outAmount = side === 'buy' ? Math.floor(amountLamports * 1000) : amountLamports;
        return {
          inputMint,
          outputMint,
          inAmount: String(amountLamports),
          outAmount: String(outAmount),
          priceImpactPct: 0.1,
          slippageBps: slippageBps || defaultSlippage,
          mode: 'paper',
        };
      }
      return fetchQuote({ inputMint, outputMint, amount: amountLamports, slippageBps });
    },

    async buildSwapTransaction(quoteResponse, userPublicKey) {
      if (paper) {
        return {
          swapTransaction: Buffer.from(`paper-tx-${Date.now()}`).toString('base64'),
          mode: 'paper',
        };
      }
      const res = await fetch(`${baseUrl}/swap`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          quoteResponse,
          userPublicKey,
          wrapAndUnwrapSol: true,
          dynamicComputeUnitLimit: true,
        }),
      });
      if (!res.ok) throw new Error(`jupiter_swap_${res.status}`);
      const data = await res.json();
      return { swapTransaction: data.swapTransaction, mode: 'live' };
    },

    async executePaperSwap(walletPubkey, amountSol, side) {
      return solana.paperSwap(walletPubkey, amountSol, side);
    },

    async submitSignedTransaction(serializedTxBase64) {
      if (paper) {
        const sig = `jup-paper-${Date.now()}`;
        logger.info('jupiter_paper_submit', { sig });
        return { signature: sig, status: 'confirmed', mode: 'paper' };
      }
      const signature = await solana.sendRawTransaction(serializedTxBase64);
      return { signature, status: 'confirmed', mode: 'live' };
    },
  };
}
