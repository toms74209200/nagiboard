import type { BoardEdited, BoardSnapshot } from "../../../openapi/nagiboard";
import type { Board, ExecutedOperation } from "../../board/_models/board";
import { parseNoteId } from "../../board/_models/noteId";
import { parseNoteType } from "../../board/_models/noteType";
import { parseDsl } from "../../dsl/_models/dsl";

export type RoomEvent =
	| { readonly kind: "snapshot"; readonly board: Board }
	| {
			readonly kind: "edited";
			readonly operations: readonly ExecutedOperation[];
	  };

export const parseRoomEvent = (event: string, data: string): RoomEvent => {
	if (event === "snapshot") {
		const { dsl }: BoardSnapshot = JSON.parse(data);
		const { board, errors } = parseDsl(dsl);
		if (errors.length) throw new Error(errors[0]);
		return { kind: "snapshot", board };
	}
	const { edits }: BoardEdited = JSON.parse(data);
	return {
		kind: "edited",
		operations: edits.map((edit) => {
			if ("positions" in edit) {
				return {
					kind: "notesMoved",
					positions: edit.positions.map(({ id, x, y }) => {
						const noteId = parseNoteId(id);
						if (!noteId) throw new Error(`${id} is not a note ID`);
						return { id: noteId, x, y };
					}),
				};
			}
			const { text, x, y } = edit;
			const id = parseNoteId(edit.id);
			const type = parseNoteType(edit.noteType);
			if (!id || !type) throw new Error(`${edit.id} is not a note`);
			return { kind: "noteAdded", note: { id, type, text, x, y } };
		}),
	};
};
