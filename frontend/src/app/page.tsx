'use client';

import { useEffect, useState } from 'react';
import { api } from '@/lib/api';
import { io } from 'socket.io-client';

type Token = { mint: string; symbol: string; priceUsd: number; marketCap: number; bondingCurvePct: number };
type Sentiment = { mint: string; symbol: string; score: number; mentions: number; velocity: number };

export default function DashboardPage() {
  const [tokens, setTokens] = useState<Token[]>([]);
  const [scores, setScores] = useState<Sentiment[]>([]);
  const [trades, setTrades] = useState<unknown[]>([]);

  useEffect(() => {
    api<{ tokens: Token[] }>('/tokens/trending').then((d) => setTokens(d.tokens)).catch(console.error);
    api<{ scores: Sentiment[] }>('/sentiment').then((d) => setScores(d.scores)).catch(console.error);
    api<{ trades: unknown[] }>('/trades').then((d) => setTrades(d.trades)).catch(console.error);

    const socket = io(process.env.NEXT_PUBLIC_API_URL || 'http://127.0.0.1:8091');
    socket.on('sentiment:update', (row: Sentiment) => {
      setScores((prev) => {
        const next = prev.filter((s) => s.mint !== row.mint);
        return [...next, row].sort((a, b) => b.score - a.score).slice(0, 10);
      });
    });
    return () => { socket.disconnect(); };
  }, []);

  return (
    <div className="space-y-8">
      <section>
        <p className="font-mono text-xs uppercase tracking-widest text-mint">Solana DEX Terminal</p>
        <h1 className="mt-2 text-4xl font-extrabold tracking-tight md:text-5xl">
          Trade smarter with <span className="text-sol">sentiment</span>
        </h1>
        <p className="mt-3 max-w-2xl text-zinc-400">
          Real-time Pump.fun tokens, social sentiment scores, one-click execution, copy trading, and rule-based automation.
        </p>
      </section>

      <div className="grid gap-4 md:grid-cols-3">
        {[
          { label: 'Trending tokens', value: tokens.length },
          { label: 'Sentiment feeds', value: scores.length },
          { label: 'Your trades', value: trades.length },
        ].map((s) => (
          <div key={s.label} className="card p-5">
            <p className="text-sm text-zinc-500">{s.label}</p>
            <p className="mt-1 font-mono text-3xl font-semibold text-mint">{s.value}</p>
          </div>
        ))}
      </div>

      <div className="grid gap-6 lg:grid-cols-2">
        <div className="card p-5">
          <h2 className="text-lg font-bold">🔥 Pump.fun Trending</h2>
          <ul className="mt-4 space-y-3">
            {tokens.slice(0, 5).map((t) => (
              <li key={t.mint} className="flex items-center justify-between border-b border-edge pb-2 last:border-0">
                <div>
                  <span className="font-semibold">{t.symbol}</span>
                  <span className="ml-2 font-mono text-xs text-zinc-500">${t.priceUsd}</span>
                </div>
                <span className="font-mono text-xs text-warn">{t.bondingCurvePct}% curve</span>
              </li>
            ))}
          </ul>
        </div>

        <div className="card p-5">
          <h2 className="text-lg font-bold">📡 Social Sentiment</h2>
          <ul className="mt-4 space-y-4">
            {scores.slice(0, 5).map((s) => (
              <li key={s.mint}>
                <div className="flex justify-between text-sm">
                  <span className="font-semibold">{s.symbol}</span>
                  <span className="font-mono text-mint">{s.score}/100</span>
                </div>
                <div className="score-bar mt-1">
                  <div className="score-fill" style={{ width: `${s.score}%` }} />
                </div>
                <p className="mt-1 font-mono text-xs text-zinc-500">
                  {s.mentions} mentions · velocity {s.velocity}
                </p>
              </li>
            ))}
          </ul>
        </div>
      </div>
    </div>
  );
}
