const LEVELS = { debug: 10, info: 20, warn: 30, error: 40 };
const MIN = LEVELS[process.env.LOG_LEVEL || 'info'] ?? 20;

export const logger = {
  _log(level, msg, meta = {}) {
    if ((LEVELS[level] ?? 20) < MIN) return;
    const line = JSON.stringify({ ts: new Date().toISOString(), level, msg, ...meta });
    if (level === 'error') console.error(line);
    else console.log(line);
  },
  debug: (m, meta) => logger._log('debug', m, meta),
  info: (m, meta) => logger._log('info', m, meta),
  warn: (m, meta) => logger._log('warn', m, meta),
  error: (m, meta) => logger._log('error', m, meta),
};
