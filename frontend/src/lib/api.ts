const API = process.env.NEXT_PUBLIC_API_URL || 'http://127.0.0.1:8091';

export function getToken(): string | null {
  if (typeof window === 'undefined') return null;
  return localStorage.getItem('soldex_token');
}

export function setToken(token: string) {
  localStorage.setItem('soldex_token', token);
}

export function clearToken() {
  localStorage.removeItem('soldex_token');
}

function headers(extra: Record<string, string> = {}) {
  const h: Record<string, string> = { 'Content-Type': 'application/json', ...extra };
  const token = getToken();
  if (token) h.Authorization = `Bearer ${token}`;
  else if (process.env.NEXT_PUBLIC_API_SECRET) {
    h['X-API-Key'] = process.env.NEXT_PUBLIC_API_SECRET;
    h['X-User-Id'] = 'demo-user';
  }
  return h;
}

export async function api<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${API}/api/v1${path}`, {
    ...init,
    headers: { ...headers(), ...(init?.headers as Record<string, string>) },
  });
  const data = await res.json();
  if (!res.ok) throw new Error(data.error || 'api_error');
  return data as T;
}

export async function authApi<T>(path: string, body: object): Promise<T> {
  const res = await fetch(`${API}/api/v1/auth${path}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  const data = await res.json();
  if (!res.ok) throw new Error(data.error || 'auth_error');
  return data as T;
}

export { API };
