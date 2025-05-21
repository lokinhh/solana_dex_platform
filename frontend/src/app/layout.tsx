import type { Metadata } from 'next';
import './globals.css';
import { Nav } from '@/components/Nav';
import { SolanaProviders } from '@/components/SolanaProviders';

export const metadata: Metadata = {
  title: 'SolDex — Solana DEX Terminal',
  description: 'Pump.fun trading with social sentiment, copy trade, and auto execution',
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body className="font-display">
        <SolanaProviders>
          <Nav />
          <main className="mx-auto max-w-6xl px-4 py-8">{children}</main>
        </SolanaProviders>
      </body>
    </html>
  );
}
