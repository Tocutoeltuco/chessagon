const evtTarget = document.getElementById("menu-events");
export const menu = document.getElementById("menu-register");

const form = menu.querySelector("[data-reg=form]");
const input = form.querySelector("input");

form.addEventListener("submit", (evt) => {
  evt.preventDefault();

  const name = input.value;
  evtTarget.dispatchEvent(new CustomEvent("chess.register", { detail: name }));
});

/**
 * @param {string} name
 */
export const setName = (name) => {
  input.value = name;
};
