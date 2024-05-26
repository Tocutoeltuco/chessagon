import { board } from "./state";

export { setScene, setPlayerName } from "./scene";
export { joinResponse } from "../menus/online.js";
export { addChatMessage, showChat, hideChat } from "./chat.js";
export { addRTT } from "./render.js";
export { showButtons } from "./buttons.js";

export const setPieces = (pieces) => board.setPieces(pieces);
export const movePieces = (pieces) => board.movePieces(pieces);
export const highlight = (hexes) => board.highlight(hexes);
export const promotePieces = (pieces) => board.promotePieces(pieces);
export const showPromotionPrompt = (color, q, r) =>
  board.showPromotionPrompt(color, q, r);
export const removeTimers = () => board.hideTimers();
export const setTimers = (light, dark, active) =>
  board.setTimers(light, dark, active);
export const setBoardPerspective = (isLight) => board.flip(!isLight);
