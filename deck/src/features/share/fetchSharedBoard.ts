import { firstBoardId } from "./_models/room";
import { API_ORIGIN } from "./apiOrigin";

export type FetchSharedBoardResult =
	| { readonly kind: "fetched"; readonly dsl: string }
	| { readonly kind: "notFound"; readonly target: "room" | "board" }
	| { readonly kind: "failed" };

export const fetchSharedBoard = async (
	roomId: string,
): Promise<FetchSharedBoardResult> => {
	try {
		const roomRes = await fetch(
			new URL(`/rooms/${encodeURIComponent(roomId)}/boards`, API_ORIGIN),
		);
		if (roomRes.status === 404) return { kind: "notFound", target: "room" };
		if (!roomRes.ok) return { kind: "failed" };
		const boardId = firstBoardId(await roomRes.json().catch(() => null));
		if (!boardId) return { kind: "notFound", target: "board" };

		const boardRes = await fetch(
			new URL(`/boards/${encodeURIComponent(boardId)}`, API_ORIGIN),
		);
		if (boardRes.status === 404) return { kind: "notFound", target: "board" };
		if (!boardRes.ok) return { kind: "failed" };
		return { kind: "fetched", dsl: await boardRes.text() };
	} catch {
		return { kind: "failed" };
	}
};
