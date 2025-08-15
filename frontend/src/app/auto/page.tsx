'use client';

import { useEffect, useState } from 'react';
import { api } from '@/lib/api';

type Wallet = { id: string };
type Rule = { id: string; mint: string; minSentiment: number; maxTradeSol: number; active: boolean };

export default function AutoPage() {
  const [wallets, setWallets] = useState<Wallet[]>([]);
  const [rules, setRules] = useState<Rule[]>([]);
  const [walletId, setWalletId] = useState('');
  const [mint, setMint] = useState('PumpFunDemoMint1111111111111111111111111111');
  const [minScore, setMinScore] = useState('70');
  const [maxSol, setMaxSol] = useState('0.1');

  useEffect(() => {
    api<{ wallets: Wallet[] }>('/wallets').then((d) => {
      setWallets(d.wallets);
      if (d.wallets[0]) setWalletId(d.wallets[0].id);
    });
    api<{ rules: Rule[] }>('/auto/rules').then((d) => setRules(d.rules));
  }, []);

  async function createRule() {
    await api('/auto/rules', {
      method: 'POST',
      body: JSON.stringify({
        walletId,
        mint,
        minSentiment: Number(minScore),
        maxTradeSol: Number(maxSol),
      }),
    });
    const d = await api<{ rules: Rule[] }>('/auto/rules');
    setRules(d.rules);
  }

  return (
    <div className="mx-auto max-w-lg space-y-6">
      <h1 className="text-3xl font-extrabold">Auto Trading</h1>
      <p className="text-zinc-400">Buy when sentiment score crosses your threshold.</p>

      <div className="card space-y-4 p-6">
        <select className="w-full rounded-lg border border-edge bg-void px-3 py-2" value={walletId} onChange={(e) => setWalletId(e.target.value)}>
          {wallets.map((w) => <option key={w.id} value={w.id}>{w.id.slice(0, 8)}…</option>)}
        </select>
        <input className="w-full rounded-lg border border-edge bg-void px-3 py-2 font-mono text-sm" value={mint} onChange={(e) => setMint(e.target.value)} />
        <div className="grid grid-cols-2 gap-3">
          <input className="rounded-lg border border-edge bg-void px-3 py-2 font-mono" value={minScore} onChange={(e) => setMinScore(e.target.value)} placeholder="Min score" />
          <input className="rounded-lg border border-edge bg-void px-3 py-2 font-mono" value={maxSol} onChange={(e) => setMaxSol(e.target.value)} placeholder="Max SOL" />
        </div>
        <button className="btn-primary w-full" onClick={createRule}>Create rule</button>
      </div>

      <ul className="space-y-2">
        {rules.map((r) => (
          <li key={r.id} className="card p-4 text-sm">
            <span className="font-mono text-mint">score ≥ {r.minSentiment}</span>
            <span className="text-zinc-500"> · max {r.maxTradeSol} SOL</span>
          </li>
        ))}
      </ul>
    </div>
  );
}
