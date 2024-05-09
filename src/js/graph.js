/**
 * @param {CanvasRenderingContext2D} ctx
 * @param {number} x
 * @param {number} y
 * @param {string} title
 * @param {number[]} data
 * @param {{min?: number, max?: number} | undefined} opt
 */
export const drawGraph = (ctx, x, y, title, data, opt) => {
  ctx.font = "10px sans-serif";
  ctx.textBaseline = "alphabetic";
  ctx.strokeStyle = "#ffffff";
  ctx.strokeText(title, x, y);
  ctx.strokeRect(x, y + 5, 120, 50);

  let min;
  if (opt?.min !== undefined) {
    min = opt.min;
  } else {
    min = Math.min(...data);
  }

  let max;
  if (opt?.max !== undefined) {
    max = opt.max;
  } else {
    max = Math.max(...data);
  }

  if (max - min < 10) {
    let delta = Math.floor((10 - (max - min)) / 2);

    if (min - delta < 0) {
      max += 2 * delta - min;
      min = 0;
    } else {
      min -= delta;
      max += delta;
    }
  }

  ctx.strokeText(min, x - 20, y + 50);
  ctx.strokeText(max, x - 20, y + 15);

  const top = y + 10;
  const left = x + 5;
  const bot = y + 48;
  const right = x + 113;

  const dist = (right - left) / (data.length - 1);
  const height = bot - top;
  const dataHeight = max - min;

  ctx.strokeStyle = "#759b92";
  ctx.beginPath();
  ctx.moveTo(left, bot - ((data[0] - min) * height) / dataHeight);

  for (let i = 1; i < data.length; i++) {
    ctx.lineTo(left + dist * i, bot - ((data[i] - min) * height) / dataHeight);
  }

  ctx.stroke();

  const last = data[data.length - 1];
  ctx.strokeText(
    last.toString(),
    x + 125,
    bot - ((last - min) * height) / dataHeight + 3,
  );
};

/**
 * Aggregates data of a periodic field
 * @callback aggregator
 * @param {number[]} data
 * @returns {number}
 */

export class GraphData {
  constructor() {
    /**
     * @type {Record<string, {period: number?, nextPeriod: number, current: number[], aggregator: aggregator?}>}
     */
    this.fields = {};
    /**
     * @type {Record<string, number[]>}
     */
    this.history = {};
  }

  /**
   * @param {string} name
   * @param {number} length
   * @param {number?} period
   * @param {aggregator?} aggregator
   */
  addField(name, length, period, aggregator) {
    this.fields[name] = {
      current: [],
      aggregator: aggregator,
      period: period,
      nextPeriod: 0,
    };
    this.history[name] = new Array(length).fill(0);
  }

  /**
   * @param {string} field
   * @param {number} value
   */
  addData(field, value) {
    if (!this.fields[field].period) {
      this.history[field].shift();
      this.history[field].push(Math.floor(value));
      return;
    }

    this.fields[field].current.push(value);
  }

  getCurrent() {
    const now = performance.now();

    for (const [name, field] of Object.entries(this.fields)) {
      if (!field.period) continue;
      if (field.nextPeriod > now) continue;
      field.nextPeriod = now + field.period;

      let value;
      if (field.current.length == 0) {
        value = 0;
      } else {
        value = Math.floor(field.aggregator(field.current));
      }
      if (isNaN(value)) {
        value = 0;
      }

      field.current = [];
      this.history[name].shift();
      this.history[name].push(value);
    }

    return this.history;
  }
}
