import { computed } from "@preact/signals";
import { board, loadBoard } from "../board/store";
import type { DslParseResult } from "./_models/dsl";
import { parseDsl, serializeDsl } from "./_models/dsl";

export const dslText = computed(() => serializeDsl(board.value));

export const applyDslText = (text: string): DslParseResult => {
	const result = parseDsl(text);
	if (!result.errors.length) loadBoard(result.board);
	return result;
};
