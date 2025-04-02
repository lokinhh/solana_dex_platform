import { logger } from '../lib/logger.js';

/**
 * Poll leader wallets on-chain for new signatures (devnet/mainnet).
 */
export class OnchainWatcher {
  constructor({ repo, solana, copyTrade }) {
    this.repo = repo;
    this.solana = solana;
    this.copyTrade = copyTrade;
    this.timer = null;
  }

  start() {
    if (this.solana.isPaper()) return;
    const ms = Number(process.env.ONCHAIN_POLL_MS || 12000);
    this.timer = setInterval(() => this.poll(), ms);
    logger.info('onchain_watcher_started', { pollMs: ms });
  }

  stop() {
    if (this.timer) clearInterval(this.timer);
  }

  async poll() {
    const subs = await this.repo.listAllCopySubs();
    const leaders = [...new Set(subs.map((s) => s.leaderAddress))];

    for (const leader of leaders) {
      try {
        const sigs = await this.solana.getRecentSignatures(leader, 5);
        if (!sigs.length) continue;

        const latest = sigs[0].signature;
        const last = await this.repo.getLeaderCursor(leader);
        if (last === latest) continue;

        if (last) {
          const newSigs = [];
          for (const s of sigs) {
            if (s.signature === last) break;
            newSigs.push(s.signature);
          }
          for (const sig of newSigs.reverse()) {
            await this.copyTrade.handleLeaderActivity(leader, { signature: sig });
          }
        }

        await this.repo.setLeaderCursor(leader, latest);
      } catch (err) {
        logger.warn('onchain_poll_failed', { leader, err: String(err) });
      }
    }
  }

  /** Helius / manual webhook payload */
  async handleWebhookEvent(event) {
    const leader = event.feePayer || event.leaderAddress || event.account;
    if (!leader) return [];
    return this.copyTrade.handleLeaderActivity(leader, {
      signature: event.signature,
      mint: event.mint,
      symbol: event.symbol,
      side: event.side || 'buy',
      amountSol: event.amountSol || 0.05,
    });
  }
}
