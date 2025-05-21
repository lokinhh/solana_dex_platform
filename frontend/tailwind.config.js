module.exports = {
  content: ['./src/**/*.{js,ts,jsx,tsx}'],
  theme: {
    extend: {
      colors: {
        void: '#08090f',
        panel: '#11131c',
        edge: '#1e2233',
        sol: '#9945ff',
        mint: '#00ffd5',
        warn: '#ffb347',
      },
      fontFamily: {
        display: ['var(--font-display)', 'system-ui', 'sans-serif'],
        mono: ['var(--font-mono)', 'monospace'],
      },
    },
  },
  plugins: [],
};
