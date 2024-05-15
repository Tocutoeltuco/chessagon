const evtTarget = document.getElementById("menu-events");
export const menu = document.getElementById("menu-settings");

const timer = menu.querySelector("[data-sett=timer]");
const startGroup = menu.querySelector("[data-sett=start]");
const start = startGroup.querySelector("select");
const nextBtn = menu.querySelector("[data-sett=continue]");

evtTarget.addEventListener("chess.gamemode", (evt) => {
  startGroup.hidden = evt.detail === "local";
});

nextBtn.addEventListener("click", () => {
  const time = parseInt(timer.value, 10) || 0;
  let startColor = start.value;

  if (startColor === "random") {
    startColor = Math.random() < 0.5 ? "light" : "dark";
  }

  evtTarget.dispatchEvent(
    new CustomEvent("chess.settings", {
      detail: {
        timer: time,
        start: startColor,
      },
    }),
  );
});
