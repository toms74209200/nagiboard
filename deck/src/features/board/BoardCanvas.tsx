import { useSignal } from "@preact/signals";
import { useEffect, useRef } from "preact/hooks";
import { ToastView } from "../../components/ToastView";
import { showToast } from "../../components/toast";
import type { Note } from "./_models/board";
import { edgeKey, findNote, noteRect, notesInRect } from "./_models/board";
import type { Point, Rect } from "./_models/geometry";
import {
	nearestSidePoint,
	rectFromPoints,
	snapToGrid,
} from "./_models/geometry";
import type { NoteId } from "./_models/noteId";
import { NOTE_TYPES } from "./_models/noteType";
import {
	emptySelection,
	onlyEdge,
	onlyNote,
	withNotes,
	withToggledEdge,
	withToggledNote,
} from "./_models/selection";
import { EdgeView } from "./EdgeView";
import { NoteView } from "./NoteView";
import { Palette } from "./Palette";
import {
	activeTool,
	board,
	connectNotes,
	copySelectionToClipboard,
	deleteSelection,
	editingNote,
	pasteFromClipboard,
	placeNote,
	selection,
	setNotePositions,
	viewTransform,
} from "./store";

const MIN_SCALE = 0.25;
const MAX_SCALE = 2.5;
const DOUBLE_CLICK_MS = 450;

type DragState =
	| {
			readonly mode: "pan";
			readonly startClientX: number;
			readonly startClientY: number;
			readonly originX: number;
			readonly originY: number;
	  }
	| {
			readonly mode: "note";
			readonly grabDX: number;
			readonly grabDY: number;
			readonly anchorStart: Point;
			readonly origins: ReadonlyMap<string, Point>;
			readonly moved: boolean;
			readonly collapseTo: NoteId | null;
	  }
	| { readonly mode: "link"; readonly from: Note }
	| { readonly mode: "marquee"; readonly start: Point };

export const BoardCanvas = () => {
	const viewportRef = useRef<HTMLDivElement>(null);
	const tempLineD = useSignal<string | null>(null);
	const marquee = useSignal<Rect | null>(null);
	const isPanning = useSignal(false);
	const isLinking = useSignal(false);
	const dragRef = useRef<DragState | null>(null);
	const pendingEditRef = useRef<{ id: NoteId; selectAll: boolean } | null>(
		null,
	);
	const lastNoteClickRef = useRef<{ id: NoteId; at: number } | null>(null);

	const viewportRect = (): DOMRect =>
		viewportRef.current?.getBoundingClientRect() ?? new DOMRect();

	const toWorld = (ev: MouseEvent): Point => {
		const rect = viewportRect();
		const v = viewTransform.peek();
		return {
			x: (ev.clientX - rect.left - v.x) / v.scale,
			y: (ev.clientY - rect.top - v.y) / v.scale,
		};
	};

	const zoomAt = (factor: number, clientPoint?: Point) => {
		const rect = viewportRect();
		const cx = clientPoint ? clientPoint.x - rect.left : rect.width / 2;
		const cy = clientPoint ? clientPoint.y - rect.top : rect.height / 2;
		const v = viewTransform.peek();
		const scale = Math.min(MAX_SCALE, Math.max(MIN_SCALE, v.scale * factor));
		viewTransform.value = {
			x: cx - (cx - v.x) * (scale / v.scale),
			y: cy - (cy - v.y) * (scale / v.scale),
			scale,
		};
	};

	const zoomFit = () => {
		const notes = board.peek().notes;
		if (!notes.length) return;
		const rects = notes.map(noteRect);
		const minX = Math.min(...rects.map((r) => r.x));
		const minY = Math.min(...rects.map((r) => r.y));
		const maxX = Math.max(...rects.map((r) => r.x + r.w));
		const maxY = Math.max(...rects.map((r) => r.y + r.h));
		const rect = viewportRect();
		const pad = 60;
		const scale = Math.min(
			2,
			Math.max(
				MIN_SCALE,
				Math.min(
					(rect.width - pad * 2) / (maxX - minX),
					(rect.height - pad * 2) / (maxY - minY),
				),
			),
		);
		viewTransform.value = {
			scale,
			x: (rect.width - (maxX - minX) * scale) / 2 - minX * scale,
			y: (rect.height - (maxY - minY) * scale) / 2 - minY * scale,
		};
	};

	const onMouseDown = (ev: MouseEvent) => {
		if (ev.button !== 0) return;
		const target = ev.target as Element;
		if (target.closest("#palette,#zoomControls,#hint,#toast")) return;
		const viewport = viewportRef.current;
		if (!viewport) return;

		const editing = editingNote.peek();
		if (editing) {
			const editingEl = viewport.querySelector(
				`.note[data-id="${CSS.escape(editing.id)}"]`,
			);
			if (editingEl?.contains(target)) return;
			editingEl?.querySelector<HTMLElement>(".txt")?.blur();
			return;
		}

		const anchorEl = target.closest(".anchor");
		const noteEl = target.closest<HTMLElement>(".note");

		if (anchorEl && noteEl) {
			const from = findNote(board.peek(), noteEl.dataset.id ?? "");
			if (!from) return;
			dragRef.current = { mode: "link", from };
			isLinking.value = true;
			ev.preventDefault();
			return;
		}

		if (noteEl) {
			const note = findNote(board.peek(), noteEl.dataset.id ?? "");
			if (!note) return;
			if (ev.shiftKey) {
				selection.value = withToggledNote(selection.peek(), note.id);
				ev.preventDefault();
				return;
			}
			const now = Date.now();
			const lastClick = lastNoteClickRef.current;
			if (
				lastClick &&
				lastClick.id === note.id &&
				now - lastClick.at < DOUBLE_CLICK_MS
			) {
				lastNoteClickRef.current = null;
				selection.value = onlyNote(note.id);
				pendingEditRef.current = { id: note.id, selectAll: false };
				ev.preventDefault();
				return;
			}
			lastNoteClickRef.current = { id: note.id, at: now };

			const wasInMultiSelection =
				selection.peek().noteIds.has(note.id) &&
				selection.peek().noteIds.size > 1;
			if (!selection.peek().noteIds.has(note.id))
				selection.value = onlyNote(note.id);
			const p = toWorld(ev);
			const origins = new Map(
				board
					.peek()
					.notes.filter((n) => selection.peek().noteIds.has(n.id))
					.map((n) => [n.id, { x: n.x, y: n.y }]),
			);
			dragRef.current = {
				mode: "note",
				grabDX: p.x - note.x,
				grabDY: p.y - note.y,
				anchorStart: { x: note.x, y: note.y },
				origins,
				moved: false,
				collapseTo: wasInMultiSelection ? note.id : null,
			};
			ev.preventDefault();
			return;
		}

		const edgeEl = target.closest<SVGGElement>(".edge");
		if (edgeEl) {
			const edge = board
				.peek()
				.edges.find((e) => edgeKey(e) === edgeEl.dataset.edge);
			if (!edge) return;
			const key = edgeKey(edge);
			selection.value = ev.shiftKey
				? withToggledEdge(selection.peek(), key)
				: onlyEdge(key);
			return;
		}

		lastNoteClickRef.current = null;
		const p = toWorld(ev);

		const tool = activeTool.peek();
		if (tool) {
			const { note, autoEdge } = placeNote(tool, p.x, p.y);
			pendingEditRef.current = { id: note.id, selectAll: true };
			if (autoEdge)
				showToast(
					`隣接により自動接続しました: ${autoEdge.from} → ${autoEdge.to}`,
				);
			ev.preventDefault();
			return;
		}

		if (ev.shiftKey) {
			dragRef.current = { mode: "marquee", start: p };
			marquee.value = { x: p.x, y: p.y, w: 0, h: 0 };
			ev.preventDefault();
			return;
		}

		selection.value = emptySelection;
		const v = viewTransform.peek();
		dragRef.current = {
			mode: "pan",
			startClientX: ev.clientX,
			startClientY: ev.clientY,
			originX: v.x,
			originY: v.y,
		};
		isPanning.value = true;
	};

	useEffect(() => {
		const viewport = viewportRef.current;
		if (!viewport) return;

		const onMouseMove = (ev: MouseEvent) => {
			const drag = dragRef.current;
			if (!drag) return;
			if (drag.mode === "pan") {
				const v = viewTransform.peek();
				viewTransform.value = {
					...v,
					x: drag.originX + ev.clientX - drag.startClientX,
					y: drag.originY + ev.clientY - drag.startClientY,
				};
			} else if (drag.mode === "note") {
				const p = toWorld(ev);
				const nx = snapToGrid(p.x - drag.grabDX);
				const ny = snapToGrid(p.y - drag.grabDY);
				const dx = nx - drag.anchorStart.x;
				const dy = ny - drag.anchorStart.y;
				if (!drag.moved && dx === 0 && dy === 0) return;
				dragRef.current = { ...drag, moved: true };
				lastNoteClickRef.current = null;
				setNotePositions(
					new Map(
						[...drag.origins].map(([id, o]) => [
							id,
							{ x: o.x + dx, y: o.y + dy },
						]),
					),
				);
			} else if (drag.mode === "link") {
				const p = toWorld(ev);
				const start = nearestSidePoint(noteRect(drag.from), p);
				tempLineD.value = `M${start.x},${start.y} L${p.x},${p.y}`;
			} else if (drag.mode === "marquee") {
				marquee.value = rectFromPoints(drag.start, toWorld(ev));
			}
		};

		const onMouseUp = (ev: MouseEvent) => {
			const pendingEdit = pendingEditRef.current;
			if (pendingEdit) {
				pendingEditRef.current = null;
				setTimeout(() => {
					editingNote.value = pendingEdit;
				}, 0);
			}
			const drag = dragRef.current;
			if (!drag) return;
			dragRef.current = null;

			if (drag.mode === "link") {
				isLinking.value = false;
				tempLineD.value = null;
				const targetEl = document
					.elementFromPoint(ev.clientX, ev.clientY)
					?.closest<HTMLElement>(".note");
				const toId = targetEl?.dataset.id;
				if (toId && toId !== drag.from.id) {
					const result = connectNotes(drag.from.id, toId);
					if (result.kind === "duplicate") {
						showToast("同じ矢印が既にあります");
					} else if (result.edge.dashed) {
						const to = findNote(board.peek(), toId);
						showToast(
							`${NOTE_TYPES[drag.from.type].label} → ${to ? NOTE_TYPES[to.type].label : "?"} はルール外のため点線(例外)にしました`,
							"error",
						);
					}
				}
			} else if (drag.mode === "pan") {
				isPanning.value = false;
			} else if (drag.mode === "marquee") {
				const rect = marquee.peek();
				marquee.value = null;
				if (rect) selection.value = withNotes(notesInRect(board.peek(), rect));
			} else if (drag.mode === "note" && !drag.moved && drag.collapseTo) {
				selection.value = onlyNote(drag.collapseTo);
			}
		};

		const onKeyDown = (ev: KeyboardEvent) => {
			if (
				(ev.target as Element).closest("textarea,input,[contenteditable=true]")
			)
				return;
			const mod = ev.metaKey || ev.ctrlKey;
			if (mod && ev.key.toLowerCase() === "c") {
				const count = copySelectionToClipboard();
				if (count) {
					showToast(`付箋 ${count} 枚をコピーしました`);
					ev.preventDefault();
				}
				return;
			}
			if (mod && ev.key.toLowerCase() === "v") {
				const count = pasteFromClipboard();
				if (count) {
					showToast(`付箋 ${count} 枚を貼り付けました`);
					ev.preventDefault();
				}
				return;
			}
			if (mod && ev.key.toLowerCase() === "a") {
				selection.value = withNotes(board.peek().notes.map((n) => n.id));
				ev.preventDefault();
				return;
			}
			if (ev.key === "Delete" || ev.key === "Backspace") {
				deleteSelection();
				ev.preventDefault();
				return;
			}
			if (ev.key === "Escape") {
				activeTool.value = null;
				selection.value = emptySelection;
			}
		};

		const onWheel = (ev: WheelEvent) => {
			ev.preventDefault();
			zoomAt(ev.deltaY < 0 ? 1.1 : 1 / 1.1, { x: ev.clientX, y: ev.clientY });
		};

		window.addEventListener("mousemove", onMouseMove);
		window.addEventListener("mouseup", onMouseUp);
		window.addEventListener("keydown", onKeyDown);
		viewport.addEventListener("wheel", onWheel, { passive: false });

		zoomFit();

		return () => {
			window.removeEventListener("mousemove", onMouseMove);
			window.removeEventListener("mouseup", onMouseUp);
			window.removeEventListener("keydown", onKeyDown);
			viewport.removeEventListener("wheel", onWheel);
		};
	}, []);

	const v = viewTransform.value;
	const byId = new Map(board.value.notes.map((n) => [n.id, n]));
	const marqueeRect = marquee.value;

	return (
		<div
			id="viewport"
			role="application"
			aria-label="ボード"
			ref={viewportRef}
			class={`${activeTool.value ? "placing" : ""}${isPanning.value ? " panning" : ""}${isLinking.value ? " linking" : ""}`}
			style={{
				backgroundPosition: `${v.x}px ${v.y}px`,
				backgroundSize: `${24 * v.scale}px ${24 * v.scale}px`,
			}}
			onMouseDown={onMouseDown}
		>
			<div
				id="world"
				style={{ transform: `translate(${v.x}px,${v.y}px) scale(${v.scale})` }}
			>
				<svg id="edgeLayer" xmlns="http://www.w3.org/2000/svg" aria-hidden>
					<defs>
						<marker
							id="arrowhead"
							viewBox="0 0 10 10"
							refX="9"
							refY="5"
							markerWidth="7"
							markerHeight="7"
							orient="auto-start-reverse"
						>
							<path d="M0,0 L10,5 L0,10 z" class="marker-arrow" />
						</marker>
						<marker
							id="arrowhead-selected"
							viewBox="0 0 10 10"
							refX="9"
							refY="5"
							markerWidth="7"
							markerHeight="7"
							orient="auto-start-reverse"
						>
							<path d="M0,0 L10,5 L0,10 z" class="marker-arrow-selected" />
						</marker>
					</defs>
					<g>
						{board.value.edges.map((edge) => {
							const from = byId.get(edge.from);
							const to = byId.get(edge.to);
							return from && to ? (
								<EdgeView
									key={`${edge.from}->${edge.to}`}
									edge={edge}
									from={from}
									to={to}
								/>
							) : null;
						})}
					</g>
					{tempLineD.value && <path id="tempLine" d={tempLineD.value} />}
				</svg>
				<div id="notes">
					{board.value.notes.map((note) => (
						<NoteView key={note.id} note={note} />
					))}
				</div>
				{marqueeRect && (
					<div
						class="marquee"
						style={{
							left: `${marqueeRect.x}px`,
							top: `${marqueeRect.y}px`,
							width: `${marqueeRect.w}px`,
							height: `${marqueeRect.h}px`,
						}}
					/>
				)}
			</div>

			<Palette />

			<div id="zoomControls">
				<button type="button" class="btn" onClick={() => zoomAt(1 / 1.2)}>
					−
				</button>
				<button
					type="button"
					class="btn"
					onClick={() => zoomAt(1 / viewTransform.peek().scale)}
				>
					100%
				</button>
				<button type="button" class="btn" onClick={() => zoomAt(1.2)}>
					＋
				</button>
				<button type="button" class="btn" onClick={zoomFit}>
					全体
				</button>
			</div>
			<div id="hint">
				付箋: パレットを選びクリック ／ ダブルクリックで編集 ／
				端の●ドラッグで矢印 ／ Shift+クリック・Shift+ドラッグで複数選択 ／
				⌘C・⌘Vで複製 ／ Deleteで削除 ／ 空白ドラッグで移動
			</div>
			<ToastView />
		</div>
	);
};
