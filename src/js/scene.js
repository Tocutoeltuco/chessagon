import { Modal } from "bootstrap";
import { ctx } from "./state";
import {
  create_room,
  join_room,
  on_menu_hidden,
  registered,
  set_gamemode,
  set_settings,
} from "../../pkg";
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
      on_menu_hidden(ctx, idx);
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

  set_gamemode(ctx, idx);
});
menuEvt.addEventListener("chess.register", (evt) => {
  setPlayerName(true, evt.detail);
  registered(ctx, evt.detail);
});
menuEvt.addEventListener("chess.join", (evt) => join_room(ctx, evt.detail));
menuEvt.addEventListener("chess.create", () => create_room(ctx));
menuEvt.addEventListener("chess.settings", (evt) => {
  set_settings(ctx, evt.detail.timer, evt.detail.start === "light");
});
