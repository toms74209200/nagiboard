import { effect } from "@preact/signals";
import { showToast } from "../../components/toast";
import { getBoard, getRoomBoards } from "../../openapi/nagiboard";
import { emptyBoard } from "../board/_models/board";
import { loadBoard } from "../board/store";
import { parseDsl, serializeDsl } from "../dsl/_models/dsl";
import { dslText } from "../dsl/dslText";
import { boardSourceOf } from "./_models/boardSource";
import { decodeParamToDsl, encodeDslToParam } from "./_models/codec";

const URL_PARAM = "data";
const URL_LENGTH_WARNING = 8000;
const WRITE_DEBOUNCE_MS = 300;

let suspendedBaseline: string | null = null;

const loadDsl = (text: string, errorPrefix: string): void => {
	const result = parseDsl(text);
	if (result.errors.length) {
		suspendedBaseline = dslText.peek();
		showToast(`${errorPrefix}: ${result.errors[0]}`, "error");
		return;
	}
	loadBoard(result.board);
	if (result.coerced) {
		showToast(
			`ルール外の矢印 ${result.coerced} 本を点線(例外)として読み込みました`,
		);
	}
};

const restoreBoardFromData = async (param: string): Promise<void> => {
	try {
		loadDsl(await decodeParamToDsl(param), "URLの復元でエラー");
	} catch {
		suspendedBaseline = dslText.peek();
		showToast("URLからボードを復元できませんでした", "error");
	}
};

const restoreBoardFromRoom = async (id: string): Promise<void> => {
	try {
		const room = await getRoomBoards(id);
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
		loadDsl(shared.data, "共有ボードの読み込みでエラー");
	} catch {
		showToast("共有ボードを読み込めませんでした", "error");
	}
};

export const restoreBoardFromUrl = async (): Promise<void> => {
	const source = boardSourceOf(location.search);
	if (source.kind === "data") await restoreBoardFromData(source.param);
	if (source.kind === "room") await restoreBoardFromRoom(source.id);
};

const EMPTY_DSL = serializeDsl(emptyBoard);

let writeTimer: ReturnType<typeof setTimeout> | undefined;
let warnedAboutLength = false;

const writeUrl = async (text: string): Promise<void> => {
	const url = new URL(location.href);
	if (text === EMPTY_DSL) {
		url.searchParams.delete(URL_PARAM);
	} else {
		url.searchParams.set(URL_PARAM, await encodeDslToParam(text));
	}
	history.replaceState(null, "", url);
	if (!warnedAboutLength && url.href.length > URL_LENGTH_WARNING) {
		warnedAboutLength = true;
		showToast("URLが長くなっています。共有にはファイル保存も検討してください");
	}
};

export const startUrlSync = (): void => {
	effect(() => {
		const text = dslText.value;
		if (suspendedBaseline !== null) {
			if (text === suspendedBaseline) return;
			suspendedBaseline = null;
		}
		clearTimeout(writeTimer);
		writeTimer = setTimeout(() => void writeUrl(text), WRITE_DEBOUNCE_MS);
	});
};
