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

  canvas.classList.remove("invisible");
  resetMouseState();
  frameRequest = requestAnimationFrame(render);
};

export const hide = () => {
  canvas.classList.add("invisible");
  cancelAnimationFrame(render);
};
