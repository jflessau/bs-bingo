module.exports = {
  content: ['./src/**/*.{html,js,tsx,ts,svelte}'],
  theme: {
    extend: {
      fontSize: {
        xxs: '0.5rem',
      },
    },
    fontFamily: {
      sans: ['sans'],
      serif: ['serif'],
      mono: ['Source Code Pro'],
    },
    colors: {
      raisin: '#232528',
      navy: '#2a2a72',
      sky: '#009ffd',
      sun: '#ffa400',
      solitude: '#eaf6ff',
      gray: '#ddd',
      white: '#fff',
    },
    fontWeight: {
      light: 200,
      normal: 400,
      bold: 700,
    },
  },
  plugins: [],
};
