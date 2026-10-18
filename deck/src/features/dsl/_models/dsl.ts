import type { Board, Edge } from "../../board/_models/board";
import type { NoteId } from "../../board/_models/noteId";
import { parseNoteId } from "../../board/_models/noteId";
import type { NoteType } from "../../board/_models/noteType";
import {
	isAllowedConnection,
	parseNoteType,
} from "../../board/_models/noteType";

export const DSL_HEADER = "# eventstorming v1";

export const escapeText = (s: string): string =>
	s.replace(/\\/g, "\\\\").replace(/"/g, '\\"').replace(/\n/g, "\\n");

export const unescapeText = (s: string): string =>
	s.replace(/\\(.)/g, (_, c: string) => (c === "n" ? "\n" : c));

export const serializeDsl = (board: Board): string => {
	const title = board.title.trim();
	const pad = Math.max(9, ...board.notes.map((n) => n.type.length));
	return `${[
		DSL_HEADER,
		...(title ? [`title ${title}`] : []),
		"",
		...board.notes.map(
			(n) =>
				`${n.type.padEnd(pad)} ${n.id.padEnd(4)} "${escapeText(n.text)}" @ ${Math.round(n.x)},${Math.round(n.y)}`,
		),
		...(board.edges.length ? [""] : []),
		...board.edges.map((e) => `${e.from} ${e.dashed ? "..>" : "->"} ${e.to}`),
	].join("\n")}\n`;
};

export type DslParseResult = {
	readonly board: Board;
	readonly errors: readonly string[];
	readonly coerced: number;
};

const NOTE_LINE =
	/^(\w+)\s+([A-Za-z_][\w-]*)\s+"((?:[^"\\]|\\.)*)"\s*(?:@\s*(-?\d+(?:\.\d+)?)\s*,\s*(-?\d+(?:\.\d+)?))?$/;
const EDGE_LINE = /^([\w-]+)\s*(->|\.\.>)\s*([\w-]+)$/;
const TITLE_LINE = /^title\s+(.+)$/;
const BEFORE_COMMENT = /^(?:[^"#]|"(?:[^"\\]|\\.)*(?:"|$))*/;

type NoteLine = {
	readonly kind: "note";
	readonly ln: number;
	readonly id: NoteId;
	readonly type: NoteType;
	readonly text: string;
	readonly x: number | undefined;
	readonly y: number | undefined;
};

type EdgeLine = {
	readonly kind: "edge";
	readonly ln: number;
	readonly from: string;
	readonly to: string;
	readonly dashed: boolean;
};

type ParsedLine =
	| { readonly kind: "title"; readonly title: string }
	| NoteLine
	| EdgeLine
	| { readonly kind: "error"; readonly message: string };

type ResolvedEdge =
	| {
			readonly kind: "resolved";
			readonly edge: Edge;
			readonly coerced: boolean;
	  }
	| { readonly kind: "unresolved"; readonly message: string };

const parseLine = (raw: string, ln: number): ParsedLine | null => {
	const line = (raw.match(BEFORE_COMMENT)?.[0] ?? raw).trim();
	if (!line) return null;

	const titleMatch = line.match(TITLE_LINE);
	if (titleMatch) return { kind: "title", title: (titleMatch[1] ?? "").trim() };

	const noteMatch = line.match(NOTE_LINE);
	if (noteMatch) {
		const [, rawType = "", rawId = "", rawText = "", x, y] = noteMatch;
		const type = parseNoteType(rawType.toLowerCase());
		const id = parseNoteId(rawId);
		if (!type)
			return {
				kind: "error",
				message: `${ln}行目: 未知の付箋タイプ "${rawType}"`,
			};
		if (!id)
			return { kind: "error", message: `${ln}行目: ID "${rawId}" が不正です` };
		return {
			kind: "note",
			ln,
			id,
			type,
			text: unescapeText(rawText),
			x: x !== undefined ? parseFloat(x) : undefined,
			y: y !== undefined ? parseFloat(y) : undefined,
		};
	}

	const edgeMatch = line.match(EDGE_LINE);
	if (edgeMatch) {
		const [, from = "", arrow, to = ""] = edgeMatch;
		return { kind: "edge", ln, from, to, dashed: arrow === "..>" };
	}

	return { kind: "error", message: `${ln}行目: 解釈できません → ${line}` };
};

export const parseDsl = (text: string): DslParseResult => {
	const lines = text.split(/\r?\n/).flatMap((raw, idx) => {
		const parsed = parseLine(raw, idx + 1);
		return parsed ? [parsed] : [];
	});

	const title =
		lines.flatMap((l) => (l.kind === "title" ? [l.title] : [])).at(-1) ?? "";

	const noteLines = lines.flatMap((l) => (l.kind === "note" ? [l] : []));
	const firstById = new Map(
		noteLines.toReversed().map((l) => [l.id, l] as const),
	);
	const notes = noteLines
		.filter((l) => firstById.get(l.id) === l)
		.map((l, i) => ({
			id: l.id,
			type: l.type,
			text: l.text,
			x: l.x ?? 40 + i * 30,
			y: l.y ?? 40 + i * 30,
		}));

	const lineErrors = lines.flatMap((l) =>
		l.kind === "error"
			? [l.message]
			: l.kind === "note" && firstById.get(l.id) !== l
				? [`${l.ln}行目: ID "${l.id}" が重複しています`]
				: [],
	);

	const byId = new Map(notes.map((n) => [n.id as string, n]));
	const resolved = lines
		.flatMap((l) => (l.kind === "edge" ? [l] : []))
		.map((e): ResolvedEdge => {
			const from = byId.get(e.from);
			const to = byId.get(e.to);
			if (!from || !to) {
				return {
					kind: "unresolved",
					message: `${e.ln}行目: 未定義のIDを参照しています (${e.from} / ${e.to})`,
				};
			}
			const dashed = e.dashed || !isAllowedConnection(from.type, to.type);
			return {
				kind: "resolved",
				edge: { from: from.id, to: to.id, dashed },
				coerced: dashed && !e.dashed,
			};
		});

	return {
		board: {
			title,
			notes,
			edges: resolved.flatMap((r) => (r.kind === "resolved" ? [r.edge] : [])),
		},
		errors: [
			...lineErrors,
			...resolved.flatMap((r) => (r.kind === "unresolved" ? [r.message] : [])),
		],
		coerced: resolved.filter((r) => r.kind === "resolved" && r.coerced).length,
	};
};
