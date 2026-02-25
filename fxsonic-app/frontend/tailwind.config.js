/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        'fx-bg': '#1a1a2e',
        'fx-bg-dark': '#0f0f1a',
        'fx-accent': '#e94560',
        'fx-accent-hover': '#ff6b81',
        'fx-text': '#e0e0e0',
        'fx-text-dim': '#a0a0a0',
        'fx-slider': '#2a2a4a',
      },
    },
  },
}
