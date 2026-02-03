/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        // Dark theme inspired by OpenClaw
        surface: {
          DEFAULT: '#0a0a0a',
          elevated: '#141414',
          50: '#1a1a1a',
          100: '#2a2a2a',
          200: '#3a3a3a',
        },
        accent: {
          DEFAULT: '#6366f1',
          hover: '#818cf8',
        },
        success: '#22c55e',
        warning: '#f59e0b',
        danger: '#ef4444',
      },
    },
  },
  plugins: [],
}
