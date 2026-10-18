import { expect, test } from "vitest";
import type { EdgeKey } from "./board";
import type { NoteId } from "./noteId";
import { parseNoteId } from "./noteId";
import {
	emptySelection,
	isEmptySelection,
	onlyEdge,
	onlyNote,
	withNotes,
	withToggledEdge,
	withToggledNote,
} from "./selection";

const id = (s: string) => parseNoteId(s) as NoteId;
const key = (s: string) => s as EdgeKey;

test("when onlyNote then selects single note and clears edges", () => {
	const selection = withToggledEdge(emptySelection, key("a1->c1"));
	expect(onlyNote(id("e1"))).toEqual({
		noteIds: new Set([id("e1")]),
		edgeKeys: new Set(),
	});
	expect(onlyNote(id("e1")).edgeKeys.size).toBe(0);
	expect(selection.edgeKeys.size).toBe(1);
});

test("when onlyEdge then selects single edge and clears notes", () => {
	expect(onlyEdge(key("a1->c1"))).toEqual({
		noteIds: new Set(),
		edgeKeys: new Set([key("a1->c1")]),
	});
});

test("when withNotes then selects all given notes", () => {
	const selection = withNotes([id("e1"), id("e2")]);
	expect([...selection.noteIds]).toEqual([id("e1"), id("e2")]);
	expect(selection.edgeKeys.size).toBe(0);
});

test.each([
	{
		name: "when withToggledNote on unselected note then adds it",
		initial: [] as string[],
		toggle: "e1",
		expected: ["e1"],
	},
	{
		name: "when withToggledNote on selected note then removes it",
		initial: ["e1", "e2"],
		toggle: "e1",
		expected: ["e2"],
	},
])("$name", ({ initial, toggle, expected }) => {
	const selection = {
		noteIds: new Set(initial.map(id)),
		edgeKeys: new Set<EdgeKey>(),
	};
	expect([...withToggledNote(selection, id(toggle)).noteIds]).toEqual(
		expected.map(id),
	);
});

test("when withToggledNote then keeps edge selection", () => {
	const selection = withToggledNote(onlyEdge(key("a1->c1")), id("e1"));
	expect(selection.edgeKeys.has(key("a1->c1"))).toBe(true);
});

test.each([
	{
		name: "when withToggledEdge on unselected edge then adds it",
		initial: [] as string[],
		toggle: "a1->c1",
		expected: ["a1->c1"],
	},
	{
		name: "when withToggledEdge on selected edge then removes it",
		initial: ["a1->c1"],
		toggle: "a1->c1",
		expected: [] as string[],
	},
])("$name", ({ initial, toggle, expected }) => {
	const selection = {
		noteIds: new Set<NoteId>(),
		edgeKeys: new Set(initial.map(key)),
	};
	expect([...withToggledEdge(selection, key(toggle)).edgeKeys]).toEqual(
		expected.map(key),
	);
});

test.each([
	{
		name: "when emptySelection then isEmptySelection",
		selection: emptySelection,
		expected: true,
	},
	{
		name: "when note selected then not empty",
		selection: onlyNote(id("e1")),
		expected: false,
	},
	{
		name: "when edge selected then not empty",
		selection: onlyEdge(key("a1->c1")),
		expected: false,
	},
])("$name", ({ selection, expected }) => {
	expect(isEmptySelection(selection)).toBe(expected);
});
