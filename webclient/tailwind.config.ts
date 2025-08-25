import type { Config } from "tailwindcss";
import { current, green, inherit, orange, red, transparent, white, zinc } from "tailwindcss/colors";

export default <Partial<Config>>{
  darkMode: "class",
  content: ["./**/*.{vue,js,ts,jsx,tsx}"],
  theme: {
    colors: {
      inherit,
      current,
      transparent,
      white,
      zinc,
      orange,
      green,
      "fuchsia-pink": {
        "50": "#faf5fa",
        "100": "#f7ecf6",
        "200": "#f1d9f0",
        "300": "#e6bbe2",
        "400": "#d591cf",
        "500": "#c56fb9",
        DEFAULT: "#b55ca5",
        "600": "#b55ca5",
        "700": "#973f85",
        "800": "#7d376d",
        "900": "#6a315d",
        "950": "#3f1837",
      },
      red,
      blue: {
        50: "#f3f6fc",
        100: "#e5edf9",
        200: "#c6d9f1",
        300: "#93bae6",
        400: "#5a97d6",
        DEFAULT: "#3070b3",
        500: "#3070b3",
        600: "#245fa5",
        700: "#1f4d85",
        800: "#1d426f",
        900: "#1d385d",
        950: "#13243e",
      },
    },
    extend: {
      aspectRatio: {
        "4/3": "4 / 3",
      },
    },
  },
  plugins: [require("nightwind")],
};
