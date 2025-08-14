/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    './pages/**/*.{js,ts,jsx,tsx,mdx}',
    './src/**/*.{js,ts,jsx,tsx,mdx}',
    './components/**/*.{js,ts,jsx,tsx,mdx}',
  ],
  theme: {
    extend: {
      colors: {
        quantum: {
          primary: '#00fdfd',
          dark: '#061018',
          darker: '#07121a',
          accent: '#0a1f2b',
        }
      }
    },
  },
  plugins: [],
}
