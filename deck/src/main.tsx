import { render } from "preact";
import { App } from "./app.tsx";
import { restoreBoardFromUrl, startUrlSync } from "./features/share/urlSync.ts";
import css from "./index.css?inline";

const style = document.createElement("style");
style.textContent = css;
document.head.append(style);

const boot = async (): Promise<void> => {
	await restoreBoardFromUrl();
	startUrlSync();
	const root = document.getElementById("app");
	if (root) {
		render(<App />, root);
	}
};

void boot();
