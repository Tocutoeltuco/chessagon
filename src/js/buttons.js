import { ctx } from "./state";

const container = document.getElementById("game-buttons");
const buttons = ["resign", "play"];

for (let i = 0; i < buttons.length; i++) {
  const name = buttons[i];
  buttons[i] = container.querySelector(`[data-action=${name}]`);
  buttons[i].onclick = () => {
    ctx.gameButtonClick(i);
  };
}

/**
 * @param {Uint8Array} ids
 */
export const showButtons = (ids) => {
  for (let i = 0; i < buttons.length; i++) {
    buttons[i].hidden = true;
  }
  for (let i = 0; i < ids.length; i++) {
    buttons[ids[i]].hidden = false;
  }
};
