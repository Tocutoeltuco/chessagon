import { resetMouseState, setupMouse } from "./mouse";
import { board } from "./state";
import { drawGraph, GraphData } from "./graph";

/**
 * @type {HTMLCanvasElement}
 */
const canvas = document.getElementById("board");
const graphs = new GraphData();
let firstShow = false;

canvas.width = window.innerWidth;
canvas.height = window.innerHeight;

setupMouse(canvas);

graphs.addField("rtt", 10);
graphs.addField("fps", 10, 1000, (values) => values.length);
graphs.addField("renderTime", 10, 500, (values) => {
  let total = 0;
  for (let i = 0; i < values.length; i++) {
    total += values[i];
  }
  return total / values.length;
});

const render = () => {
  const start = performance.now();
  let ctx = canvas.getContext("2d");

  board.render(ctx);

  const data = graphs.getCurrent();
  drawGraph(ctx, canvas.width - 150, 20, "FPS", data.fps);
  drawGraph(ctx, canvas.width - 150, 100, "Render time (ms)", data.renderTime);
  drawGraph(ctx, canvas.width - 150, 180, "RTT (ms)", data.rtt);

  const end = performance.now();

  graphs.addData("fps", 1);
  graphs.addData("renderTime", end - start);

  if (!canvas.hidden) requestAnimationFrame(render);
};

/**
 * @param {number} time
 */
export const addRTT = (time) => {
  graphs.addData("rtt", time);
};

export const show = () => {
  if (!firstShow) {
    firstShow = true;
    const handler = () => {
      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight;

      board.resize();
    };
    handler();
    window.visualViewport.onresize = handler;
  }

  canvas.hidden = false;
  resetMouseState();
  requestAnimationFrame(render);
};

export const hide = () => {
  canvas.hidden = true;
};
