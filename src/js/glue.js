import { board } from "./state";

export { showCanvas, showLoading, showMenu } from "./menus";

export const setPieces = (pieces) => board.setPieces(pieces);
export const movePieces = (pieces) => board.movePieces(pieces);
export const highlight = (hexes) => board.highlight(hexes);
