const BORDER_COLOR = "#ffffff";
const BORDER_SIZE = 2;
const BG_COLOR = "#46604b";
const TIMER_COLOR = "#ffffff";
const TIMER_COLOR_DANGER = "#ff0000";
const DANGER_AT = 20;

class Timer {
  constructor() {
    this.count = 0;
    this.activeAt = null;
    this.updated = true;
    this.lastTime = null;
  }

  get expired() {
    if (this.activeAt === null) return false;
    return this.left === 0;
  }

  get elapsed() {
    if (this.activeAt === null) return 0;

    const delta = performance.now() - this.activeAt;
    return Math.round(delta / 1000);
  }

  get left() {
    return Math.max(this.count - this.elapsed, 0);
  }

  get asText() {
    const left = this.left;
    const minutes = Math.floor(left / 60)
      .toString()
      .padStart(2, "0");
    const seconds = (left % 60).toString().padStart(2, "0");
    return `${minutes}:${seconds}`;
  }

  /**
   * @param {number} left
   * @param {boolean} isActive
   */
  update(left, isActive) {
    this.updated = true;
    this.count = left;
    if (isActive) {
      this.activeAt = performance.now();
    } else {
      this.activeAt = null;
    }
  }

  /**
   * @param {CanvasRenderingContext2D} ctx
   * @param {number} x
   * @param {number} y
   * @param {number} w
   * @param {number} h
   * @param {boolean} force
   */
  render(ctx, x, y, w, h, force) {
    const left = this.left;
    if (!force && !this.updated && left == this.lastTime) return;
    this.lastTime = left;
    this.updated = false;
    ctx.clearRect(
      x - BORDER_SIZE,
      y - BORDER_SIZE,
      w + BORDER_SIZE * 2,
      h + BORDER_SIZE * 2,
    );

    if (this.activeAt !== null) {
      // Draw border
      ctx.fillStyle = BORDER_COLOR;
      ctx.fillRect(
        x - BORDER_SIZE,
        y - BORDER_SIZE,
        w + BORDER_SIZE * 2,
        h + BORDER_SIZE * 2,
      );
    }

    ctx.fillStyle = BG_COLOR;
    ctx.fillRect(x, y, w, h);

    const letterWidth = 8 + 1 / 3;
    const letterHeight = 12;
    const xOff = (w - letterWidth * 5) / 2;
    const yOff = (h - letterHeight) / 2;
    if (left < DANGER_AT) {
      ctx.strokeStyle = TIMER_COLOR_DANGER;
    } else {
      ctx.strokeStyle = TIMER_COLOR;
    }
    ctx.font = "normal normal 14px monospace";
    ctx.textBaseline = "top";
    ctx.strokeText(this.asText, x + xOff, y + yOff);
  }
}

export class Timers {
  constructor() {
    this.light = new Timer();
    this.dark = new Timer();
    this.hidden = true;
    this.flipped = false;
    this.sentExpiration = false;
  }

  /**
   * @param {number} light
   * @param {number} dark
   * @param {number} active
   */
  setState(light, dark, active) {
    this.sentExpiration = false;
    this.light.update(light, active === 0);
    this.dark.update(dark, active === 1);
  }

  onExpired() {}

  /**
   * @param {HTMLCanvasElement} ctx
   * @param {number} x
   * @param {number} y
   * @param {number} w
   * @param {number} h
   * @param {boolean} force
   */
  render(ctx, x, y, w, h, force) {
    if (this.hidden) return;

    let top = this.dark;
    let bot = this.light;
    if (this.flipped) {
      top = this.light;
      bot = this.dark;
    }

    const timerWidth = 50;
    const timerHeight = 20;

    top.render(
      ctx,
      x + (w - timerWidth) / 2,
      y + 20,
      timerWidth,
      timerHeight,
      force,
    );
    bot.render(
      ctx,
      x + (w - timerWidth) / 2,
      y + h - 20 - timerHeight,
      timerWidth,
      timerHeight,
      force,
    );

    if (this.sentExpiration) return;
    if (this.light.expired || this.dark.expired) {
      this.sentExpiration = true;
      this.onTimerExpired();
    }
  }
}
