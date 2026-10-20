import { expect, test } from "vitest";
import { parseCreatedRoom, problemMessage, withRoom } from "./room";

test.each([
	{
		name: "when parseCreatedRoom with id and shareUrl then returns the room",
		body: {
			id: "6f1c2b7e-3a4d-4e8f-9b0a-1c2d3e4f5a6b",
			shareUrl: "/?room=6f1c2b7e-3a4d-4e8f-9b0a-1c2d3e4f5a6b",
		},
		expected: {
			id: "6f1c2b7e-3a4d-4e8f-9b0a-1c2d3e4f5a6b",
			shareUrl: "/?room=6f1c2b7e-3a4d-4e8f-9b0a-1c2d3e4f5a6b",
		},
	},
	{
		name: "when parseCreatedRoom without shareUrl then returns null",
		body: { id: "6f1c2b7e-3a4d-4e8f-9b0a-1c2d3e4f5a6b" },
		expected: null,
	},
	{
		name: "when parseCreatedRoom with non-string id then returns null",
		body: { id: 1, shareUrl: "/?room=1" },
		expected: null,
	},
	{
		name: "when parseCreatedRoom with null then returns null",
		body: null,
		expected: null,
	},
])("$name", ({ body, expected }) => {
	expect(parseCreatedRoom(body)).toEqual(expected);
});

test.each([
	{
		name: "when problemMessage with diagnostics then returns the first message",
		body: {
			status: 400,
			detail: "detail",
			diagnostics: [
				{ line: 4, message: '4行目: 未知の付箋タイプ "sticky"' },
				{ line: 5, message: "5行目: 解釈できません → ?!" },
			],
		},
		expected: '4行目: 未知の付箋タイプ "sticky"',
	},
	{
		name: "when problemMessage without diagnostics then returns the detail",
		body: { status: 400, detail: "data cannot be decoded" },
		expected: "data cannot be decoded",
	},
	{
		name: "when problemMessage with empty diagnostics then returns the detail",
		body: { status: 400, detail: "detail", diagnostics: [] },
		expected: "detail",
	},
	{
		name: "when problemMessage without diagnostics nor detail then returns null",
		body: { status: 400, title: "Bad Request" },
		expected: null,
	},
	{
		name: "when problemMessage with non-object then returns null",
		body: "Bad Request",
		expected: null,
	},
])("$name", ({ body, expected }) => {
	expect(problemMessage(body)).toBe(expected);
});

test.each([
	{
		name: "when withRoom with data then keeps data and adds room",
		href: "https://example.com/?data=abc",
		expected: "https://example.com/?data=abc&room=r1",
	},
	{
		name: "when withRoom with existing room then replaces it",
		href: "https://example.com/?data=abc&room=old",
		expected: "https://example.com/?data=abc&room=r1",
	},
	{
		name: "when withRoom without query then adds room",
		href: "https://example.com/",
		expected: "https://example.com/?room=r1",
	},
])("$name", ({ href, expected }) => {
	expect(withRoom(href, "r1")).toBe(expected);
});
