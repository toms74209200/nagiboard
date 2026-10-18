import { expect, test } from "vitest";
import { parseNoteId } from "./noteId";

test.each([
	{
		name: "when parseNoteId with prefix and number then returns the id",
		input: "e1",
		expected: "e1",
	},
	{
		name: "when parseNoteId with underscore start then returns the id",
		input: "_x-1",
		expected: "_x-1",
	},
	{
		name: "when parseNoteId with digit start then returns null",
		input: "1e",
		expected: null,
	},
	{
		name: "when parseNoteId with empty string then returns null",
		input: "",
		expected: null,
	},
	{
		name: "when parseNoteId with whitespace then returns null",
		input: "e 1",
		expected: null,
	},
	{
		name: "when parseNoteId with quote then returns null",
		input: 'e"1',
		expected: null,
	},
])("$name", ({ input, expected }) => {
	expect(parseNoteId(input)).toBe(expected);
});
