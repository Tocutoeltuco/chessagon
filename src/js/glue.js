import { board } from "./state";

export { setScene, setPlayerName } from "./scene";
export { joinResponse } from "../menus/online.js";
export { addChatMessage } from "./chat.js";
export { setTimers, removeTimers, addRTT } from "./render.js";

export const setPieces = (pieces) => board.setPieces(pieces);
export const movePieces = (pieces) => board.movePieces(pieces);
export const highlight = (hexes) => board.highlight(hexes);
