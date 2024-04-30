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

const spinner = document.getElementById("loading");
const menus = [];

/**
 * @type {"canvas" | "loading" | Modal}
 */
let current = "loading";

// Set up modals
["gamemode", "register", "online", "settings"].forEach((name, idx) => {
  const modal = new Modal(`#menu-${name}`);
  menus.push(modal);

  const element = document.getElementById(`menu-${name}`);
  const btn = element.getElementsByClassName("continue-btn")[0];

  if (btn !== undefined) {
    btn.addEventListener("click", () => {
      if (name === "register") {
        const username = document.getElementById("username").value;
        setPlayerName("self", username);
        registered(ctx, username);
      } else if (name === "online") {
        create_room(ctx);
      } else if (name === "settings") {
        let timer = parseInt(document.getElementById("game-timer").value, 10);
        if (!timer) {
          timer = null;
        }

        let light = document.getElementById("start-color").value;
        if (light === "light") {
          light = true;
        } else if (light === "dark") {
          light = false;
        } else {
          light = Math.random() < 0.5;
        }

        set_settings(ctx, timer, light);
      }
    });
  }

  element.addEventListener("hidden.bs.modal", () => {
    // Only trigger if closed by user (not by js/rust)
    if (current === modal) {
      on_menu_hidden(ctx, idx);
    }
  });
});

// Add click listener to game mode buttons
["local", "online", "bot"].forEach((name, idx) => {
  const element = document.getElementById(`mode-${name}`);
  element.addEventListener("click", () => {
    set_gamemode(ctx, idx);
    document.getElementById("start-color-group").hidden = name === "local";
  });
});

const joinBtn = document.getElementById("join-room-btn");
const joinError = document.getElementById("join-error");
const joinSpinner = joinBtn.getElementsByClassName("spinner-border")[0];
joinBtn.addEventListener("click", () => {
  joinBtn.disabled = true;
  joinError.hidden = true;
  joinSpinner.hidden = false;

  const code = document.getElementById("join-room").value;
  join_room(ctx, code);
});

export const setJoinResponse = (success) => {
  joinBtn.disabled = false;
  joinError.hidden = success;
  joinSpinner.hidden = true;

  if (success) return;
  setTimeout(() => (joinError.hidden = true), 5000);
};

export const setPlayerName = (target, name) => {
  const list = document.querySelectorAll(`[data-name=${target}]`);
  for (const element of list) {
    element.textContent = name;
  }

  if (target === "self") {
    document.getElementById("username").value = name;
  }
};

const hideCurrent = () => {
  if (current === "canvas") {
    hide();
  } else if (current === "loading") {
    spinner.classList.add("invisible");
  } else {
    current.hide();
  }
};

export const showCanvas = () => {
  if (current === "canvas") return;

  hideCurrent();
  current = "canvas";
  show();
};

export const showLoading = () => {
  if (current === "loading") return;

  hideCurrent();
  current = "loading";
  spinner.classList.remove("invisible");
};

export const showMenu = (idx) => {
  if (current === menus[idx]) return;

  hideCurrent();
  current = menus[idx];
  current.show();
};
