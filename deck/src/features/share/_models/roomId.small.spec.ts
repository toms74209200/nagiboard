import { expect, test } from "vitest";
import { parseRoomId } from "./roomId";

test.each([
	{
		name: "when parseRoomId with uuid then returns the id",
		input: "6f1c2b7e-3a4d-4e8f-9b0a-1c2d3e4f5a6b",
		expected: "6f1c2b7e-3a4d-4e8f-9b0a-1c2d3e4f5a6b",
	},
	{
		name: "when parseRoomId with text that is not uuid then returns null",
		input: "abc",
		expected: null,
	},
	{
		name: "when parseRoomId with empty string then returns null",
		input: "",
		expected: null,
	},
])("$name", ({ input, expected }) => {
	expect(parseRoomId(input)).toBe(expected);
});
