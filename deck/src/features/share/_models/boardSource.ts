export type BoardSource =
	| { readonly kind: "data"; readonly param: string }
	| { readonly kind: "room"; readonly id: string }
	| { readonly kind: "empty" };

export const boardSourceOf = (search: string): BoardSource => {
	const params = new URLSearchParams(search);
	const data = params.get("data");
	if (data) return { kind: "data", param: data };
	const room = params.get("room");
	if (room) return { kind: "room", id: room };
	return { kind: "empty" };
};
