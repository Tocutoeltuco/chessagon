import { AssetManager } from "./assets";
import { getMouseState } from "./mouse";
import { ctx } from "./state";

const SQRT = Math.sqrt(3);
const PIECES = ["k", "q", "r", "b", "n", "p"];
const EFFECTS = ["light", "check"];

/**
 * @typedef {{"k" | "q" | "r" | "b" | "n" | "p"}} PieceType
 */
/**
 * @typedef {{kind: PieceType, light: boolean, q: number, r: number}} Piece
 */

export class Board {
  /**
   * @param {number} size
   * @param {AssetManager} assets
   */
  constructor(size, assets) {
    this.size = size;
    this.assets = assets;

    this.x = 0;
    this.y = 0;
    this.width = 0;
    this.height = 0;
    this.hexSize = 0;
    this.resized = false;
    this.highlighted = [];
    /**
     * @type {Piece[]}
     */
    this.pieces = [];

    this.resize();
  }

  resize() {
    this.width = window.innerWidth;
    this.height = window.innerHeight;

    // Total width is 17x hex size
    // Total height is 19x hex size
    // Desired size is 95% width OR 80% height

    const desiredW = (0.95 * this.width) / 17;
    const desiredH = (0.8 * this.height) / 19;
    this.hexSize = Math.min(desiredW, desiredH);

    this.x = (this.width - this.hexSize * 17) / 2;
    this.y = (this.height - this.hexSize * 19) / 2 - 5;

    const middle = Math.floor(this.size / 2);
    this.y -= (SQRT / 2) * middle * this.hexSize;

    this.resized = true;
  }

  _getPixel(q, r) {
    const x = 1.5 * q;
    const y = (SQRT / 2) * q + SQRT * r;

    return [x * this.hexSize + this.x, y * this.hexSize + this.y];
  }

  _getHex(x, y) {
    x = (x - this.x) / this.hexSize - 1;
    y = (y - this.y) / this.hexSize - 1;

    // Get hex position
    const q = x / 1.5;
    const r = (y - (SQRT / 2) * q) / SQRT;

    // Round to nearest hex
    const qgrid = Math.round(q);
    const rgrid = Math.round(r);

    const dq = q - qgrid;
    const dr = r - rgrid;

    if (dq * dq >= dr * dr) {
      return [qgrid + Math.round(dq + 0.5 * dr), rgrid];
    }
    return [qgrid, rgrid + Math.round(dr + 0.5 * dq)];
  }

  /**
   * Highlights a subset of hexes.
   * @param {Uint16Array} hexes
   */
  highlight(hexes) {
    this.highlighted = [];
    for (let i = 0; i < hexes.length; i++) {
      const flags = hexes[i] >> 8;
      if (flags === 0) {
        continue;
      }

      const q = (hexes[i] & 0xf0) >> 4;
      const r = hexes[i] & 0xf;
      if (!this.isInBounds(q, r)) {
        continue;
      }

      const effects = [];
      for (let j = 0; j < EFFECTS.length; j++) {
        if ((flags & (1 << j)) > 0) {
          effects.push(EFFECTS[j]);
        }
      }
      this.highlighted.push([q, r, effects]);
    }
  }

  /**
   * Sets the pieces in the board.
   * @param {Uint16Array} pieces
   */
  setPieces(pieces) {
    this.pieces = [];
    for (let i = 0; i < pieces.length; i++) {
      const dark = (pieces[i] & 0x800) >> 11;
      const piece = (pieces[i] & 0x700) >> 8;
      const q = (pieces[i] & 0xf0) >> 4;
      const r = pieces[i] & 0xf;
      this.pieces.push({
        light: dark === 0,
        q: q,
        r: r,
        kind: PIECES[piece],
      });
    }
  }

  /**
   * Moves pieces in the board.
   * @param {Uint16Array} pieces
   */
  movePieces(pieces) {
    for (let i = 0; i < pieces.length; i++) {
      const idx = pieces[i] >> 8;

      const piece = this.pieces[idx];
      if (!piece) continue;
      piece.q = (pieces[i] & 0xf0) >> 4;
      piece.r = pieces[i] & 0xf;
    }
  }

  /**
   * Checks whether or not a hexagon is within board bounds.
   * @param {number} q
   * @param {number} r
   * @returns {boolean} validity
   */
  isInBounds(q, r) {
    if (q < 0 || r < 0) {
      return false;
    }
    if (q >= this.size || r >= this.size) {
      return false;
    }
    if (r < Math.floor(this.size / 2) - q) {
      // Top left corner is out of bounds.
      // We are in a hexagon.
      return false;
    }
    if (r > Math.floor(this.size * 1.5) - q - 1) {
      // Bottom right corner is out of bounds.
      // We are in a hexagon.
      return false;
    }
    return true;
  }

  /**
   * @param {number} q
   * @param {number} r
   */
  onClick(q, r) {
    ctx.hexClicked(q, r);
  }

  /**
   * @param {CanvasRenderingContext2D} ctx
   */
  render(ctx) {
    if (this.resized) {
      this.resized = false;

      for (const [name, img] in Object.entries(this.assets.assets)) {
        if (name.startsWith("hex_") || name.startsWith("./assets/piece_")) {
          img.width = this.hexSize;
          img.height = this.hexSize;
        }
      }
    }

    const size = this.hexSize * 2;

    const mouse = getMouseState();
    const [mouseQ, mouseR] = this._getHex(mouse.pos.x, mouse.pos.y);

    if (mouse.state === 2) {
      const [endQ, endR] = this._getHex(mouse.end.x, mouse.end.y);
      if (this.isInBounds(endQ, endR) && endQ === mouseQ && endR === mouseR) {
        // Only trigger click event if mouse didn't move out of hex
        // during click
        this.onClick(mouseQ, mouseR);
      }
    }

    for (let q = 0; q < this.size; q++) {
      for (let r = 0; r < this.size; r++) {
        if (!this.isInBounds(q, r)) {
          continue;
        }

        const [x, y] = this._getPixel(q, r);
        const color = (q + r * 2 + 1) % 3;

        ctx.drawImage(this.assets.get(`hex_${color}`), x, y, size, size);
      }
    }

    for (let i = 0; i < this.highlighted.length; i++) {
      const [q, r, effects] = this.highlighted[i];
      const [x, y] = this._getPixel(q, r);
      for (let j = 0; j < effects.length; j++) {
        ctx.drawImage(
          this.assets.get(`hex_effect_${effects[j]}`),
          x,
          y,
          size,
          size,
        );
      }
    }

    const pSize = size * 0.75;
    const offset = (size - pSize) / 2;
    for (let i = 0; i < this.pieces.length; i++) {
      const piece = this.pieces[i];
      if (!this.isInBounds(piece.q, piece.r)) {
        continue;
      }

      const [x, y] = this._getPixel(piece.q, piece.r);
      const light = piece.light ? "l" : "d";
      const asset = this.assets.get(`./assets/piece_${piece.kind}${light}.svg`);

      ctx.drawImage(asset, offset + x, offset + y, pSize, pSize);
    }

    if (this.isInBounds(mouseQ, mouseR)) {
      let effect;
      if (mouse.state == 0) {
        effect = "hex_effect_hover";
      } else {
        effect = "hex_effect_click";
      }

      const [x, y] = this._getPixel(mouseQ, mouseR);
      ctx.drawImage(this.assets.get(effect), x, y, size, size);
    }
  }
}
