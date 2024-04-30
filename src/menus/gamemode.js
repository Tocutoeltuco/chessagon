const evtTarget = document.getElementById("menu-events");
export const menu = document.getElementById("menu-gamemode");

const gamemodes = menu.querySelectorAll("[data-gamemode]:not([disabled])");

for (const btn of gamemodes) {
  btn.addEventListener("click", () => {
    const evt = new CustomEvent("chess.gamemode", {
      detail: btn.dataset.gamemode,
    });
    evtTarget.dispatchEvent(evt);
  });
}
