import { expect, test } from "vitest";
import { parseDsl } from "../../dsl/_models/dsl";
import { parseRoomEvent } from "./roomEvent";

test("when parseRoomEvent with snapshot then returns the board of its dsl", () => {
	const dsl = '# eventstorming v1\n\ncommand c1 "b" @ 8,16\n';
	expect(parseRoomEvent("snapshot", JSON.stringify({ seq: 2, dsl }))).toEqual({
		kind: "snapshot",
		board: parseDsl(dsl).board,
	});
});

test("when parseRoomEvent with snapshot of invalid dsl then throws", () => {
	expect(() =>
		parseRoomEvent("snapshot", JSON.stringify({ seq: 2, dsl: "sticky s1" })),
	).toThrow();
});

test("when parseRoomEvent with edited then returns its operations in order", () => {
	expect(
		parseRoomEvent(
			"edited",
			JSON.stringify({
				seq: 3,
				edits: [
					{
						type: "noteAdded",
						noteType: "event",
						id: "e2",
						text: "Order placed",
						x: 384,
						y: 224,
					},
					{ type: "notesMoved", positions: [{ id: "c1", x: 216, y: 240 }] },
				],
			}),
		),
	).toEqual({
		kind: "edited",
		operations: [
			{
				kind: "noteAdded",
				note: {
					id: "e2",
					type: "event",
					text: "Order placed",
					x: 384,
					y: 224,
				},
			},
			{ kind: "notesMoved", positions: [{ id: "c1", x: 216, y: 240 }] },
		],
	});
});

test("when parseRoomEvent with unknown note type then throws", () => {
	expect(() =>
		parseRoomEvent(
			"edited",
			JSON.stringify({
				seq: 3,
				edits: [
					{
						type: "noteAdded",
						noteType: "sticky",
						id: "s1",
						text: "",
						x: 0,
						y: 0,
					},
				],
			}),
		),
	).toThrow();
});

test("when parseRoomEvent with text that is not json then throws", () => {
	expect(() => parseRoomEvent("edited", "abc")).toThrow();
});
