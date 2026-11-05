import type { Board, EdgeKey } from "./board";
import { edgeKey } from "./board";
import type { NoteId } from "./noteId";

export type Selection = {
	readonly noteIds: ReadonlySet<NoteId>;
	readonly edgeKeys: ReadonlySet<EdgeKey>;
};

export const emptySelection: Selection = {
	noteIds: new Set(),
	edgeKeys: new Set(),
};

export const onlyNote = (id: NoteId): Selection => ({
	noteIds: new Set([id]),
	edgeKeys: new Set(),
});

export const onlyEdge = (key: EdgeKey): Selection => ({
	noteIds: new Set(),
	edgeKeys: new Set([key]),
});

export const withNotes = (ids: Iterable<NoteId>): Selection => ({
	noteIds: new Set(ids),
	edgeKeys: new Set(),
});

export const withToggledNote = (
	selection: Selection,
	id: NoteId,
): Selection => ({
	noteIds: selection.noteIds.has(id)
		? new Set([...selection.noteIds].filter((x) => x !== id))
		: new Set([...selection.noteIds, id]),
	edgeKeys: selection.edgeKeys,
});

export const withToggledEdge = (
	selection: Selection,
	key: EdgeKey,
): Selection => ({
	noteIds: selection.noteIds,
	edgeKeys: selection.edgeKeys.has(key)
		? new Set([...selection.edgeKeys].filter((x) => x !== key))
		: new Set([...selection.edgeKeys, key]),
});

export const isEmptySelection = (selection: Selection): boolean =>
	!selection.noteIds.size && !selection.edgeKeys.size;

export const withinBoard = (selection: Selection, board: Board): Selection => {
	const notes = new Set(board.notes.map((n) => n.id));
	const edges = new Set(board.edges.map(edgeKey));
	const noteIds = [...selection.noteIds].filter((id) => notes.has(id));
	const edgeKeys = [...selection.edgeKeys].filter((key) => edges.has(key));
	return noteIds.length === selection.noteIds.size &&
		edgeKeys.length === selection.edgeKeys.size
		? selection
		: { noteIds: new Set(noteIds), edgeKeys: new Set(edgeKeys) };
};
