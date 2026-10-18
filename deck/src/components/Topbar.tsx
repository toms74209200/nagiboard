import {
	board,
	clearBoard,
	deleteSelection,
	dslPanelVisible,
	setTitle,
} from "../features/board/store";
import {
	downloadBoardPng,
	downloadBoardSvg,
} from "../features/imageExport/download";
import { showToast } from "./toast";

export const Topbar = ({ onShowRules }: { onShowRules: () => void }) => {
	const exportImage = async (kind: "svg" | "png") => {
		try {
			const ok =
				kind === "svg"
					? downloadBoardSvg(board.peek())
					: await downloadBoardPng(board.peek());
			if (!ok) showToast("エクスポートする付箋がありません");
		} catch {
			showToast("画像のエクスポートに失敗しました", "error");
		}
	};

	return (
		<header id="topbar">
			<div class="logo">Event Storming Board</div>
			<input
				id="boardTitle"
				placeholder="ボード名 (title)"
				value={board.value.title}
				onInput={(ev) => setTitle(ev.currentTarget.value)}
			/>
			<div class="spacer" />
			<button type="button" class="btn" onClick={onShowRules}>
				接続ルール
			</button>
			<button type="button" class="btn" onClick={() => void exportImage("svg")}>
				SVG
			</button>
			<button type="button" class="btn" onClick={() => void exportImage("png")}>
				PNG
			</button>
			<button
				type="button"
				class="btn"
				onClick={() => {
					if (!deleteSelection()) showToast("削除する選択がありません");
				}}
			>
				選択を削除
			</button>
			<button
				type="button"
				class="btn"
				onClick={() => {
					clearBoard();
					showToast(
						"全消去しました(DSLテキストを貼り直して適用すれば復元できます)",
					);
				}}
			>
				全消去
			</button>
			<button
				type="button"
				class="btn"
				onClick={() => {
					dslPanelVisible.value = !dslPanelVisible.value;
				}}
			>
				DSLパネル
			</button>
		</header>
	);
};
