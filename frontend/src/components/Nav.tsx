'use client';

import Link from 'next/link';
import { usePathname, useRouter } from 'next/navigation';
import { ConnectWallet } from './ConnectWallet';
import { clearToken, getToken } from '@/lib/api';

const links = [
  { href: '/', label: 'Dashboard' },
  { href: '/trade', label: 'Trade' },
  { href: '/sentiment', label: 'Sentiment' },
  { href: '/copy', label: 'Copy' },
  { href: '/auto', label: 'Auto' },
];

export function Nav() {
  const path = usePathname();
  const router = useRouter();
  const loggedIn = typeof window !== 'undefined' && !!getToken();

  return (
    <header className="border-b border-edge/80 bg-panel/60 backdrop-blur-md sticky top-0 z-50">
      <div className="mx-auto flex max-w-6xl items-center justify-between gap-4 px-4 py-3">
        <Link href="/" className="font-display text-xl font-extrabold tracking-tight shrink-0">
          <span className="text-mint">◆</span> SolDex
        </Link>
        <nav className="flex gap-1 text-sm overflow-x-auto">
          {links.map((l) => (
            <Link
              key={l.href}
              href={l.href}
              className={`rounded-lg px-3 py-1.5 whitespace-nowrap transition ${
                path === l.href ? 'bg-sol/20 text-mint' : 'text-zinc-400 hover:text-white'
              }`}
            >
              {l.label}
            </Link>
          ))}
        </nav>
        <div className="flex items-center gap-2 shrink-0">
          <ConnectWallet />
          {loggedIn ? (
            <button
              className="text-xs text-zinc-500 hover:text-white"
              onClick={() => { clearToken(); router.push('/login'); }}
            >
              Logout
            </button>
          ) : (
            <Link href="/login" className="text-xs text-mint hover:underline">Login</Link>
          )}
        </div>
      </div>
    </header>
  );
}
