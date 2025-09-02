'use client';

import { useEffect, useState } from 'react';
import { api } from '@/lib/api';

type Score = { mint: string; symbol: string; score: number; mentions: number; velocity: number; sources?: { twitter: number; telegram: number } };

export default function SentimentPage() {
  const [scores, setScores] = useState<Score[]>([]);

  useEffect(() => {
    api<{ scores: Score[] }>('/sentiment').then((d) => setScores(d.scores));
  }, []);

  return (
    <div className="space-y-6">
      <h1 className="text-3xl font-extrabold">Social Sentiment</h1>
      <p className="text-zinc-400">Aggregated Twitter + Telegram signals scored per token mint.</p>

      <div className="grid gap-4 md:grid-cols-2">
        {scores.map((s) => (
          <div key={s.mint} className="card p-5">
            <div className="flex items-start justify-between">
              <div>
                <h2 className="text-xl font-bold">{s.symbol}</h2>
                <p className="font-mono text-xs text-zinc-500">{s.mint.slice(0, 20)}…</p>
              </div>
              <span className={`font-mono text-2xl font-bold ${s.score >= 70 ? 'text-mint' : 'text-zinc-400'}`}>
                {s.score}
              </span>
            </div>
            <div className="score-bar mt-4">
              <div className="score-fill" style={{ width: `${s.score}%` }} />
            </div>
            <div className="mt-3 flex gap-4 font-mono text-xs text-zinc-500">
              <span>🐦 {s.sources?.twitter ?? 0}</span>
              <span>✈️ {s.sources?.telegram ?? 0}</span>
              <span>⚡ v{s.velocity}</span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
