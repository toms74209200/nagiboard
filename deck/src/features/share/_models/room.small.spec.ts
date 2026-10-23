import { expect, test } from "vitest";
import {
	firstBoardId,
	parseCreatedRoom,
	problemMessage,
	withRoom,
} from "./room";

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

test.each([
	{
		name: "when firstBoardId with boards then returns the first id",
		body: {
			boards: [
				"0b7c9e2d-5f14-4a3b-8c6d-2e9f1a4b7c3e",
				"1c8d0f3e-6a25-4b4c-9d7e-3f0a2b5c8d4f",
			],
		},
		expected: "0b7c9e2d-5f14-4a3b-8c6d-2e9f1a4b7c3e",
	},
	{
		name: "when firstBoardId with empty boards then returns null",
		body: { boards: [] },
		expected: null,
	},
	{
		name: "when firstBoardId with non-string id then returns null",
		body: { boards: [1] },
		expected: null,
	},
	{
		name: "when firstBoardId without boards then returns null",
		body: {},
		expected: null,
	},
	{
		name: "when firstBoardId with null then returns null",
		body: null,
		expected: null,
	},
])("$name", ({ body, expected }) => {
	expect(firstBoardId(body)).toBe(expected);
});
