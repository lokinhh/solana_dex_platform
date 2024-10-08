import crypto from 'crypto';
import { verifyToken } from '../lib/jwt.js';

export function timingSafeEqual(a, b) {
  const bufA = Buffer.from(String(a));
  const bufB = Buffer.from(String(b));
  if (bufA.length !== bufB.length) return false;
  return crypto.timingSafeEqual(bufA, bufB);
}

/** JWT (users) or API key (E2E / service). */
export function requireAuth(req, res, next) {
  const header = req.headers.authorization || '';
  const bearer = header.startsWith('Bearer ') ? header.slice(7).trim() : '';

  if (bearer && !bearer.startsWith('sk_')) {
    const user = verifyToken(bearer);
    if (!user?.uid) return res.status(401).json({ error: 'invalid_token' });
    req.userId = user.uid;
    req.userEmail = user.email;
    return next();
  }

  const apiKey = req.headers['x-api-key'] || bearer;
  const expected = process.env.API_SECRET;
  if (expected && apiKey && timingSafeEqual(apiKey, expected)) {
    req.userId = req.headers['x-user-id'] || 'service-user';
    req.authMode = 'api_key';
    return next();
  }

  return res.status(401).json({ error: 'unauthorized' });
}

export function verifyWebhookSecret(req, res, next) {
  const expected = process.env.HELIUS_WEBHOOK_SECRET || process.env.API_SECRET;
  const got = req.headers['authorization'] || req.headers['x-webhook-secret'] || '';
  const token = got.startsWith('Bearer ') ? got.slice(7) : got;
  if (!expected || !token || !timingSafeEqual(token, expected)) {
    return res.status(401).json({ error: 'invalid_webhook_secret' });
  }
  next();
}
