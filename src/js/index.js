import "../scss/styles.scss";
import { onReady } from "./loader";
import { ctx } from "./state";

onReady().then(() => ctx.start());
