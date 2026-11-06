export type RoomId = string & { readonly __brand: unique symbol };

const ROOM_ID_PATTERN =
	/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

export const parseRoomId = (value: string): RoomId | null =>
	ROOM_ID_PATTERN.test(value) ? (value as RoomId) : null;
