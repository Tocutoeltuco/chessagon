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
    const buf = Uint16Array.from([time, hostAsLight ? 256 : 0]);
    wasm.dispatch(JsEvent.SetSettings, new Uint8Array(buf.buffer));
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
}

export const ctx = new JsContext();
