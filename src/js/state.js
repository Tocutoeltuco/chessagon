import { setup, JsEvent } from "../../pkg/index";
import { AssetManager } from "./assets";
import { Board } from "./board";

export const assets = new AssetManager();
export const board = new Board(11, assets);

const wasm = setup();
class JsContext {
  constructor() {
    this.text = new TextEncoder();
  }

  start() {
    wasm.dispatch_empty(JsEvent.Start);
  }

  setGamemode(mode) {
    const buf = Uint8Array.from([mode]);
    wasm.dispatch(JsEvent.SetGamemode, buf);
  }

  register(name) {
    wasm.dispatch(JsEvent.Register, this.text.encode(name));
  }

  createRoom() {
    wasm.dispatch_empty(JsEvent.CreateRoom);
  }

  joinRoom(code) {
    wasm.dispatch(JsEvent.JoinRoom, this.text.encode(code));
  }

  setSettings(time, hostAsLight) {
    const buf = new Uint8Array(3);
    buf[0] = (time >> 8) & 0xff;
    buf[1] = time & 0xff;
    buf[2] = hostAsLight ? 1 : 0;
    wasm.dispatch(JsEvent.SetSettings, buf);
  }

  timerExpired() {
    wasm.dispatch_empty(JsEvent.TimerExpired);
  }

  sendMessage(msg) {
    wasm.dispatch(JsEvent.SendMessage, this.text.encode(msg));
  }

  menuHidden(menu) {
    const buf = Uint8Array.from([menu]);
    wasm.dispatch(JsEvent.MenuHidden, buf);
  }

  hexClicked(q, r) {
    const buf = Uint8Array.from([q, r]);
    wasm.dispatch(JsEvent.HexClicked, buf);
  }

  gameButtonClick(id) {
    wasm.dispatch(JsEvent.GameButtonClick, new Uint8Array([id]));
  }
}

export const ctx = new JsContext();
