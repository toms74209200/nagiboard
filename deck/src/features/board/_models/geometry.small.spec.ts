import { expect, test } from "vitest";
import {
	edgePathD,
	intersects,
	nearestSidePoint,
	rectFromPoints,
	sidePoint,
	snapToGrid,
} from "./geometry";

test.each([
	{
		name: "when sidePoint north then returns top center",
		side: "n",
		expected: { x: 50, y: 0 },
	},
	{
		name: "when sidePoint south then returns bottom center",
		side: "s",
		expected: { x: 50, y: 80 },
	},
	{
		name: "when sidePoint west then returns left middle",
		side: "w",
		expected: { x: 0, y: 40 },
	},
	{
		name: "when sidePoint east then returns right middle",
		side: "e",
		expected: { x: 100, y: 40 },
	},
] as const)("$name", ({ side, expected }) => {
	expect(sidePoint({ x: 0, y: 0, w: 100, h: 80 }, side)).toEqual(expected);
});

test.each([
	{
		name: "when rects overlap then intersects",
		a: { x: 0, y: 0, w: 10, h: 10 },
		b: { x: 5, y: 5, w: 10, h: 10 },
		expected: true,
	},
	{
		name: "when rects are apart then does not intersect",
		a: { x: 0, y: 0, w: 10, h: 10 },
		b: { x: 20, y: 20, w: 10, h: 10 },
		expected: false,
	},
	{
		name: "when rects share only an edge then does not intersect",
		a: { x: 0, y: 0, w: 10, h: 10 },
		b: { x: 10, y: 0, w: 10, h: 10 },
		expected: false,
	},
])("$name", ({ a, b, expected }) => {
	expect(intersects(a, b)).toBe(expected);
});

test("when edgePathD with b right of a then connects east side to west side", () => {
	const a = { x: 0, y: 0, w: 100, h: 80 };
	const b = { x: 200, y: 0, w: 100, h: 80 };
	const d = edgePathD(a, b);
	expect(d.startsWith("M100,40 ")).toBe(true);
	expect(d.endsWith(" 200,40")).toBe(true);
});

test("when nearestSidePoint with point above then returns north anchor", () => {
	expect(
		nearestSidePoint({ x: 0, y: 0, w: 100, h: 80 }, { x: 50, y: -100 }),
	).toEqual({ x: 50, y: 0 });
});

test("when rectFromPoints with reversed corners then normalizes", () => {
	expect(rectFromPoints({ x: 10, y: 20 }, { x: 0, y: 0 })).toEqual({
		x: 0,
		y: 0,
		w: 10,
		h: 20,
	});
});

test.each([
	{
		name: "when snapToGrid below midpoint then rounds down",
		input: 3,
		expected: 0,
	},
	{
		name: "when snapToGrid above midpoint then rounds up",
		input: 5,
		expected: 8,
	},
	{
		name: "when snapToGrid negative then rounds to nearest",
		input: -13,
		expected: -16,
	},
])("$name", ({ input, expected }) => {
	expect(snapToGrid(input)).toBe(expected);
});
