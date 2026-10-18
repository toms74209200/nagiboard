export type NoteId = string & { readonly __brand: unique symbol };

const NOTE_ID_PATTERN = /^[A-Za-z_][\w-]*$/;

export const parseNoteId = (value: string): NoteId | null =>
	NOTE_ID_PATTERN.test(value) ? (value as NoteId) : null;
