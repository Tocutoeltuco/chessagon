const evtTarget = document.getElementById("menu-events");
export const menu = document.getElementById("menu-online");

const joinForm = menu.querySelector("[data-online=join]");
const joinBtn = joinForm.querySelector("button");
const codeInput = joinForm.querySelector("input");
const joinSpinner = joinBtn.querySelector(".spinner-border");
const joinErr = menu.querySelector("[data-online=error]");
const createBtn = menu.querySelector("[data-online=create]");
const createSpinner = createBtn.querySelector(".spinner-border");
const backBtn = menu.querySelector(".btn-secondary");
let timeout;

const params = new URLSearchParams(location.search);

let readCode = false;
menu.addEventListener("show.bs.modal", () => {
  if (readCode) return;
  readCode = true;

  const code = params.get("code");
  if (!code) return;

  codeInput.value = code;
  joinForm.requestSubmit();
});

joinForm.addEventListener("submit", (evt) => {
  evt.preventDefault();
  if (joinBtn.disabled) return;

  backBtn.disabled = true;
  createBtn.disabled = true;
  joinBtn.disabled = true;
  joinErr.hidden = true;
  joinSpinner.hidden = false;

  clearTimeout(timeout);
  timeout = setTimeout(() => joinResponse("timeout"), 15000);

  const code = codeInput.value;
  evtTarget.dispatchEvent(new CustomEvent("chess.join", { detail: code }));
});

createBtn.addEventListener("click", (evt) => {
  evt.preventDefault();
  if (createBtn.disabled) return;

  backBtn.disabled = true;
  createBtn.disabled = true;
  joinBtn.disabled = true;
  joinErr.hidden = true;
  createSpinner.hidden = false;

  clearTimeout(timeout);
  timeout = setTimeout(() => joinResponse("timeout"), 15000);

  evtTarget.dispatchEvent(new Event("chess.create"));
});

/**
 * @param {"success" | string} resp
 */
export const joinResponse = (resp) => {
  backBtn.disabled = false;
  createBtn.disabled = false;
  joinBtn.disabled = false;
  joinErr.hidden = resp === "success";
  joinSpinner.hidden = true;
  createSpinner.hidden = true;

  clearTimeout(timeout);

  if (resp === "success") return;

  for (const text of joinErr.querySelectorAll("[data-online-error]")) {
    text.hidden = text.dataset.onlineError !== resp;
  }

  timeout = setTimeout(() => joinResponse("success"), 5000);
};
