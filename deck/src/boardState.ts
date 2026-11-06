import { computed, signal } from "@preact/signals";
import { createContext } from "preact";
import { emptyBoard } from "./features/board/_models/board";
import { serializeDsl } from "./features/dsl/_models/dsl";
import type { RoomId } from "./features/share/_models/roomId";

export const createBoardState = () => {
	const board = signal(emptyBoard);
	const dslText = computed(() => serializeDsl(board.value));
	const roomId = signal<RoomId | null>(null);
	return { board, dslText, roomId };
};

export const BoardState = createContext(createBoardState());
