import type { RoomId } from "./roomId";
import { parseRoomId } from "./roomId";

export type BoardSource =
	| {
			readonly kind: "data";
			readonly param: string;
			readonly roomId: RoomId | null;
	  }
	| { readonly kind: "room"; readonly roomId: RoomId }
	| { readonly kind: "empty" };

export const boardSourceOf = (search: string): BoardSource => {
	const params = new URLSearchParams(search);
	const data = params.get("data");
	const roomId = parseRoomId(params.get("room") ?? "");
	if (data) return { kind: "data", param: data, roomId };
	if (roomId) return { kind: "room", roomId };
	return { kind: "empty" };
};
