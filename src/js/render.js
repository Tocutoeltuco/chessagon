import { resetMouseState, setupMouse } from "./mouse";
import { board } from "./state";

const canvas = document.getElementById("board");
let firstShow = false;
let frameRequest;

setupMouse(canvas);

const render = () => {
  board.render(canvas);
  frameRequest = requestAnimationFrame(render);
};

export const show = () => {
  if (!firstShow) {
    firstShow = true;
    window.visualViewport.onresize = () => board.resize();
  }

  canvas.hidden = false;
  resetMouseState();
  frameRequest = requestAnimationFrame(render);
};

export const hide = () => {
  canvas.hidden = true;
  cancelAnimationFrame(render);
};
