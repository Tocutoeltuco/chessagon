import { create_context } from "../../pkg/index";
import { AssetManager } from "./assets";
import { Board } from "./board";

export const ctx = create_context();
export const assets = new AssetManager();
export const board = new Board(11, assets);
