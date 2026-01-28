/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
  theme: {
    extend: {
      colors: {
        primary: {
          50: '#fef3f2',
          100: '#fee5e2',
          200: '#fdcfca',
          300: '#fbada5',
          400: '#f67e71',
          500: '#ec5545',
          600: '#d93a28',
          700: '#b62d1e',
          800: '#97291c',
          900: '#7d281e',
        },
        accent: {
          orange: '#f7931e',
          green: '#5cb85c',
          blue: '#4a90d9',
          purple: '#9b59b6',
          yellow: '#f0c419',
        },
      },
      fontFamily: {
        sans: ['Outfit', 'system-ui', 'sans-serif'],
        display: ['Clash Display', 'system-ui', 'sans-serif'],
      },
    },
  },
  plugins: [],
};
