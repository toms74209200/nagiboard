import type { Board, Note } from "../../board/_models/board";
import { edgeKey, noteRect } from "../../board/_models/board";
import { edgePathD } from "../../board/_models/geometry";
import { NOTE_TYPES } from "../../board/_models/noteType";

export type MeasureText = (text: string, fontPx: number) => number;

export const FONT_FAMILY =
	'-apple-system,BlinkMacSystemFont,"Hiragino Kaku Gothic ProN","Hiragino Sans","Noto Sans JP","Yu Gothic UI",Meiryo,sans-serif';

const BODY_FONT_PX = 12;
const BADGE_FONT_PX = 9;
const LINE_HEIGHT = 17;
const NOTE_PADDING = 8;

export const escapeXml = (s: string): string =>
	s
		.replace(/&/g, "&amp;")
		.replace(/</g, "&lt;")
		.replace(/>/g, "&gt;")
		.replace(/"/g, "&quot;");

export const wrapText = (
	text: string,
	maxWidth: number,
	fontPx: number,
	measure: MeasureText,
): readonly string[] =>
	text.split("\n").flatMap((paragraph) => {
		const lines = [""];
		for (const ch of paragraph) {
			const current = lines[lines.length - 1] ?? "";
			if (current && measure(current + ch, fontPx) > maxWidth) {
				lines.push(ch);
			} else {
				lines[lines.length - 1] = current + ch;
			}
		}
		return lines;
	});

const noteSvg = (n: Note, measure: MeasureText): string => {
	const meta = NOTE_TYPES[n.type];
	const badgeColor =
		n.type === "hotspot" ? "rgba(255,255,255,0.75)" : "#717171";
	const lines = wrapText(
		n.text,
		meta.width - NOTE_PADDING * 2,
		BODY_FONT_PX,
		measure,
	);
	const firstLineY =
		n.y +
		meta.height / 2 -
		((lines.length - 1) * LINE_HEIGHT) / 2 +
		BODY_FONT_PX / 3;
	return [
		`<g>`,
		`<rect x="${n.x}" y="${n.y}" width="${meta.width}" height="${meta.height}" rx="2" fill="${meta.color}"/>`,
		`<text x="${n.x + 6}" y="${n.y + 12}" font-size="${BADGE_FONT_PX}" fill="${badgeColor}">${escapeXml(`${meta.label} · ${n.id}`)}</text>`,
		...lines.map(
			(line, i) =>
				`<text x="${n.x + meta.width / 2}" y="${firstLineY + i * LINE_HEIGHT}" font-size="${BODY_FONT_PX}" text-anchor="middle" fill="${meta.textColor}">${escapeXml(line)}</text>`,
		),
		`</g>`,
	].join("");
};

export type BoardSvg = {
	readonly svg: string;
	readonly width: number;
	readonly height: number;
};

export const buildBoardSvg = (
	board: Board,
	measure: MeasureText,
): BoardSvg | null => {
	if (!board.notes.length) return null;

	const rects = board.notes.map(noteRect);
	const padTop = 40 + (board.title.trim() ? 28 : 0);
	const minX = Math.min(...rects.map((r) => r.x)) - 40;
	const minY = Math.min(...rects.map((r) => r.y)) - padTop;
	const maxX = Math.max(...rects.map((r) => r.x + r.w)) + 40;
	const maxY = Math.max(...rects.map((r) => r.y + r.h)) + 40;
	const width = maxX - minX;
	const height = maxY - minY;

	const byId = new Map(board.notes.map((n) => [n.id, n]));
	const edgeParts = board.edges.flatMap((e) => {
		const from = byId.get(e.from);
		const to = byId.get(e.to);
		if (!from || !to) return [];
		const dash = e.dashed ? ' stroke-dasharray="6 5"' : "";
		return [
			`<path data-edge="${escapeXml(edgeKey(e))}" d="${edgePathD(noteRect(from), noteRect(to))}" fill="none" stroke="#717171" stroke-width="2"${dash} marker-end="url(#arrowhead)"/>`,
		];
	});

	const svg = [
		`<svg xmlns="http://www.w3.org/2000/svg" viewBox="${minX} ${minY} ${width} ${height}" width="${width}" height="${height}" font-family='${FONT_FAMILY}'>`,
		`<defs><marker id="arrowhead" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" fill="#717171"/></marker></defs>`,
		`<rect x="${minX}" y="${minY}" width="${width}" height="${height}" fill="#fdfdfd"/>`,
		...(board.title.trim()
			? [
					`<text x="${minX + 16}" y="${minY + 26}" font-size="14" fill="#717171">${escapeXml(board.title.trim())}</text>`,
				]
			: []),
		...edgeParts,
		...board.notes.map((n) => noteSvg(n, measure)),
		`</svg>`,
	].join("\n");

	return { svg, width, height };
};
