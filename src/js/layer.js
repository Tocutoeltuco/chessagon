// https://www.redblobgames.com/grids/hexagons/

import { AssetManager } from "./assets";
import { getMouseState } from "./mouse";

const SQRT = Math.sqrt(3);

/**
 * @readonly
 * @enum {number}
 */
export const Shape = {
  SQUARE: 0,
  HEX: 1,
};

/**
 * @readonly
 * @enum {number}
 */
export const PieceKind = {
  KING: 0,
  QUEEN: 1,
  ROOK: 2,
  BISHOP: 3,
  KNIGHT: 4,
  PAWN: 5,
};

export const PieceKindAsset = {
  [PieceKind.KING]: "k",
  [PieceKind.QUEEN]: "q",
  [PieceKind.ROOK]: "r",
  [PieceKind.BISHOP]: "b",
  [PieceKind.KNIGHT]: "n",
  [PieceKind.PAWN]: "p",
};

export const EffectsAsset = ["light", "check"];

/**
 * @readonly
 * @enum {number}
 */
export const Color = {
  LIGHT: 0,
  DARK: 1,
};

export class Piece {
  /**
   * @param {PieceKind} kind
   * @param {Color} color
   * @param {number} q
   * @param {number} r
   */
  constructor(kind, color, q, r) {
    this.kind = kind;
    this.color = color;
    this.q = q;
    this.r = r;
  }

  get assetKey() {
    const kind = PieceKindAsset[this.kind];
    const color = this.color == Color.LIGHT ? "l" : "d";
    return `piece_${kind}${color}`;
  }
}

export class Layer {
  /**
   * @param {Object} opt
   * @param {Shape} opt.shape
   * @param {number} opt.colors
   * @param {number} opt.size
   * @param {AssetManager} opt.assets
   * @param {number?} opt.borderSize
   * @param {string?} opt.assetPrefix
   */
  constructor(opt) {
    this.shape = opt.shape;
    this.colors = opt.colors;
    this.size = opt.size;
    this.assets = opt.assets;
    this.borderSize = opt.borderSize;
    this.assetPrefix = opt.assetPrefix || "";

    this.flipped = false;
    /**
     * @type {Piece[]}
     */
    this.pieces = [];
    /**
     * @type {{q: number, r: number, effects: string[]}[]}
     */
    this.highlight = [];

    this.updated = new Uint8Array(this.size * this.size);
    this.nextUpdate = new Uint8Array(this.size * this.size);
    this.fullUpdate = true;
    this.nextFullUpdate = false;
  }

  getAppropriateRadius(width, height) {
    const hWidth = 4 * width * (1 + 1 / (3 * this.size));
    const hHeight = height / this.size;

    return Math.min(hWidth / 2, hHeight / SQRT);
  }

  /**
   * @param {number} hexRadius
   */
  resize(hexRadius) {
    const hWidth = hexRadius * 2;
    const hHeight = hexRadius * SQRT;

    this.hexRadius = hexRadius;
    this.width = 0.75 * hWidth * this.size + 0.25 * hWidth;

    if (this.shape == Shape.SQUARE) {
      this.height = hHeight * (this.size + (this.size - 1) / 2);
      this.yOffset = 4;
    } else {
      this.height = hHeight * this.size;
      const middle = Math.floor(this.size / 2);
      this.yOffset = (SQRT / 2) * middle * hexRadius + 4;
    }

    this.fullUpdate = true;
    this.nextFullUpdate = true;
  }

  /**
   * @param {number} x
   * @param {number} y
   */
  move(x, y) {
    this.x = x;
    this.y = y;
    this.fullUpdate = true;
    this.nextFullUpdate = true;
  }

  flip(state) {
    if (this.flipped === state) return;
    this.flipped = state;
    this.fullUpdate = true;
    this.nextFullUpdate = true;
  }

  shouldUpdate(q, r) {
    if (this.fullUpdate) return true;

    return this.updated[q * this.size + r] === 1;
  }

  markUpdate(q, r) {
    this.nextUpdate[q * this.size + r] = 1;
  }

  _flipX(x) {
    if (this.flipped) {
      return this.width - x;
    }
    return x;
  }
  _flipY(y) {
    if (this.flipped) {
      return this.height - y;
    }
    return y;
  }

  /**
   * @param {number} q
   * @param {number} r
   */
  getPixel(q, r) {
    const x = 1.5 * q * this.hexRadius + this.hexRadius;
    const y = SQRT * (q / 2 + r) * this.hexRadius + this.hexRadius;

    return [this.x + this._flipX(x), this.y + this._flipY(y - this.yOffset)];
  }

  /**
   * @param {number} x
   * @param {number} y
   */
  getHex(x, y) {
    x = this._flipX(x - this.x) / this.hexRadius - 1;
    y = (this._flipY(y - this.y) + this.yOffset) / this.hexRadius - 1;

    const q = x / 1.5;
    const r = y / SQRT - q / 2;

    const qgrid = Math.round(q);
    const rgrid = Math.round(r);

    const dq = q - qgrid;
    const dr = r - rgrid;

    if (dq * dq >= dr * dr) {
      return [qgrid + Math.round(dq + 0.5 * dr), rgrid];
    }
    return [qgrid, rgrid + Math.round(dr + 0.5 * dq)];
  }

  isInBounds(q, r) {
    if (q < 0 || r < 0) {
      return false;
    }
    if (q >= this.size || r >= this.size) {
      return false;
    }

    if (this.shape == Shape.SQUARE) {
      // No more checks for square shape.
      return true;
    }

    // Hexagonal shape requires corner cutoff
    if (r < Math.floor(this.size / 2) - q) {
      // Top left corner
      return false;
    }
    if (r > Math.floor(this.size * 1.5) - q - 1) {
      // Bottom right corner
      return false;
    }
    return true;
  }

  *hexIterator() {
    for (let q = 0; q < this.size; q++) {
      for (let r = 0; r < this.size; r++) {
        if (!this.shouldUpdate(q, r)) continue;
        if (!this.isInBounds(q, r)) continue;
        yield [q, r];
      }
    }
  }

  *borderIterator() {
    if (this.shape == Shape.SQUARE) {
      for (let q = 0; q < this.size; q++) {
        yield [q, 0];
        yield [q, this.size - 1];
      }

      for (let r = 1; r < this.size - 1; r++) {
        yield [0, r];
        yield [this.size - 1, r];
      }

      return;
    }

    const half = Math.floor(this.size / 2);

    for (let i = 0; i <= half; i++) {
      yield [i, half - i];
      yield [half + i, this.size - 1 - i];
    }

    for (let i = 0; i <= half; i++) {
      yield [10, i];
      yield [i, 10];
    }

    for (let i = half; i < this.size; i++) {
      yield [0, i];
      yield [i, 0];
    }
  }

  *highlightIterator() {
    for (let i = 0; i < this.highlight.length; i++) {
      const highlight = this.highlight[i];
      if (!this.shouldUpdate(highlight.q, highlight.r)) continue;
      yield highlight;
    }
  }

  *pieceIterator() {
    for (let i = 0; i < this.pieces.length; i++) {
      const piece = this.pieces[i];
      if (!this.shouldUpdate(piece.q, piece.r)) continue;
      if (!this.isInBounds(piece.q, piece.r)) continue;
      yield piece;
    }
  }

  /**
   * @param {number} _q
   * @param {number} _r
   */
  onClick(_q, _r) {}

  /**
   * @param {CanvasRenderingContext2D} ctx
   */
  handleMouse(ctx) {
    const mouse = getMouseState();
    let [q, r] = this.getHex(mouse.pos.x, mouse.pos.y);
    if (!this.isInBounds(q, r)) {
      return;
    }
    this.markUpdate(q, r);

    let effect;
    if (mouse.state === 0) {
      effect = "hover";
    } else {
      effect = "click";
    }

    const [x, y] = this.getPixel(q, r);
    ctx.drawImage(
      this.assets.get(`hex_effect_${effect}`),
      x - this.hexRadius,
      y - this.hexRadius,
      this.hexRadius * 2,
      this.hexRadius * 2,
    );

    if (mouse.state === 2) {
      let [Q, R] = this.getHex(mouse.end.x, mouse.end.y);
      if (q === Q && r === R) {
        this.onClick(q, r);
      }
    }
  }

  /**
   * @param {CanvasRenderingContext2D} ctx
   */
  render(ctx) {
    const diam = this.hexRadius * 2;

    if (this.borderSize && this.fullUpdate) {
      const asset = this.assets.get(`hex_border`);
      const radius = this.hexRadius + this.borderSize;
      const diameter = radius * 2;

      for (const [q, r] of this.borderIterator()) {
        const [x, y] = this.getPixel(q, r);
        ctx.drawImage(asset, x - radius, y - radius, diameter, diameter);
      }
    }

    for (const [q, r] of this.hexIterator()) {
      const [x, y] = this.getPixel(q, r);
      const color = (q + r * 2) % this.colors;

      ctx.drawImage(
        this.assets.get(`${this.assetPrefix}hex_${color}`),
        x - this.hexRadius,
        y - this.hexRadius,
        diam,
        diam,
      );
    }

    for (const { q, r, effects } of this.highlightIterator()) {
      const [x, y] = this.getPixel(q, r);

      for (let j = 0; j < effects.length; j++) {
        ctx.drawImage(
          this.assets.get(`hex_effect_${effects[j]}`),
          x - this.hexRadius,
          y - this.hexRadius,
          diam,
          diam,
        );
      }
    }

    const pieceDiam = diam * 0.75;
    const pieceOffset = (diam - pieceDiam) / 2 - this.hexRadius;
    for (const piece of this.pieceIterator()) {
      const [x, y] = this.getPixel(piece.q, piece.r);
      const asset = this.assets.get(piece.assetKey);

      ctx.drawImage(
        asset,
        pieceOffset + x,
        pieceOffset + y,
        pieceDiam,
        pieceDiam,
      );
    }
  }

  finishUpdate() {
    this.fullUpdate = this.nextFullUpdate;
    this.nextFullUpdate = false;
    this.updated = this.nextUpdate;
    this.nextUpdate = new Uint8Array(this.size * this.size);
  }
}
