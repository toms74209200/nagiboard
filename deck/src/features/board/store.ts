import { batch, signal } from "@preact/signals";
import type {
	Board,
	Clipboard,
	ConnectResult,
	Edge,
	Note,
} from "./_models/board";
import {
	addNote,
	clipSelection,
	connect,
	edgeKey,
	emptyBoard,
	findAdjacentConnection,
	pasteClipboard,
	removeSelection,
	setNoteText,
} from "./_models/board";
import type { Point } from "./_models/geometry";
import { snapToGrid } from "./_models/geometry";
import type { NoteId } from "./_models/noteId";
import type { NoteType } from "./_models/noteType";
import { NOTE_TYPES } from "./_models/noteType";
import type { Selection } from "./_models/selection";
import {
	emptySelection,
	isEmptySelection,
	onlyEdge,
	onlyNote,
	withNotes,
} from "./_models/selection";

export type ViewTransform = {
	readonly x: number;
	readonly y: number;
	readonly scale: number;
};
export type EditingNote = { readonly id: NoteId; readonly selectAll: boolean };

export const board = signal<Board>(emptyBoard);
export const selection = signal<Selection>(emptySelection);
export const viewTransform = signal<ViewTransform>({ x: 60, y: 20, scale: 1 });
export const activeTool = signal<NoteType | null>(null);
export const editingNote = signal<EditingNote | null>(null);
export const dslPanelVisible = signal(true);

export const loadBoard = (next: Board): void => {
	batch(() => {
		board.value = next;
		selection.value = emptySelection;
		editingNote.value = null;
	});
};

export const setTitle = (title: string): void => {
	board.value = { ...board.value, title };
};

export const clearBoard = (): void => {
	loadBoard({ ...emptyBoard, title: board.value.title });
};

export const placeNote = (
	type: NoteType,
	centerX: number,
	centerY: number,
): { note: Note; autoEdge: Edge | null } => {
	const meta = NOTE_TYPES[type];
	const added = addNote(
		board.value,
		type,
		snapToGrid(centerX - meta.width / 2),
		snapToGrid(centerY - meta.height / 2),
	);
	const autoEdge = findAdjacentConnection(added.board, added.note);
	batch(() => {
		board.value = autoEdge
			? { ...added.board, edges: [...added.board.edges, autoEdge] }
			: added.board;
		selection.value = onlyNote(added.note.id);
	});
	return { note: added.note, autoEdge };
};

export const setNotePositions = (
	positions: ReadonlyMap<string, Point>,
): void => {
	board.value = {
		...board.value,
		notes: board.value.notes.map((n) => {
			const p = positions.get(n.id);
			return p ? { ...n, x: p.x, y: p.y } : n;
		}),
	};
};

export const connectNotes = (fromId: string, toId: string): ConnectResult => {
	const result = connect(board.value, fromId, toId);
	if (result.kind === "connected") {
		batch(() => {
			board.value = result.board;
			selection.value = onlyEdge(edgeKey(result.edge));
		});
	}
	return result;
};

export const commitNoteText = (id: string, rawText: string): void => {
	batch(() => {
		board.value = setNoteText(board.value, id, rawText);
		if (editingNote.value?.id === id) editingNote.value = null;
	});
};

export const deleteSelection = (): boolean => {
	const current = selection.value;
	if (isEmptySelection(current)) return false;
	batch(() => {
		board.value = removeSelection(board.value, current);
		selection.value = emptySelection;
	});
	return true;
};

const PASTE_OFFSET = 24;
let clipboard: Clipboard | null = null;

export const copySelectionToClipboard = (): number => {
	const ids = selection.value.noteIds;
	if (!ids.size) return 0;
	clipboard = clipSelection(board.value, ids);
	return clipboard.notes.length;
};

export const pasteFromClipboard = (): number => {
	if (!clipboard?.notes.length) return 0;
	const { board: next, pastedIds } = pasteClipboard(
		board.value,
		clipboard,
		PASTE_OFFSET,
	);
	batch(() => {
		board.value = next;
		selection.value = withNotes(pastedIds);
	});
	clipboard = {
		notes: clipboard.notes.map((n) => ({
			...n,
			x: n.x + PASTE_OFFSET,
			y: n.y + PASTE_OFFSET,
		})),
		edges: clipboard.edges,
	};
	return pastedIds.length;
};
