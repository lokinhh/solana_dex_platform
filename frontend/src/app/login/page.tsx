'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { authApi, setToken } from '@/lib/api';

export default function LoginPage() {
  const router = useRouter();
  const [email, setEmail] = useState('trader@demo.com');
  const [password, setPassword] = useState('password123');
  const [mode, setMode] = useState<'login' | 'register'>('login');
  const [error, setError] = useState('');

  async function submit(e: React.FormEvent) {
    e.preventDefault();
    setError('');
    try {
      const path = mode === 'login' ? '/login' : '/register';
      const data = await authApi<{ token: string }>(path, { email, password, name: 'Trader' });
      setToken(data.token);
      router.push('/');
    } catch (err) {
      setError(String(err));
    }
  }

  return (
    <div className="mx-auto max-w-md">
      <h1 className="text-3xl font-extrabold">{mode === 'login' ? 'Sign in' : 'Create account'}</h1>
      <form onSubmit={submit} className="card mt-6 space-y-4 p-6">
        <input
          className="w-full rounded-lg border border-edge bg-void px-3 py-2"
          type="email"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
          placeholder="Email"
        />
        <input
          className="w-full rounded-lg border border-edge bg-void px-3 py-2"
          type="password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          placeholder="Password (8+ chars)"
        />
        {error && <p className="text-sm text-red-400">{error}</p>}
        <button type="submit" className="btn-primary w-full">
          {mode === 'login' ? 'Sign in' : 'Register'}
        </button>
        <button
          type="button"
          className="w-full text-sm text-zinc-500 hover:text-mint"
          onClick={() => setMode(mode === 'login' ? 'register' : 'login')}
        >
          {mode === 'login' ? 'Need an account? Register' : 'Have an account? Sign in'}
        </button>
      </form>
    </div>
  );
}
