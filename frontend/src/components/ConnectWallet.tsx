'use client';

import { useWallet } from '@solana/wallet-adapter-react';
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import { useEffect } from 'react';
import { api } from '@/lib/api';

export function ConnectWallet() {
  const { publicKey, connected } = useWallet();

  useEffect(() => {
    if (!connected || !publicKey) return;
    const pk = publicKey.toBase58();
    api('/wallets/link', {
      method: 'POST',
      body: JSON.stringify({ publicKey: pk, label: 'Connected Wallet' }),
    }).catch(() => {});
  }, [connected, publicKey]);

  return <WalletMultiButton className="!bg-sol !rounded-lg !h-9 !text-sm" />;
}
