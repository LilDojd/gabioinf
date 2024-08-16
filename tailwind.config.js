/** @type {import('tailwindcss').Config} */
module.exports = {
  mode: "all",
  content: [
    // include all rust, html and css files in the src directory
    "./src/**/*.{rs,html,css}",
    // include all html files in the output (dist) directory
    "./dist/**/*.html",
  ],
  theme: {
    extend: {
      colors: {
        "nasty-black": "#1a1a1a",
        jet: "#292929",
        onyx: "#3d3d3d",
        "alien-green": "#c2f9bb",
        white: "#f2f2f2",
        coral: "#ef6f6c",
        glaucolus: "#6b7fd7",
      },
    },
  },
  plugins: [require("@tailwindcss/typography")],
};
