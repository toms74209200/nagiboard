import { useSignal, useSignalEffect } from "@preact/signals";
import { useContext, useEffect, useMemo, useState } from "preact/hooks";
import { BoardState } from "./boardState";
import { Topbar } from "./components/Topbar";
import { showToast } from "./components/toast";
import { emptyBoard } from "./features/board/_models/board";
import { emptySelection } from "./features/board/_models/selection";
import { BoardCanvas } from "./features/board/BoardCanvas";
import { RulesDialog, useRulesDialog } from "./features/board/RulesDialog";
import { parseDsl, serializeDsl } from "./features/dsl/_models/dsl";
import { DslPanel } from "./features/dsl/DslPanel";
import { boardSourceOf } from "./features/share/_models/boardSource";
import {
	decodeParamToDsl,
	encodeDslToParam,
} from "./features/share/_models/codec";
import { getBoard, getRoomBoards } from "./openapi/nagiboard";

const URL_PARAM = "data";
const URL_LENGTH_WARNING = 8000;
const WRITE_DEBOUNCE_MS = 300;
const EMPTY_DSL = serializeDsl(emptyBoard);

export const App = () => {
	const { board, dslText } = useContext(BoardState);
	const rulesDialog = useRulesDialog();
	const [dslPanelVisible, setDslPanelVisible] = useState(true);
	const selection = useSignal(emptySelection);
	const source = useMemo(() => boardSourceOf(location.search), []);
	const restored = useSignal(source.kind === "empty");
	const suspendedBaseline = useSignal<string | null>(null);
	const warnedAboutLength = useSignal(false);

	useEffect(() => {
		if (source.kind !== "data") return;
		decodeParamToDsl(source.param)
			.then((text) => {
				const result = parseDsl(text);
				if (result.errors.length) {
					suspendedBaseline.value = dslText.peek();
					showToast(`URLの復元でエラー: ${result.errors[0]}`, "error");
					return;
				}
				board.value = result.board;
				if (result.coerced) {
					showToast(
						`ルール外の矢印 ${result.coerced} 本を点線(例外)として読み込みました`,
					);
				}
			})
			.catch(() => {
				suspendedBaseline.value = dslText.peek();
				showToast("URLからボードを復元できませんでした", "error");
			})
			.finally(() => {
				restored.value = true;
			});
	}, [board, dslText, restored, source, suspendedBaseline]);

	useEffect(() => {
		if (source.kind !== "room") return;
		getRoomBoards(source.id)
			.then(async (room) => {
				if (room.status === 404) {
					showToast("共有ルームが見つかりません", "error");
					return;
				}
				if (room.status !== 200) {
					showToast("共有ボードを読み込めませんでした", "error");
					return;
				}
				const [boardId] = room.data.boards;
				const shared = boardId ? await getBoard(boardId) : null;
				if (!shared || shared.status === 404) {
					showToast("共有ボードが見つかりません", "error");
					return;
				}
				if (shared.status !== 200) {
					showToast("共有ボードを読み込めませんでした", "error");
					return;
				}
				const result = parseDsl(shared.data);
				if (result.errors.length) {
					suspendedBaseline.value = dslText.peek();
					showToast(
						`共有ボードの読み込みでエラー: ${result.errors[0]}`,
						"error",
					);
					return;
				}
				board.value = result.board;
				if (result.coerced) {
					showToast(
						`ルール外の矢印 ${result.coerced} 本を点線(例外)として読み込みました`,
					);
				}
			})
			.catch(() => showToast("共有ボードを読み込めませんでした", "error"))
			.finally(() => {
				restored.value = true;
			});
	}, [board, dslText, restored, source, suspendedBaseline]);

	useSignalEffect(() => {
		if (!restored.value) return;
		const text = dslText.value;
		if (text === suspendedBaseline.peek()) return;
		suspendedBaseline.value = null;
		const timer = setTimeout(() => {
			void encodeDslToParam(text).then((param) => {
				const url = new URL(location.href);
				if (text === EMPTY_DSL) {
					url.searchParams.delete(URL_PARAM);
				} else {
					url.searchParams.set(URL_PARAM, param);
				}
				history.replaceState(null, "", url);
				if (!warnedAboutLength.peek() && url.href.length > URL_LENGTH_WARNING) {
					warnedAboutLength.value = true;
					showToast(
						"URLが長くなっています。共有にはファイル保存も検討してください",
					);
				}
			});
		}, WRITE_DEBOUNCE_MS);
		return () => clearTimeout(timer);
	});

	if (!restored.value) return null;

	return (
		<div id="layout" class={dslPanelVisible ? "" : "dsl-hidden"}>
			<Topbar
				selection={selection}
				onToggleDslPanel={() => setDslPanelVisible((visible) => !visible)}
				onShowRules={rulesDialog.show}
			/>
			<BoardCanvas selection={selection} />
			<DslPanel />
			<RulesDialog dialogRef={rulesDialog.dialogRef} />
		</div>
	);
};
