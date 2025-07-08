'use client';

import { useEffect, useState } from 'react';
import { useWallet } from '@solana/wallet-adapter-react';
import { api } from '@/lib/api';

type Wallet = { id: string; publicKey: string; label: string };
type Token = { mint: string; symbol: string; priceUsd: number };

export default function TradePage() {
  const { publicKey, connected } = useWallet();
  const [wallets, setWallets] = useState<Wallet[]>([]);
  const [tokens, setTokens] = useState<Token[]>([]);
  const [walletId, setWalletId] = useState('');
  const [mint, setMint] = useState('');
  const [amount, setAmount] = useState('0.1');
  const [status, setStatus] = useState('');
  const paper = process.env.NEXT_PUBLIC_PAPER_TRADING !== 'false';

  useEffect(() => {
    api<{ wallets: Wallet[] }>('/wallets').then((d) => {
      setWallets(d.wallets);
      if (d.wallets[0]) setWalletId(d.wallets[0].id);
    }).catch(() => {
      api<{ wallet: Wallet }>('/wallets', { method: 'POST', body: '{}' }).then((d) => {
        setWallets([d.wallet]);
        setWalletId(d.wallet.id);
      });
    });
    api<{ tokens: Token[] }>('/tokens/trending').then((d) => {
      const t = d.tokens.filter((x) => x.symbol !== 'SOL');
      setTokens(t);
      if (t[0]) setMint(t[0].mint);
    });
  }, []);

  const activePubkey = connected && publicKey ? publicKey.toBase58() : wallets.find((w) => w.id === walletId)?.publicKey;

  async function trade(side: 'buy' | 'sell') {
    if (!activePubkey) {
      setStatus('Connect Phantom or create a platform wallet');
      return;
    }
    setStatus('Executing…');
    try {
      const token = tokens.find((t) => t.mint === mint);
      const amountSol = parseFloat(amount);

      if (paper) {
        await api('/trade', {
          method: 'POST',
          body: JSON.stringify({ walletId, mint, symbol: token?.symbol, side, amountSol }),
        });
        setStatus(`${side.toUpperCase()} confirmed (paper) ✓`);
        return;
      }

      const { quote } = await api<{ quote: object }>('/trade/quote', {
        method: 'POST',
        body: JSON.stringify({ mint, side, amountSol, walletPublicKey: activePubkey }),
      });

      const { swapTransaction } = await api<{ swapTransaction: string }>('/trade/build', {
        method: 'POST',
        body: JSON.stringify({ quote, walletPublicKey: activePubkey }),
      });

      setStatus('Sign transaction in Phantom…');
      // Live: deserialize, sign with wallet, submit
      await api('/trade/submit', {
        method: 'POST',
        headers: { 'Idempotency-Key': `${mint}:${side}:${Date.now()}` },
        body: JSON.stringify({
          mint,
          symbol: token?.symbol,
          side,
          amountSol,
          walletPublicKey: activePubkey,
          walletId,
          signedTransaction: swapTransaction,
        }),
      });
      setStatus(`${side.toUpperCase()} submitted on-chain ✓`);
    } catch (e) {
      setStatus(String(e));
    }
  }

  return (
    <div className="mx-auto max-w-lg space-y-6">
      <h1 className="text-3xl font-extrabold">One-click Trade</h1>
      <p className="text-sm text-zinc-500">
        Mode: <span className="text-mint">{paper ? 'paper' : 'devnet/live'}</span>
        {activePubkey && <> · <span className="font-mono text-xs">{activePubkey.slice(0, 8)}…</span></>}
      </p>

      <div className="card space-y-4 p-6">
        {!connected && (
          <select className="w-full rounded-lg border border-edge bg-void px-3 py-2" value={walletId} onChange={(e) => setWalletId(e.target.value)}>
            {wallets.map((w) => (
              <option key={w.id} value={w.id}>{w.label} ({w.publicKey.slice(0, 8)}…)</option>
            ))}
          </select>
        )}

        <select className="w-full rounded-lg border border-edge bg-void px-3 py-2" value={mint} onChange={(e) => setMint(e.target.value)}>
          {tokens.map((t) => (
            <option key={t.mint} value={t.mint}>{t.symbol} — ${t.priceUsd}</option>
          ))}
        </select>

        <input className="w-full rounded-lg border border-edge bg-void px-3 py-2 font-mono" value={amount} onChange={(e) => setAmount(e.target.value)} placeholder="Amount SOL" />

        <div className="flex gap-3 pt-2">
          <button className="btn-mint flex-1" onClick={() => trade('buy')}>Buy</button>
          <button className="btn-primary flex-1" onClick={() => trade('sell')}>Sell</button>
        </div>

        {status && <p className="font-mono text-sm text-mint">{status}</p>}
      </div>
    </div>
  );
}
