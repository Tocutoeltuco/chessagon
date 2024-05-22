const evtTarget = document.getElementById("menu-events");
export const menu = document.getElementById("menu-online");

const joinForm = menu.querySelector("[data-online=join]");
const joinBtn = joinForm.querySelector("button");
const codeInput = joinForm.querySelector("input");
const spinner = joinBtn.querySelector(".spinner-border");
const joinErr = menu.querySelector("[data-online=error]");
const createBtn = menu.querySelector("[data-online=create]");
let timeout;

joinForm.addEventListener("submit", (evt) => {
  evt.preventDefault();
  if (joinBtn.disabled) return;

  joinBtn.disabled = true;
  joinErr.hidden = true;
  spinner.hidden = false;

  clearTimeout(timeout);
  timeout = setTimeout(() => joinResponse("timeout"), 15000);

  const code = codeInput.value;
  evtTarget.dispatchEvent(new CustomEvent("chess.join", { detail: code }));
});

createBtn.addEventListener("click", () => {
  evtTarget.dispatchEvent(new Event("chess.create"));
});

/**
 * @param {"success" | string} resp
 */
export const joinResponse = (resp) => {
  joinBtn.disabled = false;
  joinErr.hidden = resp === "success";
  spinner.hidden = true;

  clearTimeout(timeout);

  if (resp === "success") return;

  for (const text of joinErr.querySelectorAll("[data-online-error]")) {
    text.hidden = text.dataset.onlineError !== resp;
  }

  timeout = setTimeout(() => joinResponse("success"), 5000);
};
