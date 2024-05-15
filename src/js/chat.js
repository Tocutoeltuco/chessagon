import { ctx } from "./state";

const chat = document.getElementById("chat");
const container = chat.querySelector(".messages");
const light = chat.querySelector("[data-template=light]");
const dark = chat.querySelector("[data-template=dark]");
const inp = chat.querySelector("input");

window.onkeyup = (e) => {
  if (e.code === "Enter" || e.code === "NumpadEnter") {
    inp.focus();
  }
};

inp.onkeyup = (e) => {
  if (e.code === "Enter" || e.code === "NumpadEnter") {
    e.stopPropagation();
  }
};

inp.onkeydown = (e) => {
  if (e.code === "Enter" || e.code === "NumpadEnter") {
    if (inp.value === "") return;
    ctx.sendMessage(inp.value);
    inp.value = "";
  }
};

/**
 * @param {"light" | "dark"} type
 * @param {string} name
 * @param {string} content
 */
export const addChatMessage = (type, name, content) => {
  let template;
  if (type === "light") {
    template = light.cloneNode(true);
  } else {
    template = dark.cloneNode(true);
  }

  const nameSlot = template.querySelector("[data-slot=name]");
  const contentSlot = template.querySelector("[data-slot=content]");
  nameSlot.innerText = name;
  contentSlot.innerText = content;

  container.insertBefore(
    template.querySelector(".message"),
    container.querySelector(".message"),
  );
};
