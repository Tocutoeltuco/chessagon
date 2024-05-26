import { AssetManager } from "./assets";
import { PieceKindAsset } from "./layer";

let isReady = false;
let toResolve = [];

const border = "#ffffff";
export const colors = {
  main: ["#e8ab6f", "#ffce9e", "#d18b47"],
  promotion: ["#387039", "#5db55f", "#abd2ac"],
};
const effects = {
  light: "rgba(255, 255, 0, 0.35)",
  check: "rgba(255, 0, 0, 0.35)",
  hover: "rgba(255, 255, 255, 0.4)",
  click: "rgba(255, 255, 255, 0.5)",
};

export const assets = new AssetManager();
for (const kind of Object.values(PieceKindAsset)) {
  assets.fetchAndSave(`./assets/piece_${kind}l.svg`, `piece_${kind}l`);
  assets.fetchAndSave(`./assets/piece_${kind}d.svg`, `piece_${kind}d`);
}

assets.fetch("./assets/hexagon.svg").then((content) => {
  assets.create("hex_border", content.replace(/#/g, border));

  for (let i = 0; i < colors.main.length; i++) {
    assets.create(`hex_${i}`, content.replace(/#/g, colors.main[i]));
  }
  for (let i = 0; i < colors.promotion.length; i++) {
    assets.create(`prom_hex_${i}`, content.replace(/#/g, colors.promotion[i]));
  }

  for (const [name, color] of Object.entries(effects)) {
    assets.create(`hex_effect_${name}`, content.replace(/#/g, color));
  }

  assets.waitReady().then(() => {
    isReady = true;
    for (let i = 0; i < toResolve.length; i++) {
      toResolve[i]();
    }
    toResolve = [];
  });
});

export const onReady = () => {
  return new Promise((resolve) => {
    if (isReady) {
      return resolve();
    }

    toResolve.push(resolve);
  });
};
