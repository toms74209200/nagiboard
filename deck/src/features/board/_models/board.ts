import type { Rect } from "./geometry";
import { intersects } from "./geometry";
import type { NoteId } from "./noteId";
import { parseNoteId } from "./noteId";
import type { NoteType } from "./noteType";
import { ID_PREFIX, isAllowedConnection, NOTE_TYPES } from "./noteType";
import type { Selection } from "./selection";

export type Note = {
	readonly id: NoteId;
	readonly type: NoteType;
	readonly text: string;
	readonly x: number;
	readonly y: number;
};

export type Edge = {
	readonly from: NoteId;
	readonly to: NoteId;
	readonly dashed: boolean;
};

export type Board = {
	readonly title: string;
	readonly notes: readonly Note[];
	readonly edges: readonly Edge[];
};

export const emptyBoard: Board = { title: "", notes: [], edges: [] };

export type EdgeKey = string & { readonly __brand: unique symbol };

export const edgeKey = (e: Pick<Edge, "from" | "to">): EdgeKey =>
	`${e.from}->${e.to}` as EdgeKey;

export const noteRect = (n: Note): Rect => {
	const meta = NOTE_TYPES[n.type];
	return { x: n.x, y: n.y, w: meta.width, h: meta.height };
};

export const findNote = (board: Board, id: string): Note | null =>
	board.notes.find((n) => n.id === id) ?? null;

export const nextNoteId = (
	type: NoteType,
	taken: ReadonlySet<string>,
): NoteId =>
	(function find(i: number): NoteId {
		const id = parseNoteId(`${ID_PREFIX[type]}${i}`);
		return id && !taken.has(id) ? id : find(i + 1);
	})(1);

export const addNote = (
	board: Board,
	type: NoteType,
	x: number,
	y: number,
): { board: Board; note: Note } => {
	const note: Note = {
		id: nextNoteId(type, new Set(board.notes.map((n) => n.id))),
		type,
		text: NOTE_TYPES[type].label,
		x,
		y,
	};
	return { board: { ...board, notes: [...board.notes, note] }, note };
};

export const moveNotes = (
	board: Board,
	ids: ReadonlySet<string>,
	dx: number,
	dy: number,
): Board => ({
	...board,
	notes: board.notes.map((n) =>
		ids.has(n.id) ? { ...n, x: n.x + dx, y: n.y + dy } : n,
	),
});

export const setNoteText = (
	board: Board,
	id: string,
	rawText: string,
): Board => {
	const text = rawText.trim();
	return {
		...board,
		notes: board.notes.map((n) =>
			n.id === id
				? {
						...n,
						text: text || (n.type === "policy" ? "" : NOTE_TYPES[n.type].label),
					}
				: n,
		),
	};
};

export type ConnectResult =
	| { readonly kind: "connected"; readonly board: Board; readonly edge: Edge }
	| { readonly kind: "duplicate" };

export const connect = (
	board: Board,
	fromId: string,
	toId: string,
): ConnectResult => {
	if (board.edges.some((e) => e.from === fromId && e.to === toId))
		return { kind: "duplicate" };
	const from = findNote(board, fromId);
	const to = findNote(board, toId);
	if (!from || !to) return { kind: "duplicate" };
	const edge: Edge = {
		from: from.id,
		to: to.id,
		dashed: !isAllowedConnection(from.type, to.type),
	};
	return {
		kind: "connected",
		board: { ...board, edges: [...board.edges, edge] },
		edge,
	};
};

export const removeSelection = (board: Board, selection: Selection): Board => ({
	...board,
	notes: board.notes.filter((n) => !selection.noteIds.has(n.id)),
	edges: board.edges.filter(
		(e) =>
			!selection.edgeKeys.has(edgeKey(e)) &&
			!selection.noteIds.has(e.from) &&
			!selection.noteIds.has(e.to),
	),
});

export const notesInRect = (board: Board, rect: Rect): readonly NoteId[] =>
	board.notes.filter((n) => intersects(noteRect(n), rect)).map((n) => n.id);

export type Clipboard = {
	readonly notes: readonly Note[];
	readonly edges: readonly Edge[];
};

export const clipSelection = (
	board: Board,
	noteIds: ReadonlySet<string>,
): Clipboard => ({
	notes: board.notes.filter((n) => noteIds.has(n.id)),
	edges: board.edges.filter((e) => noteIds.has(e.from) && noteIds.has(e.to)),
});

export const pasteClipboard = (
	board: Board,
	clip: Clipboard,
	offset: number,
): { board: Board; pastedIds: readonly NoteId[] } => {
	const { idMap, notes } = clip.notes.reduce<{
		taken: ReadonlySet<string>;
		idMap: ReadonlyMap<NoteId, NoteId>;
		notes: readonly Note[];
	}>(
		(acc, n) => {
			const id = nextNoteId(n.type, acc.taken);
			return {
				taken: new Set([...acc.taken, id]),
				idMap: new Map([...acc.idMap, [n.id, id]]),
				notes: [...acc.notes, { ...n, id, x: n.x + offset, y: n.y + offset }],
			};
		},
		{
			taken: new Set(board.notes.map((n) => n.id)),
			idMap: new Map(),
			notes: [],
		},
	);
	const edges = clip.edges.flatMap((e) => {
		const from = idMap.get(e.from);
		const to = idMap.get(e.to);
		return from && to ? [{ ...e, from, to }] : [];
	});
	return {
		board: {
			...board,
			notes: [...board.notes, ...notes],
			edges: [...board.edges, ...edges],
		},
		pastedIds: notes.map((n) => n.id),
	};
};

export const ADJACENT_GAP = 48;

export const findAdjacentConnection = (
	board: Board,
	placed: Note,
): Edge | null => {
	const rect = noteRect(placed);
	const candidates = board.notes.flatMap((other) => {
		if (other.id === placed.id) return [];
		const o = noteRect(other);
		const overlapX =
			Math.min(rect.x + rect.w, o.x + o.w) - Math.max(rect.x, o.x);
		const overlapY =
			Math.min(rect.y + rect.h, o.y + o.h) - Math.max(rect.y, o.y);
		const gap =
			overlapY >= Math.min(rect.h, o.h) / 2
				? rect.x >= o.x + o.w
					? rect.x - (o.x + o.w)
					: o.x >= rect.x + rect.w
						? o.x - (rect.x + rect.w)
						: -1
				: overlapX >= Math.min(rect.w, o.w) / 2
					? rect.y >= o.y + o.h
						? rect.y - (o.y + o.h)
						: o.y >= rect.y + rect.h
							? o.y - (rect.y + rect.h)
							: -1
					: -1;
		if (gap < 0 || gap > ADJACENT_GAP) return [];
		const pair = isAllowedConnection(other.type, placed.type)
			? { from: other.id, to: placed.id }
			: isAllowedConnection(placed.type, other.type)
				? { from: placed.id, to: other.id }
				: null;
		if (!pair) return [];
		if (board.edges.some((e) => e.from === pair.from && e.to === pair.to))
			return [];
		return [{ gap, edge: { ...pair, dashed: false } }];
	});
	return candidates.length
		? candidates.reduce((best, cur) => (cur.gap < best.gap ? cur : best)).edge
		: null;
};
