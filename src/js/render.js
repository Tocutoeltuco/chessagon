import { resetMouseState, setupMouse } from "./mouse";
import { board } from "./state";
import { drawGraph, PeriodicData } from "./graph";

/**
 * @type {HTMLCanvasElement}
 */
const canvas = document.getElementById("board");
const graphs = new PeriodicData(1000);
let firstShow = false;
let frameRequest;

canvas.width = window.innerWidth;
canvas.height = window.innerHeight;

setupMouse(canvas);

graphs.addField("fps", 10, (values) => values.length);
graphs.addField("renderTime", 10, (values) => {
  let total = 0;
  for (let i = 0; i < values.length; i++) {
    total += values[i];
  }
  return total / values.length;
});

const render = () => {
  const start = Math.floor(performance.now());
  let ctx = canvas.getContext("2d");
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  board.render(ctx);

  const data = graphs.getCurrent();
  drawGraph(ctx, canvas.width - 150, 20, "FPS", data.fps);
  drawGraph(ctx, canvas.width - 150, 100, "Render time (ms)", data.renderTime);

  const end = Math.floor(performance.now());

  graphs.addData("fps", 1);
  graphs.addData("renderTime", end - start);

  frameRequest = requestAnimationFrame(render);
};

export const show = () => {
  if (!firstShow) {
    firstShow = true;
    window.visualViewport.onresize = () => {
      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight;

      board.resize();
    };
  }

  canvas.hidden = false;
  resetMouseState();
  frameRequest = requestAnimationFrame(render);
};

export const hide = () => {
  canvas.hidden = true;
  cancelAnimationFrame(render);
};
