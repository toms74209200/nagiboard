export type CreatedRoom = { readonly id: string; readonly shareUrl: string };

const isRecord = (value: unknown): value is Record<string, unknown> =>
	typeof value === "object" && value !== null;

export const parseCreatedRoom = (body: unknown): CreatedRoom | null => {
	if (!isRecord(body)) return null;
	const { id, shareUrl } = body;
	return typeof id === "string" && typeof shareUrl === "string"
		? { id, shareUrl }
		: null;
};

export const problemMessage = (body: unknown): string | null => {
	if (!isRecord(body)) return null;
	const [first] = Array.isArray(body.diagnostics) ? body.diagnostics : [];
	if (isRecord(first) && typeof first.message === "string")
		return first.message;
	return typeof body.detail === "string" ? body.detail : null;
};

export const firstBoardId = (body: unknown): string | null => {
	if (!isRecord(body) || !Array.isArray(body.boards)) return null;
	const [first] = body.boards;
	return typeof first === "string" ? first : null;
};

export const withRoom = (href: string, id: string): string => {
	const url = new URL(href);
	url.searchParams.set("room", id);
	return url.href;
};
