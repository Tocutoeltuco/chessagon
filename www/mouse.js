/**
 * @typedef {{x: number, y: number}} Position
 */
/**
 * @typedef {{start: Position, end: Position}} Click
 */
/**
 * @typedef {{state: 0 | 1 | 2, pos: Position, end?: Position}} MouseState
 */

/**
 * @type {Click[]}
 */
let clickQueue = [];
/**
 * @type {{current: boolean, x: number, y:number}}
 */
let click = {
  current: false,
  x: 0,
  y: 0,
};
/**
 * @type {Position}
 */
let mouse = {
  x: 0,
  y: 0,
};

/**
 * @param {MouseEvent} e
 */
const onmousemove = (e) => {
  mouse.x = e.offsetX;
  mouse.y = e.offsetY;
};

/**
 * @param {MouseEvent} e
 */
const onmousedown = (e) => {
  click.current = true;
  click.x = e.offsetX;
  click.y = e.offsetY;
};

/**
 * @param {MouseEvent} e
 */
const onmouseup = (e) => {
  click.current = false;
  clickQueue.unshift({
    start: {
      x: click.x,
      y: click.y,
    },
    end: {
      x: e.offsetX,
      y: e.offsetY,
    },
  });
};

/**
 * Sets up mouse listeners.
 * @param {HTMLElement} canvas
 */
export const setupMouse = (canvas) => {
  canvas.addEventListener("mousemove", onmousemove);
  canvas.addEventListener("mousedown", onmousedown);
  canvas.addEventListener("mouseup", onmouseup);
};

/**
 * Returns mouse state.
 * @returns {MouseState}
 */
export const getMouseState = () => {
  const item = clickQueue.pop();
  if (item !== undefined) {
    return {
      state: 2,
      pos: item.start,
      end: item.end,
    };
  }

  if (click.current) {
    return {
      state: 1,
      pos: click,
    };
  }

  return {
    state: 0,
    pos: mouse,
  };
};

export const resetMouseState = () => {
  clickQueue = [];
};
