import { ctx } from "./state";

const MAX_MESSAGES = 50;

const chat = document.getElementById("chat");
const container = chat.querySelector(".messages");
const inp = chat.querySelector("input");

window.onkeyup = (e) => {
  if (inp.hidden) return;

  if (e.code === "Enter" || e.code === "NumpadEnter") {
    inp.focus();
  }
};

inp.onkeyup = (e) => {
  if (inp.hidden) return;

  if (e.code === "Enter" || e.code === "NumpadEnter") {
    e.stopPropagation();
  }
};

inp.onkeydown = (e) => {
  if (inp.hidden) return;

  if (e.code === "Enter" || e.code === "NumpadEnter") {
    if (inp.value === "") return;
    ctx.sendMessage(inp.value);
    inp.value = "";
  }
};

const templates = [
  "light",
  "dark",
  "join",
  "start",
  "win-light",
  "win-dark",
  "expired-light",
  "expired-dark",
  "resign-light",
  "resign-dark",
];

/**
 * @param {number} kind
 * @param {string[]} slots
 */
export const addChatMessage = (kind, slots) => {
  const template = chat
    .querySelector(`[data-template=${templates[kind]}]`)
    .cloneNode(true);

  for (let i = 0; i < slots.length; i++) {
    const slot = template.querySelector(`[data-slot="${i}"]`);
    slot.innerText = slots[i];
  }

  container.insertBefore(
    template.querySelector(".message"),
    container.querySelector(".message"),
  );

  if (container.childElementCount > MAX_MESSAGES) {
    const last = container.children[container.children.length - 1];
    container.removeChild(last);
  }
};

export const showChat = () => {
  inp.hidden = false;
};

export const hideChat = () => {
  inp.hidden = true;
};
