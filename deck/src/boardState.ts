import { computed, signal } from "@preact/signals";
import { createContext } from "preact";
import { emptyBoard } from "./features/board/_models/board";
import { serializeDsl } from "./features/dsl/_models/dsl";

export const createBoardState = () => {
	const board = signal(emptyBoard);
	const dslText = computed(() => serializeDsl(board.value));
	return { board, dslText };
};

export const BoardState = createContext(createBoardState());
