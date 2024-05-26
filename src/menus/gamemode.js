const evtTarget = document.getElementById("menu-events");
export const menu = document.getElementById("menu-gamemode");

const gamemodes = menu.querySelectorAll("[data-gamemode]:not([disabled])");

const params = new URLSearchParams(location.search);

let readCode = false;
menu.addEventListener("show.bs.modal", (evt) => {
  if (readCode) return;
  readCode = true;

  if (location.search !== "") {
    // Replace after first modal shows up
    history.replaceState(null, "", "/");
  }
  if (!params.get("code")) return;

  evt.preventDefault();
  evtTarget.dispatchEvent(
    new CustomEvent("chess.gamemode", { detail: "online" }),
  );
});

for (const btn of gamemodes) {
  btn.addEventListener("click", () => {
    const evt = new CustomEvent("chess.gamemode", {
      detail: btn.dataset.gamemode,
    });
    evtTarget.dispatchEvent(evt);
  });
}
