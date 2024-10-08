import { Router } from 'express';
import { z } from 'zod';
import { hashPassword, verifyPassword, signToken } from '../lib/jwt.js';

const registerSchema = z.object({
  email: z.string().email(),
  password: z.string().min(8),
  name: z.string().optional(),
});

const loginSchema = z.object({
  email: z.string().email(),
  password: z.string().min(1),
});

export function createAuthRouter(repo) {
  const router = Router();

  router.post('/register', async (req, res) => {
    try {
      const body = registerSchema.parse(req.body);
      const existing = await repo.findUserByEmail(body.email);
      if (existing) return res.status(409).json({ error: 'email_taken' });

      const user = await repo.createUser({
        email: body.email,
        passwordHash: hashPassword(body.password),
        name: body.name || body.email.split('@')[0],
      });

      const token = signToken({ uid: user.id, email: user.email });
      res.status(201).json({
        ok: true,
        token,
        user: { id: user.id, email: user.email, name: user.name },
      });
    } catch (err) {
      res.status(400).json({ error: err.message });
    }
  });

  router.post('/login', async (req, res) => {
    try {
      const body = loginSchema.parse(req.body);
      const user = await repo.findUserByEmail(body.email);
      if (!user || !verifyPassword(body.password, user.passwordHash)) {
        return res.status(401).json({ error: 'invalid_credentials' });
      }
      const token = signToken({ uid: user.id, email: user.email });
      res.json({
        ok: true,
        token,
        user: { id: user.id, email: user.email, name: user.name },
      });
    } catch (err) {
      res.status(400).json({ error: err.message });
    }
  });

  return router;
}
