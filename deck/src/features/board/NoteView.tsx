import type { Signal } from "@preact/signals";
import { batch } from "@preact/signals";
import { memo } from "preact/compat";
import { useContext, useLayoutEffect, useRef } from "preact/hooks";
import { BoardState } from "../../boardState";
import type { Note } from "./_models/board";
import { setNoteText } from "./_models/board";
import type { NoteId } from "./_models/noteId";
import { NOTE_TYPES } from "./_models/noteType";
import type { Selection } from "./_models/selection";

export type EditingNote = { readonly id: NoteId; readonly selectAll: boolean };

export const NoteView = memo(
	({
		note,
		selection,
		editingNote,
	}: {
		note: Note;
		selection: Signal<Selection>;
		editingNote: Signal<EditingNote | null>;
	}) => {
		const { board } = useContext(BoardState);
		const meta = NOTE_TYPES[note.type];
		const isEditing = editingNote.value?.id === note.id;
		const isSelected = selection.value.noteIds.has(note.id);
		const textRef = useRef<HTMLTextAreaElement>(null);

		const fitHeight = (el: HTMLTextAreaElement) => {
			el.style.height = "auto";
			el.style.height = `${el.scrollHeight}px`;
		};

		useLayoutEffect(() => {
			if (!isEditing) return;
			const el = textRef.current;
			if (!el) return;
			fitHeight(el);
			el.focus();
			if (editingNote.peek()?.selectAll) {
				el.select();
			} else {
				el.setSelectionRange(el.value.length, el.value.length);
			}
		}, [isEditing]);

		return (
			<div
				class={`note note--${note.type}${isSelected ? " selected" : ""}${isEditing ? " editing" : ""}`}
				data-id={note.id}
				style={{
					left: `${note.x}px`,
					top: `${note.y}px`,
					width: `${meta.width}px`,
					height: `${meta.height}px`,
				}}
			>
				<span class="badge">
					{meta.label} · {note.id}
				</span>
				{isEditing ? (
					<textarea
						ref={textRef}
						class="txt"
						rows={1}
						defaultValue={note.text}
						onInput={(ev) => fitHeight(ev.currentTarget)}
						onBlur={(ev) => {
							const text = ev.currentTarget.value;
							batch(() => {
								board.value = setNoteText(board.peek(), note.id, text);
								if (editingNote.peek()?.id === note.id)
									editingNote.value = null;
							});
						}}
						onKeyDown={(ev) => {
							if ((ev.key === "Enter" && !ev.shiftKey) || ev.key === "Escape") {
								ev.preventDefault();
								ev.currentTarget.blur();
							}
						}}
					/>
				) : (
					<span class="txt">{note.text}</span>
				)}
				<span class="anchor anchor--n" />
				<span class="anchor anchor--s" />
				<span class="anchor anchor--w" />
				<span class="anchor anchor--e" />
			</div>
		);
	},
);
