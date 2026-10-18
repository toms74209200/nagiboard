export type Point = { readonly x: number; readonly y: number };
export type Rect = {
	readonly x: number;
	readonly y: number;
	readonly w: number;
	readonly h: number;
};
export type Side = "n" | "s" | "w" | "e";

const SIDES: readonly Side[] = ["n", "s", "w", "e"];

export const sidePoint = (r: Rect, side: Side): Point => {
	switch (side) {
		case "n":
			return { x: r.x + r.w / 2, y: r.y };
		case "s":
			return { x: r.x + r.w / 2, y: r.y + r.h };
		case "w":
			return { x: r.x, y: r.y + r.h / 2 };
		case "e":
			return { x: r.x + r.w, y: r.y + r.h / 2 };
	}
};

export const nearestSidePoint = (r: Rect, p: Point): Point =>
	SIDES.map((s) => sidePoint(r, s)).reduce((best, pt) =>
		Math.hypot(pt.x - p.x, pt.y - p.y) < Math.hypot(best.x - p.x, best.y - p.y)
			? pt
			: best,
	);

const controlOffset = (p: Point, side: Side, k: number): Point => ({
	x: p.x + (side === "e" ? k : side === "w" ? -k : 0),
	y: p.y + (side === "s" ? k : side === "n" ? -k : 0),
});

export const edgePathD = (a: Rect, b: Rect): string => {
	const { d, p1, p2, s1, s2 } = SIDES.flatMap((sa) =>
		SIDES.map((sb) => {
			const p1 = sidePoint(a, sa);
			const p2 = sidePoint(b, sb);
			return {
				d: Math.hypot(p1.x - p2.x, p1.y - p2.y),
				p1,
				p2,
				s1: sa,
				s2: sb,
			};
		}),
	).reduce((best, cur) => (cur.d < best.d ? cur : best));
	const k = Math.max(30, d * 0.35);
	const c1 = controlOffset(p1, s1, k);
	const c2 = controlOffset(p2, s2, k);
	return `M${p1.x},${p1.y} C${c1.x},${c1.y} ${c2.x},${c2.y} ${p2.x},${p2.y}`;
};

export const intersects = (a: Rect, b: Rect): boolean =>
	a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h;

export const rectFromPoints = (p1: Point, p2: Point): Rect => ({
	x: Math.min(p1.x, p2.x),
	y: Math.min(p1.y, p2.y),
	w: Math.abs(p1.x - p2.x),
	h: Math.abs(p1.y - p2.y),
});

export const snapToGrid = (value: number, grid = 8): number =>
	Math.round(value / grid) * grid;
