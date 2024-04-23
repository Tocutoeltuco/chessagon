import { AssetManager } from "./assets.js";
import { Board } from "./board.js";
import { setupMouse, resetMouseState } from "./mouse.js";
import { start, on_assets_ready, on_hex_clicked } from "../pkg/chessagon.js";

let ctx;

const assets = new AssetManager();
const board = new Board(11, assets);
const canvas = document.getElementById("board");
let frameRequest;

const colors = ["#d18b47", "#e8ab6f", "#ffce9e"];
const effects = {
  light: "rgba(255, 255, 0, 0.35)",
  hover: "rgba(255, 255, 255, 0.4)",
  click: "rgba(255, 255, 255, 0.5)",
};

export const loadAssets = () => {
  const pieces = ["k", "q", "r", "b", "n", "p"];
  for (let i = 0; i < pieces.length; i++) {
    assets.fetchAndSave(`./assets/piece_${pieces[i]}l.svg`);
    assets.fetchAndSave(`./assets/piece_${pieces[i]}d.svg`);
  }

  assets.fetch("./assets/hexagon.svg").then((content) => {
    for (let i = 0; i < colors.length; i++) {
      assets.create(`hex_${i}`, content.replace(/#000000/g, colors[i]));
    }

    for (const [name, color] of Object.entries(effects)) {
      assets.create(`hex_effect_${name}`, content.replace(/#000000/g, color));
    }

    assets.waitReady().then(() => {
      // Trigger event
      on_assets_ready(ctx);
    });
  });
};

export const resetBoard = () => {
  board.highlight([]);
  board.setPieces([]);
};

export const resumeBoard = () => {
  resetMouseState();
  frameRequest = requestAnimationFrame(render);
};

export const pauseBoard = () => {
  if (frameRequest === undefined) {
    return;
  }
  cancelAnimationFrame(frameRequest);
};

export const setPieces = (pieces) => {
  board.setPieces(pieces);
};
export const movePieces = (pieces) => {
  board.movePieces(pieces);
};
export const highlight = (pieces) => {
  board.highlight(pieces);
};

/**
 * @param {number} q
 * @param {number} r
 */
board.onClick = (q, r) => {
  on_hex_clicked(ctx, q, r);
};

setupMouse(canvas);

window.visualViewport.onresize = () => {
  board.resize();
};

const render = () => {
  board.render(canvas);
  frameRequest = requestAnimationFrame(render);
};

export const main = (context) => {
  ctx = context;
  start(ctx);
};
