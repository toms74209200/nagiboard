import { useSignalEffect } from "@preact/signals";
import { useRef, useState } from "preact/hooks";
import { showToast } from "../../components/toast";
import { board } from "../board/store";
import { applyDslText, dslText } from "./dslText";

export const DslPanel = () => {
	const textareaRef = useRef<HTMLTextAreaElement>(null);
	const fileInputRef = useRef<HTMLInputElement>(null);
	const [errors, setErrors] = useState<readonly string[]>([]);

	useSignalEffect(() => {
		const text = dslText.value;
		const textarea = textareaRef.current;
		if (textarea && document.activeElement !== textarea) {
			textarea.value = text;
		}
	});

	const apply = (text: string) => {
		const result = applyDslText(text);
		setErrors(result.errors);
		if (result.errors.length) {
			showToast(
				result.errors[0] +
					(result.errors.length > 1
						? ` (他${result.errors.length - 1}件)`
						: ""),
				"error",
			);
			return;
		}
		showToast(
			result.coerced
				? `適用しました(ルール外の矢印 ${result.coerced} 本を点線に変換)`
				: "適用しました",
		);
	};

	const copyToClipboard = async () => {
		try {
			await navigator.clipboard.writeText(dslText.peek());
			showToast("クリップボードにコピーしました");
		} catch {
			textareaRef.current?.select();
			showToast(
				"コピーできませんでした。全選択したので⌘Cでコピーしてください",
				"error",
			);
		}
	};

	const download = () => {
		const name = (board.peek().title.trim() || "eventstorming").replace(
			/[\\/:*?"<>|\s]+/g,
			"_",
		);
		const a = document.createElement("a");
		a.href = URL.createObjectURL(
			new Blob([dslText.peek()], { type: "text/plain" }),
		);
		a.download = `${name}.es.txt`;
		a.click();
		URL.revokeObjectURL(a.href);
	};

	const openFile = async (ev: Event) => {
		const input = ev.currentTarget as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) return;
		const text = await file.text();
		if (textareaRef.current) textareaRef.current.value = text;
		apply(text);
		input.value = "";
	};

	return (
		<aside id="dslPanel">
			<header>
				<h2>DSL (plain text)</h2>
				<div class="spacer" />
				<button
					type="button"
					class="btn"
					onClick={() => apply(textareaRef.current?.value ?? "")}
				>
					テキストを適用
				</button>
			</header>
			<textarea id="dslTextarea" ref={textareaRef} spellcheck={false} />
			<div id="dslStatus">
				{errors.length
					? errors.join(" / ")
					: `付箋 ${board.value.notes.length} / 矢印 ${board.value.edges.length}`}
			</div>
			<footer id="dslFooter">
				<button
					type="button"
					class="btn"
					onClick={() => void copyToClipboard()}
				>
					コピー
				</button>
				<button type="button" class="btn" onClick={download}>
					ダウンロード (.es.txt)
				</button>
				<button
					type="button"
					class="btn"
					onClick={() => fileInputRef.current?.click()}
				>
					ファイルを開く
				</button>
				<input
					type="file"
					ref={fileInputRef}
					accept=".txt,.es,.dsl,text/plain"
					hidden
					onChange={(ev) => void openFile(ev)}
				/>
			</footer>
		</aside>
	);
};
