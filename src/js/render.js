import { resetMouseState, setupMouse } from "./mouse";
import { board, ctx } from "./state";
import { drawGraph, GraphData } from "./graph";

/**
 * @type {HTMLCanvasElement}
 */
const canvas = document.getElementById("board");
const graphs = new GraphData();
let firstShow = false;
let frameRequest;
/**
 * @typedef {{count: number, activatedAt: number?}} Timer
 */
/**
 * @type {{light: Timer, dark: Timer}?}
 */
let timers = null;
let sentExpired = false;

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

const sendTimerExpired = () => {
  if (sentExpired) return;
  sentExpired = true;
  ctx.timerExpired();
};

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

  if (left === 0) {
    sendTimerExpired();
  }
};

const render = () => {
  const start = performance.now();
  let ctx = canvas.getContext("2d");
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  board.render(ctx);

  if (timers) {
    const margin = 20;
    let [top, bot] = [timers.dark, timers.light];
    if (board.main.flipped) {
      [top, bot] = [bot, top];
    }
    drawTimer(ctx, margin, top, "top");
    drawTimer(ctx, canvas.height - margin, bot, "bottom");
  }

  const data = graphs.getCurrent();
  drawGraph(ctx, canvas.width - 150, 20, "FPS", data.fps);
  drawGraph(ctx, canvas.width - 150, 100, "Render time (ms)", data.renderTime);
  drawGraph(ctx, canvas.width - 150, 180, "RTT (ms)", data.rtt);

  const end = performance.now();

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
 * @param {number} active
 */
export const setTimers = (light, dark, active) => {
  sentExpired = false;
  timers = {
    light: {
      count: light,
      activatedAt: active == 0 ? performance.now() : null,
    },
    dark: {
      count: dark,
      activatedAt: active == 1 ? performance.now() : null,
    },
  };
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
  frameRequest = requestAnimationFrame(render);
};

export const hide = () => {
  canvas.hidden = true;
  cancelAnimationFrame(render);
};
