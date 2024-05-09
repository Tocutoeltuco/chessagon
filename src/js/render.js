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
/**
 * @typedef {{count: number, activatedAt: number?}} Timer
 */
/**
 * @type {{light: Timer, dark: Timer}?}
 */
let timers = null;

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

/**
 * @param {CanvasRenderingContext2D} ctx
 * @param {number} y
 * @param {Timer} timer
 * @param {"top" | "bottom"} baseline
 */
const drawTimer = (ctx, y, timer, baseline) => {
  const w = 50;
  const h = 20;
  const x = canvas.width / 2 - 25;

  if (baseline === "bottom") {
    y -= h;
  }

  let left = timer.count;
  if (timer.activatedAt !== null) {
    left -= (performance.now() - timer.activatedAt) / 1000;
    ctx.fillStyle = "#ffffff";
    ctx.fillRect(x - 2, y - 2, w + 4, h + 4);
  }

  ctx.fillStyle = "#46604b";
  ctx.fillRect(x, y, w, h);

  ctx.strokeStyle = left < 20 ? "#ff0000" : "#ffffff";
  ctx.font = "14px monospace";
  ctx.textBaseline = "top";

  left = Math.floor(Math.max(0, left));
  const minutes = Math.floor(left / 60)
    .toString()
    .padStart(2, "0");
  const seconds = (left % 60).toString().padStart(2, "0");
  ctx.strokeText(minutes + ":" + seconds, x + 4, y + 4);
};

const render = () => {
  const start = Math.floor(performance.now());
  let ctx = canvas.getContext("2d");
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  board.render(ctx);

  if (timers) {
    const margin = 20;
    drawTimer(ctx, margin, timers.dark, "top");
    drawTimer(ctx, canvas.height - margin, timers.light, "bottom");
  }

  const data = graphs.getCurrent();
  drawGraph(ctx, canvas.width - 150, 20, "FPS", data.fps);
  drawGraph(ctx, canvas.width - 150, 100, "Render time (ms)", data.renderTime);

  const end = Math.floor(performance.now());

  graphs.addData("fps", 1);
  graphs.addData("renderTime", end - start);

  frameRequest = requestAnimationFrame(render);
};

export const removeTimers = () => {
  timers = null;
};

/**
 * @param {number} light
 * @param {number} dark
 * @param {"light" | "dark" | "none"} active
 */
export const setTimers = (light, dark, active) => {
  timers = {
    light: {
      count: light,
      activatedAt: active == "light" ? performance.now() : null,
    },
    dark: {
      count: dark,
      activatedAt: active == "dark" ? performance.now() : null,
    },
  };
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
