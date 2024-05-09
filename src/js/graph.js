/**
 * @param {CanvasRenderingContext2D} ctx
 * @param {number} x
 * @param {number} y
 * @param {string} title
 * @param {number[]} data
 * @param {{min?: number, max?: number} | undefined} opt
 */
export const drawGraph = (ctx, x, y, title, data, opt) => {
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

export class PeriodicData {
  constructor(period) {
    this.period = period;
    this.nextPeriod = 0;
    /**
     * @type {Record<string, {current: number[], aggregator: aggregator}>}
     */
    this.fields = {};
    /**
     * @type {Record<string, number[]>}
     */
    this.history = {};
  }

  tryFinishPeriod() {
    const now = performance.now();
    if (this.nextPeriod > now) return;

    this.nextPeriod = now + this.period;
    for (const [name, field] of Object.entries(this.fields)) {
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
  }

  /**
   * @param {string} name
   * @param {number} length
   * @param {aggregator} aggregator
   */
  addField(name, length, aggregator) {
    this.fields[name] = {
      current: [],
      aggregator: aggregator,
    };
    this.history[name] = new Array(length).fill(0);
  }

  /**
   * @param {string} field
   * @param {number} value
   */
  addData(field, value) {
    this.tryFinishPeriod();
    this.fields[field].current.push(value);
  }

  getCurrent() {
    this.tryFinishPeriod();
    return this.history;
  }
}
