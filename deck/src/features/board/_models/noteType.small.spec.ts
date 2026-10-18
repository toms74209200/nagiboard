import { expect, test } from "vitest";
import { isAllowedConnection, parseNoteType } from "./noteType";

test.each([
	{
		name: "when parseNoteType with known type then returns the type",
		input: "event",
		expected: "event",
	},
	{
		name: "when parseNoteType with unknown type then returns null",
		input: "unknown",
		expected: null,
	},
	{
		name: "when parseNoteType with prototype pollution key then returns null",
		input: "__proto__",
		expected: null,
	},
	{
		name: "when parseNoteType with empty string then returns null",
		input: "",
		expected: null,
	},
])("$name", ({ input, expected }) => {
	expect(parseNoteType(input)).toBe(expected);
});

test.each([
	{
		name: "when actor to command then allowed",
		from: "actor",
		to: "command",
		expected: true,
	},
	{
		name: "when command to aggregate then allowed",
		from: "command",
		to: "aggregate",
		expected: true,
	},
	{
		name: "when event to policy then allowed",
		from: "event",
		to: "policy",
		expected: true,
	},
	{
		name: "when command to actor then not allowed",
		from: "command",
		to: "actor",
		expected: false,
	},
	{
		name: "when hotspot to event then not allowed",
		from: "hotspot",
		to: "event",
		expected: false,
	},
	{
		name: "when event to hotspot then not allowed",
		from: "event",
		to: "hotspot",
		expected: false,
	},
] as const)("$name", ({ from, to, expected }) => {
	expect(isAllowedConnection(from, to)).toBe(expected);
});
