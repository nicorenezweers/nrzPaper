// Tailwind für nrzPaper – erzeugt static/app.css aus assets/app.css (scripts/tailwind.sh).
// Klassen stehen nur in den Templates und in app.js/viewer.js (nicht im Rust-Code).
module.exports = {
  content: ["./templates/**/*.html", "./static/*.js"],
  theme: {
    extend: {
      fontFamily: { sans: ["Inter", "ui-sans-serif", "system-ui", "sans-serif"] },
    },
  },
  plugins: [],
};
