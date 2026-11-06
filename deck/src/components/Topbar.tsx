import type { Signal } from "@preact/signals";
import { batch } from "@preact/signals";
import { useContext } from "preact/hooks";
import { BoardState } from "../boardState";
import { emptyBoard, removeSelection } from "../features/board/_models/board";
import type { Selection } from "../features/board/_models/selection";
import {
	emptySelection,
	isEmptySelection,
} from "../features/board/_models/selection";
import {
	downloadBoardPng,
	downloadBoardSvg,
} from "../features/imageExport/download";
import { encodeDslToParam } from "../features/share/_models/codec";
import { parseRoomId } from "../features/share/_models/roomId";
import { postRooms } from "../openapi/nagiboard";
import { showToast } from "./toast";

export const Topbar = ({
	selection,
	onToggleDslPanel,
	onShowRules,
}: {
	selection: Signal<Selection>;
	onToggleDslPanel: () => void;
	onShowRules: () => void;
}) => {
	const { board, dslText, roomId } = useContext(BoardState);

	const share = async () => {
		try {
			const res = await postRooms({
				data: await encodeDslToParam(dslText.peek()),
			});
			if (res.status !== 201) {
				showToast(
					res.data.diagnostics?.[0]?.message ??
						res.data.detail ??
						"ボードを共有できませんでした",
					"error",
				);
				return;
			}
			const url = new URL(location.href);
			url.searchParams.set("room", res.data.id);
			roomId.value = parseRoomId(res.data.id);
			history.replaceState(null, "", url);
			const shareUrl = new URL(res.data.shareUrl, location.href).href;
			try {
				await navigator.clipboard.writeText(shareUrl);
				showToast("共有リンクをコピーしました");
			} catch {
				showToast(`共有リンク: ${shareUrl}`);
			}
		} catch {
			showToast("ボードを共有できませんでした", "error");
		}
	};

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
				onInput={(ev) => {
					board.value = { ...board.peek(), title: ev.currentTarget.value };
				}}
			/>
			<div class="spacer" />
			<button type="button" class="btn" onClick={() => void share()}>
				共有
			</button>
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
					const current = selection.peek();
					if (isEmptySelection(current)) {
						showToast("削除する選択がありません");
						return;
					}
					batch(() => {
						board.value = removeSelection(board.peek(), current);
						selection.value = emptySelection;
					});
				}}
			>
				選択を削除
			</button>
			<button
				type="button"
				class="btn"
				onClick={() => {
					board.value = { ...emptyBoard, title: board.peek().title };
					showToast(
						"全消去しました(DSLテキストを貼り直して適用すれば復元できます)",
					);
				}}
			>
				全消去
			</button>
			<button type="button" class="btn" onClick={onToggleDslPanel}>
				DSLパネル
			</button>
		</header>
	);
};
