import type { Board } from "../board/_models/board";
import type { MeasureText } from "./_models/boardSvg";
import { buildBoardSvg, FONT_FAMILY } from "./_models/boardSvg";

const canvasMeasure = (): MeasureText => {
	const ctx = document.createElement("canvas").getContext("2d");
	if (!ctx) return (text, fontPx) => [...text].length * fontPx;
	return (text, fontPx) => {
		ctx.font = `${fontPx}px ${FONT_FAMILY}`;
		return ctx.measureText(text).width;
	};
};

const downloadBlob = (blob: Blob, filename: string): void => {
	const a = document.createElement("a");
	a.href = URL.createObjectURL(blob);
	a.download = filename;
	a.click();
	URL.revokeObjectURL(a.href);
};

const exportFileName = (board: Board): string =>
	(board.title.trim() || "eventstorming").replace(/[\\/:*?"<>|\s]+/g, "_");

export const downloadBoardSvg = (board: Board): boolean => {
	const built = buildBoardSvg(board, canvasMeasure());
	if (!built) return false;
	downloadBlob(
		new Blob([built.svg], { type: "image/svg+xml" }),
		`${exportFileName(board)}.svg`,
	);
	return true;
};

export const downloadBoardPng = async (
	board: Board,
	scale = 2,
): Promise<boolean> => {
	const built = buildBoardSvg(board, canvasMeasure());
	if (!built) return false;
	const url = URL.createObjectURL(
		new Blob([built.svg], { type: "image/svg+xml" }),
	);
	try {
		const img = new Image();
		await new Promise<void>((resolve, reject) => {
			img.onload = () => resolve();
			img.onerror = () => reject(new Error("SVG画像の読み込みに失敗"));
			img.src = url;
		});
		const canvas = document.createElement("canvas");
		canvas.width = Math.round(built.width * scale);
		canvas.height = Math.round(built.height * scale);
		const ctx = canvas.getContext("2d");
		if (!ctx) return false;
		ctx.scale(scale, scale);
		ctx.drawImage(img, 0, 0);
		const blob = await new Promise<Blob | null>((resolve) =>
			canvas.toBlob(resolve, "image/png"),
		);
		if (!blob) return false;
		downloadBlob(blob, `${exportFileName(board)}.png`);
		return true;
	} finally {
		URL.revokeObjectURL(url);
	}
};
