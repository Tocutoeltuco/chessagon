import { Modal } from "bootstrap";
import { ctx } from "./state";
import { hide, show } from "./render";

import { menu as gamemodeMenu } from "../menus/gamemode.js";
import { menu as registerMenu, setName } from "../menus/register.js";
import { menu as onlineMenu } from "../menus/online.js";
import { menu as settingsMenu } from "../menus/settings.js";

const menuEvt = document.getElementById("menu-events");
const spinner = document.getElementById("loading");

/**
 * @type {Modal[]}
 */
const menus = [];
const LOADING = -2;
const CANVAS = -1;
let currentScene = LOADING;

// Set up modals
[gamemodeMenu, registerMenu, onlineMenu, settingsMenu].forEach((menu, idx) => {
  const modal = new Modal(menu);
  menus.push(modal);

  menu.addEventListener("hidden.bs.modal", () => {
    // Only trigger if closed by user (not by js/rust)
    if (currentScene === idx) {
      ctx.menuHidden(idx);
    }
  });
});

export const setPlayerName = (isSelf, name) => {
  const target = isSelf ? "self" : "opponent";
  const list = document.querySelectorAll(`[data-name=${target}]`);
  for (const element of list) {
    element.textContent = name;
  }

  if (isSelf) {
    setName(name);
  }
};

const hideScene = () => {
  if (currentScene === CANVAS) {
    hide();
  } else if (currentScene === LOADING) {
    spinner.hidden = true;
  } else {
    menus[currentScene].hide();
  }
};

/**
 * @param {number} scene
 */
export const setScene = (scene) => {
  if (currentScene === scene) return;

  hideScene();
  currentScene = scene;

  if (scene === CANVAS) {
    show();
  } else if (scene === LOADING) {
    spinner.hidden = false;
  } else {
    menus[scene].show();
  }
};

menuEvt.addEventListener("chess.gamemode", (evt) => {
  let idx;
  if (evt.detail === "local") {
    idx = 0;
  } else if (evt.detail === "online") {
    idx = 1;
  } else {
    idx = 2;
  }

  ctx.setGamemode(idx);
});
menuEvt.addEventListener("chess.register", (evt) => {
  ctx.register(evt.detail);
});
menuEvt.addEventListener("chess.join", (evt) => ctx.joinRoom(evt.detail));
menuEvt.addEventListener("chess.create", () => ctx.createRoom());
menuEvt.addEventListener("chess.settings", (evt) => {
  ctx.setSettings(evt.detail.timer, evt.detail.start === "light");
});
