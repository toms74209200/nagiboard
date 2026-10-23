import { encodeDslToParam } from "./_models/codec";
import { parseCreatedRoom, problemMessage, withRoom } from "./_models/room";
import { API_ORIGIN } from "./apiOrigin";

export type ShareResult =
	| { readonly kind: "shared"; readonly shareUrl: string }
	| { readonly kind: "rejected"; readonly message: string }
	| { readonly kind: "failed" };

export const shareBoard = async (dsl: string): Promise<ShareResult> => {
	const requestUrl = new URL("/rooms", API_ORIGIN);
	try {
		const res = await fetch(requestUrl, {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ data: await encodeDslToParam(dsl) }),
		});
		const body: unknown = await res.json().catch(() => null);
		if (res.status === 400) {
			return {
				kind: "rejected",
				message: problemMessage(body) ?? "ボードを共有できませんでした",
			};
		}
		const room = res.status === 201 ? parseCreatedRoom(body) : null;
		if (!room) return { kind: "failed" };
		history.replaceState(null, "", withRoom(location.href, room.id));
		return {
			kind: "shared",
			shareUrl: new URL(room.shareUrl, requestUrl).href,
		};
	} catch {
		return { kind: "failed" };
	}
};
