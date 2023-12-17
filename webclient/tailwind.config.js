/** @type {import('tailwindcss').Config} */
const colors = require("tailwindcss/colors");

export default {
  content: ["./index.html", "./src/**/*.{vue,js,ts,jsx,tsx}"],
  theme: {
    colors: {
      ...colors,
      tumBlue: {
        50: "#f3f6fc",
        100: "#e5edf9",
        200: "#c6d9f1",
        300: "#93bae6",
        400: "#5a97d6",
        500: "#3070b3",
        600: "#245fa5",
        700: "#1f4d85",
        800: "#1d426f",
        900: "#1d385d",
        950: "#13243e",
      },
    },
    extend: {},
  },
  plugins: [],
};
