import type { RefObject } from "preact";
import { ADJACENT_GAP } from "./_models/board";
import {
	CONNECTION_RULES,
	NOTE_TYPE_ORDER,
	NOTE_TYPES,
} from "./_models/noteType";

export const RulesDialog = ({
	dialogRef,
}: {
	dialogRef: RefObject<HTMLDialogElement>;
}) => (
	<dialog ref={dialogRef}>
		<h3>イベントストーミングの接続ルール</h3>
		<p>
			実線の矢印(<code>-&gt;</code>)は以下の組み合わせのみ許可されます。
			それ以外は自動的に点線の例外矢印(<code>..&gt;</code>)になります。
		</p>
		<table>
			<tbody>
				<tr>
					<th>起点</th>
					<th>→ 実線で接続できる先</th>
				</tr>
				{NOTE_TYPE_ORDER.filter((type) => CONNECTION_RULES[type].length).map(
					(from) => (
						<tr key={from}>
							<td>
								<span class="chip" style={{ background: `var(--c-${from})` }} />
								{NOTE_TYPES[from].label}
							</td>
							<td>
								{CONNECTION_RULES[from].map((to, i) => (
									<span key={to}>
										{i > 0 && "、 "}
										<span
											class="chip"
											style={{ background: `var(--c-${to})` }}
										/>
										{NOTE_TYPES[to].label}
									</span>
								))}
							</td>
						</tr>
					),
				)}
			</tbody>
		</table>
		<p>ホットスポット(課題)はどの付箋とも点線でのみ接続できます。</p>
		<p>
			既存の付箋のすぐ隣({ADJACENT_GAP}px 以内)にルールに合う付箋を置くと、
			実線の矢印が自動で引かれます。
		</p>
		<form method="dialog" style={{ textAlign: "right" }}>
			<button type="submit" class="btn">
				閉じる
			</button>
		</form>
	</dialog>
);
