import { expect, test } from "vitest";
import { boardSourceOf } from "./boardSource";

test.each([
	{
		name: "when boardSourceOf with data then returns data",
		search: "?data=abc",
		expected: { kind: "data", param: "abc" },
	},
	{
		name: "when boardSourceOf with room then returns room",
		search: "?room=6f1c2b7e-3a4d-4e8f-9b0a-1c2d3e4f5a6b",
		expected: { kind: "room", id: "6f1c2b7e-3a4d-4e8f-9b0a-1c2d3e4f5a6b" },
	},
	{
		name: "when boardSourceOf with data and room then prefers data",
		search: "?room=6f1c2b7e-3a4d-4e8f-9b0a-1c2d3e4f5a6b&data=abc",
		expected: { kind: "data", param: "abc" },
	},
	{
		name: "when boardSourceOf with empty data and room then returns room",
		search: "?data=&room=6f1c2b7e-3a4d-4e8f-9b0a-1c2d3e4f5a6b",
		expected: { kind: "room", id: "6f1c2b7e-3a4d-4e8f-9b0a-1c2d3e4f5a6b" },
	},
	{
		name: "when boardSourceOf without query then returns empty",
		search: "",
		expected: { kind: "empty" },
	},
	{
		name: "when boardSourceOf with empty room then returns empty",
		search: "?room=",
		expected: { kind: "empty" },
	},
	{
		name: "when boardSourceOf with unrelated params then returns empty",
		search: "?foo=bar",
		expected: { kind: "empty" },
	},
])("$name", ({ search, expected }) => {
	expect(boardSourceOf(search)).toEqual(expected);
});
