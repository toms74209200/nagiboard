import { render } from "preact";
import { App } from "./app.tsx";
import { BoardState, createBoardState } from "./boardState.ts";
import css from "./index.css?inline";

const style = document.createElement("style");
style.textContent = css;
document.head.append(style);

const root = document.getElementById("app");
if (root) {
	render(
		<BoardState.Provider value={createBoardState()}>
			<App />
		</BoardState.Provider>,
		root,
	);
}
