import { expect, test } from "vitest";
import type { Board, Edge, Note } from "./board";
import {
	addNote,
	clipSelection,
	connect,
	edgeKey,
	emptyBoard,
	findAdjacentConnection,
	moveNotes,
	nextNoteId,
	notesInRect,
	pasteClipboard,
	removeSelection,
	setNoteText,
} from "./board";
import type { NoteId } from "./noteId";
import { parseNoteId } from "./noteId";

const nid = (s: string) => parseNoteId(s) as NoteId;

const note = (
	id: string,
	type: Note["type"],
	x = 0,
	y = 0,
	text = "t",
): Note => ({
	id: nid(id),
	type,
	text,
	x,
	y,
});

const edge = (from: string, to: string, dashed = false): Edge => ({
	from: nid(from),
	to: nid(to),
	dashed,
});

const board = (notes: readonly Note[], edges: readonly Edge[] = []): Board => ({
	title: "",
	notes,
	edges,
});

test.each([
	{
		name: "when nextNoteId with no taken ids then returns prefix 1",
		type: "event",
		taken: [],
		expected: "e1",
	},
	{
		name: "when nextNoteId with taken ids then skips to free number",
		type: "event",
		taken: ["e1", "e2"],
		expected: "e3",
	},
	{
		name: "when nextNoteId with gap in taken ids then fills the gap",
		type: "command",
		taken: ["c1", "c3"],
		expected: "c2",
	},
] as const)("$name", ({ type, taken, expected }) => {
	expect(nextNoteId(type, new Set(taken))).toBe(expected);
});

test("when addNote then appends note with type label as default text", () => {
	const { board: next, note: added } = addNote(emptyBoard, "event", 10, 20);
	expect(added).toEqual({
		id: "e1",
		type: "event",
		text: "ドメインイベント",
		x: 10,
		y: 20,
	});
	expect(next.notes).toEqual([added]);
});

test("when moveNotes then moves only the given ids", () => {
	const b = board([note("e1", "event", 0, 0), note("c1", "command", 100, 0)]);
	const moved = moveNotes(b, new Set(["e1"]), 8, 16);
	expect(moved.notes[0]).toMatchObject({ x: 8, y: 16 });
	expect(moved.notes[1]).toMatchObject({ x: 100, y: 0 });
	expect(moved.notes[1]).toBe(b.notes[1]);
});

test.each([
	{
		name: "when setNoteText with blank on policy then keeps empty (always fires)",
		type: "policy",
		input: "  ",
		expected: "",
	},
	{
		name: "when setNoteText with blank on event then falls back to type label",
		type: "event",
		input: "",
		expected: "ドメインイベント",
	},
	{
		name: "when setNoteText with text then trims it",
		type: "command",
		input: " 注文する ",
		expected: "注文する",
	},
] as const)("$name", ({ type, input, expected }) => {
	const b = board([note("n1", type)]);
	expect(setNoteText(b, "n1", input).notes[0]?.text).toBe(expected);
});

test.each([
	{
		name: "when connect rule-conformant pair then solid edge",
		notes: [note("a1", "actor"), note("c1", "command")],
		from: "a1",
		to: "c1",
		expectedDashed: false,
	},
	{
		name: "when connect out-of-rule pair then dashed exception edge",
		notes: [note("c1", "command"), note("a1", "actor")],
		from: "c1",
		to: "a1",
		expectedDashed: true,
	},
])("$name", ({ notes, from, to, expectedDashed }) => {
	const result = connect(board(notes), from, to);
	expect(result.kind).toBe("connected");
	if (result.kind === "connected") {
		expect(result.edge.dashed).toBe(expectedDashed);
	}
});

test("when connect same pair twice then duplicate", () => {
	const b = board([note("a1", "actor"), note("c1", "command")]);
	const first = connect(b, "a1", "c1");
	expect(first.kind).toBe("connected");
	if (first.kind !== "connected") return;
	expect(connect(first.board, "a1", "c1")).toEqual({ kind: "duplicate" });
});

test("when removeSelection with note then removes note and its edges", () => {
	const b = board(
		[note("a1", "actor"), note("c1", "command"), note("g1", "aggregate")],
		[edge("a1", "c1"), edge("c1", "g1")],
	);
	const next = removeSelection(b, {
		noteIds: new Set([nid("c1")]),
		edgeKeys: new Set(),
	});
	expect(next.notes.map((n) => n.id)).toEqual(["a1", "g1"]);
	expect(next.edges).toEqual([]);
});

test("when removeSelection with edge key then removes only that edge", () => {
	const b = board(
		[note("a1", "actor"), note("c1", "command")],
		[edge("a1", "c1")],
	);
	const next = removeSelection(b, {
		noteIds: new Set(),
		edgeKeys: new Set([edgeKey({ from: nid("a1"), to: nid("c1") })]),
	});
	expect(next.notes).toHaveLength(2);
	expect(next.edges).toEqual([]);
});

test("when notesInRect then returns intersecting note ids", () => {
	const b = board([note("e1", "event", 0, 0), note("e2", "event", 500, 500)]);
	expect(notesInRect(b, { x: -10, y: -10, w: 50, h: 50 })).toEqual(["e1"]);
});

test("when clip and paste then assigns new ids and keeps internal edges only", () => {
	const b = board(
		[
			note("a1", "actor", 0, 0),
			note("c1", "command", 200, 0),
			note("g1", "aggregate", 400, 0),
		],
		[edge("a1", "c1"), edge("c1", "g1")],
	);
	const clip = clipSelection(b, new Set(["a1", "c1"]));
	expect(clip.notes.map((n) => n.id)).toEqual(["a1", "c1"]);
	expect(clip.edges).toEqual([edge("a1", "c1")]);

	const { board: next, pastedIds } = pasteClipboard(b, clip, 24);
	expect(pastedIds).toEqual(["a2", "c2"]);
	expect(next.notes).toHaveLength(5);
	expect(next.edges).toContainEqual(edge("a2", "c2"));
	expect(next.notes.find((n) => n.id === "a2")).toMatchObject({
		x: 24,
		y: 24,
	});
});

test.each([
	{
		name: "when placed right next to allowed source then connects existing to placed",
		existing: note("a1", "actor", 0, 0),
		placed: note("c1", "command", 130 + 24, 0),
		expected: { from: "a1", to: "c1", dashed: false },
	},
	{
		name: "when placed left of allowed target then connects placed to existing",
		existing: note("g1", "aggregate", 200, 0),
		placed: note("c1", "command", 200 - 130 - 24, 0),
		expected: { from: "c1", to: "g1", dashed: false },
	},
	{
		name: "when placed below allowed source then connects vertically",
		existing: note("e1", "event", 0, 0),
		placed: note("p1", "policy", 0, 84 + 24),
		expected: { from: "e1", to: "p1", dashed: false },
	},
	{
		name: "when gap exceeds threshold then no connection",
		existing: note("a1", "actor", 0, 0),
		placed: note("c1", "command", 130 + 49, 0),
		expected: null,
	},
	{
		name: "when no rule fits either direction then no connection",
		existing: note("e1", "event", 0, 0),
		placed: note("a1", "actor", 130 + 24, 0),
		expected: null,
	},
	{
		name: "when notes are diagonal without enough overlap then no connection",
		existing: note("a1", "actor", 0, 0),
		placed: note("c1", "command", 130 + 24, 84),
		expected: null,
	},
])("$name", ({ existing, placed, expected }) => {
	const b = board([existing, placed]);
	expect(findAdjacentConnection(b, placed)).toEqual(expected);
});
