import { expect, test } from "vitest";
import {
	base64UrlDecode,
	base64UrlEncode,
	decodeParamToDsl,
	encodeDslToParam,
} from "./codec";

test.each([
	{
		name: "when base64UrlEncode with bytes producing + and / then uses - and _",
		input: [251, 255, 191],
		expected: "-_-_",
	},
	{
		name: "when base64UrlEncode with padding-length input then strips =",
		input: [104, 105],
		expected: "aGk",
	},
])("$name", ({ input, expected }) => {
	expect(base64UrlEncode(new Uint8Array(input))).toBe(expected);
});

test("when base64UrlDecode then reverses base64UrlEncode", () => {
	const bytes = new Uint8Array([0, 1, 2, 250, 251, 252, 253, 254, 255]);
	expect(base64UrlDecode(base64UrlEncode(bytes))).toEqual(bytes);
});

test.each([
	{
		name: "when encode then decode ascii dsl then round trips",
		input: '# eventstorming v1\nevent e1 "a" @ 0,0\n',
	},
	{
		name: "when encode then decode japanese dsl then round trips",
		input: 'title 注文フロー\nevent e1 "注文が確定した" @ 672,224\n',
	},
	{ name: "when encode then decode empty text then round trips", input: "" },
	{
		name: "when encode then decode long repetitive dsl then round trips",
		input: Array.from(
			{ length: 200 },
			(_, i) => `event e${i} "イベント${i}" @ ${i * 30},${i * 30}`,
		).join("\n"),
	},
])("$name", async ({ input }) => {
	expect(await decodeParamToDsl(await encodeDslToParam(input))).toBe(input);
});

test("when encodeDslToParam then output is URL-safe without encoding", async () => {
	const param = await encodeDslToParam(
		'title テスト\nevent e1 "改行\nあり" @ 0,0\n',
	);
	expect(param).toMatch(/^[A-Za-z0-9_-]+$/);
	expect(encodeURIComponent(param)).toBe(param);
});

test("when encodeDslToParam with repetitive dsl then compresses below raw size", async () => {
	const dsl = Array.from(
		{ length: 100 },
		(_, i) => `event e${i} "ドメインイベント" @ ${i * 30},0`,
	).join("\n");
	const param = await encodeDslToParam(dsl);
	expect(param.length).toBeLessThan(dsl.length);
});
