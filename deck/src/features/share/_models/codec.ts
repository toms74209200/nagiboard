const COMPRESSION_FORMAT = "deflate-raw";

const pipeBytes = async (
	bytes: Uint8Array,
	transform: CompressionStream | DecompressionStream,
): Promise<Uint8Array> =>
	new Uint8Array(
		await new Response(
			new Blob([bytes as BlobPart]).stream().pipeThrough(transform),
		).arrayBuffer(),
	);

export const base64UrlEncode = (bytes: Uint8Array): string =>
	btoa(Array.from(bytes, (b) => String.fromCharCode(b)).join(""))
		.replace(/\+/g, "-")
		.replace(/\//g, "_")
		.replace(/=+$/, "");

export const base64UrlDecode = (s: string): Uint8Array => {
	const bin = atob(s.replace(/-/g, "+").replace(/_/g, "/"));
	return Uint8Array.from(bin, (c) => c.charCodeAt(0));
};

export const encodeDslToParam = async (dsl: string): Promise<string> =>
	base64UrlEncode(
		await pipeBytes(
			new TextEncoder().encode(dsl),
			new CompressionStream(COMPRESSION_FORMAT),
		),
	);

export const decodeParamToDsl = async (param: string): Promise<string> =>
	new TextDecoder().decode(
		await pipeBytes(
			base64UrlDecode(param),
			new DecompressionStream(COMPRESSION_FORMAT),
		),
	);
