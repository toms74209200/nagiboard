import { memo } from "preact/compat";
import type { Edge, Note } from "./_models/board";
import { edgeKey, noteRect } from "./_models/board";
import { edgePathD } from "./_models/geometry";
import { selection } from "./store";

export const EdgeView = memo(
	({ edge, from, to }: { edge: Edge; from: Note; to: Note }) => {
		const key = edgeKey(edge);
		const isSelected = selection.value.edgeKeys.has(key);
		const d = edgePathD(noteRect(from), noteRect(to));
		return (
			<g class={`edge${isSelected ? " selected" : ""}`} data-edge={key}>
				<path class="edge-hit" d={d} />
				<path
					class={`edge-line${edge.dashed ? " dashed" : ""}`}
					d={d}
					marker-end={
						isSelected ? "url(#arrowhead-selected)" : "url(#arrowhead)"
					}
				/>
			</g>
		);
	},
);
