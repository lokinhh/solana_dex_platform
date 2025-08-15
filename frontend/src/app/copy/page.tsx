'use client';

import { useEffect, useState } from 'react';
import { api } from '@/lib/api';

type Wallet = { id: string; publicKey: string };
type Sub = { id: string; leaderAddress: string; sizePct: number; active: boolean };

export default function CopyPage() {
  const [wallets, setWallets] = useState<Wallet[]>([]);
  const [subs, setSubs] = useState<Sub[]>([]);
  const [leader, setLeader] = useState('LeaderWallet1111111111111111111111111111');
  const [walletId, setWalletId] = useState('');
  const [sizePct, setSizePct] = useState('100');

  useEffect(() => {
    api<{ wallets: Wallet[] }>('/wallets').then((d) => {
      setWallets(d.wallets);
      if (d.wallets[0]) setWalletId(d.wallets[0].id);
    });
    api<{ subscriptions: Sub[] }>('/copy/subscriptions').then((d) => setSubs(d.subscriptions));
  }, []);

  async function subscribe() {
    await api('/copy/subscribe', {
      method: 'POST',
      body: JSON.stringify({ leaderAddress: leader, followerWalletId: walletId, sizePct: Number(sizePct) }),
    });
    const d = await api<{ subscriptions: Sub[] }>('/copy/subscriptions');
    setSubs(d.subscriptions);
  }

  async function simulate() {
    await api('/copy/simulate', {
      method: 'POST',
      body: JSON.stringify({
        leaderAddress: leader,
        mint: 'PumpFunDemoMint1111111111111111111111111111',
        symbol: 'PEPE2',
        side: 'buy',
        amountSol: 0.05,
      }),
    });
    alert('Leader trade simulated — check trades on dashboard');
  }

  return (
    <div className="mx-auto max-w-lg space-y-6">
      <h1 className="text-3xl font-extrabold">Copy Trading</h1>
      <p className="text-zinc-400">Mirror leader wallet trades with configurable position sizing.</p>

      <div className="card space-y-4 p-6">
        <input className="w-full rounded-lg border border-edge bg-void px-3 py-2 font-mono text-sm" value={leader} onChange={(e) => setLeader(e.target.value)} placeholder="Leader wallet address" />
        <select className="w-full rounded-lg border border-edge bg-void px-3 py-2" value={walletId} onChange={(e) => setWalletId(e.target.value)}>
          {wallets.map((w) => <option key={w.id} value={w.id}>{w.publicKey.slice(0, 12)}…</option>)}
        </select>
        <input className="w-full rounded-lg border border-edge bg-void px-3 py-2 font-mono" value={sizePct} onChange={(e) => setSizePct(e.target.value)} placeholder="Size %" />
        <button className="btn-primary w-full" onClick={subscribe}>Subscribe</button>
        <button className="btn-mint w-full" onClick={simulate}>Simulate leader buy</button>
      </div>

      <ul className="space-y-2">
        {subs.map((s) => (
          <li key={s.id} className="card p-4 font-mono text-sm">
            {s.leaderAddress.slice(0, 16)}… @ {s.sizePct}%
          </li>
        ))}
      </ul>
    </div>
  );
}
