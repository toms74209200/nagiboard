import { expect, test } from "vitest";
import type { Board, Edge, Note } from "../../board/_models/board";
import type { NoteId } from "../../board/_models/noteId";
import { parseNoteId } from "../../board/_models/noteId";
import { escapeText, parseDsl, serializeDsl, unescapeText } from "./dsl";

const nid = (s: string) => parseNoteId(s) as NoteId;
const note = (
	id: string,
	type: Note["type"],
	text: string,
	x: number,
	y: number,
): Note => ({
	id: nid(id),
	type,
	text,
	x,
	y,
});
const edge = (from: string, to: string, dashed: boolean): Edge => ({
	from: nid(from),
	to: nid(to),
	dashed,
});

test.each([
	{
		name: "when escapeText with backslash then doubles it",
		input: "a\\n",
		expected: "a\\\\n",
	},
	{
		name: "when escapeText with quote then escapes it",
		input: 'say "hi"',
		expected: 'say \\"hi\\"',
	},
	{
		name: "when escapeText with newline then becomes backslash n",
		input: "a\nb",
		expected: "a\\nb",
	},
])("$name", ({ input, expected }) => {
	expect(escapeText(input)).toBe(expected);
});

test.each([
	{ name: "when unescapeText reverses escaped newline", input: "a\nb" },
	{ name: "when unescapeText reverses literal backslash n", input: "a\\n b" },
	{
		name: "when unescapeText reverses quotes and backslashes",
		input: '\\ " \\\\ "" \n',
	},
])("$name (round trip)", ({ input }) => {
	expect(unescapeText(escapeText(input))).toBe(input);
});

const sample: Board = {
	title: "注文フロー",
	notes: [
		note("a1", "actor", "顧客", 48, 232),
		note("c1", "command", "注文を確定する", 216, 224),
		note("g1", "aggregate", "注文", 416, 208),
		note("e1", "event", "注文が確定した", 672, 224),
		note("p1", "policy", "", 880, 120),
		note("h1", "hotspot", "キャンセル期限は?", 672, 56),
	],
	edges: [
		edge("a1", "c1", false),
		edge("c1", "g1", false),
		edge("g1", "e1", false),
		edge("e1", "p1", false),
		edge("e1", "h1", true),
	],
};

test("when serialize then parse then board round trips", () => {
	const result = parseDsl(serializeDsl(sample));
	expect(result.errors).toEqual([]);
	expect(result.coerced).toBe(0);
	expect(result.board).toEqual(sample);
});

test("when serializeDsl then starts with version header", () => {
	expect(serializeDsl(sample).startsWith("# eventstorming v1\n")).toBe(true);
});

test("when serializeDsl with empty policy then writes empty quoted text", () => {
	expect(serializeDsl(sample)).toContain('p1   ""');
});

test("when parseDsl with comment after declaration then ignores the comment", () => {
	const result = parseDsl(
		'# eventstorming v1\nevent e1 "イベント # not comment" @ 0,0 # trailing\n',
	);
	expect(result.errors).toEqual([]);
	expect(result.board.notes[0]?.text).toBe("イベント # not comment");
});

test("when parseDsl without coordinates then assigns cascading defaults", () => {
	const result = parseDsl('event e1 "a"\nevent e2 "b"\n');
	expect(result.errors).toEqual([]);
	expect(result.board.notes[0]).toMatchObject({ x: 40, y: 40 });
	expect(result.board.notes[1]).toMatchObject({ x: 70, y: 70 });
});

test.each([
	{
		name: "when parseDsl with unknown type then reports line number",
		input: 'sticky x1 "a" @ 0,0',
		expectedError: '1行目: 未知の付箋タイプ "sticky"',
	},
	{
		name: "when parseDsl with duplicated id then reports error",
		input: 'event e1 "a" @ 0,0\nevent e1 "b" @ 0,0',
		expectedError: '2行目: ID "e1" が重複しています',
	},
	{
		name: "when parseDsl with undefined edge reference then reports error",
		input: 'event e1 "a" @ 0,0\ne1 -> zz',
		expectedError: "2行目: 未定義のIDを参照しています (e1 / zz)",
	},
	{
		name: "when parseDsl with garbage line then reports error",
		input: "?!",
		expectedError: "1行目: 解釈できません → ?!",
	},
])("$name", ({ input, expectedError }) => {
	expect(parseDsl(input).errors).toContain(expectedError);
});

test("when parseDsl with out-of-rule solid edge then coerces to dashed", () => {
	const result = parseDsl(
		'command c1 "a" @ 0,0\nactor a1 "b" @ 0,0\nc1 -> a1\n',
	);
	expect(result.errors).toEqual([]);
	expect(result.coerced).toBe(1);
	expect(result.board.edges[0]).toEqual({ from: "c1", to: "a1", dashed: true });
});

test("when parseDsl with explicit dashed edge then keeps it without coercion count", () => {
	const result = parseDsl(
		'event e1 "a" @ 0,0\nhotspot h1 "b" @ 0,0\ne1 ..> h1\n',
	);
	expect(result.coerced).toBe(0);
	expect(result.board.edges[0]).toEqual({ from: "e1", to: "h1", dashed: true });
});
