import "../scss/styles.scss";
import { assets, ctx } from "./state";

const colors = ["#d18b47", "#e8ab6f", "#ffce9e"];
const effects = {
  light: "rgba(255, 255, 0, 0.35)",
  hover: "rgba(255, 255, 255, 0.4)",
  click: "rgba(255, 255, 255, 0.5)",
};

for (const piece of ["k", "q", "r", "b", "n", "p"]) {
  assets.fetchAndSave(`./assets/piece_${piece}l.svg`);
  assets.fetchAndSave(`./assets/piece_${piece}d.svg`);
}

assets.fetch("./assets/hexagon.svg").then((content) => {
  for (let i = 0; i < colors.length; i++) {
    const color = colors[i];
    assets.create(`hex_${i}`, content.replace(/#000000/g, color));
  }

  for (const [name, color] of Object.entries(effects)) {
    assets.create(`hex_effect_${name}`, content.replace(/#000000/g, color));
  }

  assets.waitReady().then(() => {
    // Ready!
    ctx.start();
  });
});
