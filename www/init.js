import { AssetManager } from "./assets.js";
import { Board } from "./board.js";
import { setupMouse, resetMouseState } from "./mouse.js";
import { on_assets_ready, on_hex_clicked } from "../pkg/chessagon.js";

const assets = new AssetManager();
const board = new Board(11, assets);
const canvas = document.getElementById("board");
let frameRequest;

const colors = [
  {
    normal: "hsl(0, 0%, 0%)",
    light: "hsl(0, 0%, 15%)",
    hover: "hsl(0, 0%, 20%)",
    click: "hsl(0, 0%, 25%)",
  },
  {
    normal: "hsl(0, 0%, 35%)",
    light: "hsl(0, 0%, 45%)",
    hover: "hsl(0, 0%, 50%)",
    click: "hsl(0, 0%, 60%)",
  },
  {
    normal: "hsl(0, 0%, 70%)",
    light: "hsl(0, 0%, 85%)",
    hover: "hsl(0, 0%, 90%)",
    click: "hsl(0, 0%, 100%)",
  },
];

export const loadAssets = () => {
  const pieces = ["k", "q", "r", "b", "n", "p"];
  for (let i = 0; i < pieces.length; i++) {
    assets.fetchAndSave(`./assets/piece_${pieces[i]}l.svg`);
    assets.fetchAndSave(`./assets/piece_${pieces[i]}d.svg`);
  }

  assets.fetch("./assets/hexagon.svg").then((content) => {
    for (let i = 0; i < colors.length; i++) {
      for (const [name, color] of Object.entries(colors[i])) {
        assets.create(`hex_${name}_${i}`, content.replace(/#000000/g, color));
      }
    }

    assets.waitReady().then(() => {
      // Trigger event
      on_assets_ready();
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
  on_hex_clicked(q, r);
};

setupMouse(canvas);

window.visualViewport.onresize = () => {
  board.resize();
};

const render = () => {
  board.render(canvas);
  frameRequest = requestAnimationFrame(render);
};
