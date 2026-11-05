import type { Signal } from "@preact/signals";
import type { NoteType } from "./_models/noteType";
import { NOTE_TYPE_ORDER, NOTE_TYPES } from "./_models/noteType";

export const Palette = ({
	activeTool,
}: {
	activeTool: Signal<NoteType | null>;
}) => (
	<div id="palette">
		{NOTE_TYPE_ORDER.map((type) => (
			<button
				key={type}
				type="button"
				class={`tool${activeTool.value === type ? " active" : ""}`}
				onClick={() => {
					activeTool.value = activeTool.peek() === type ? null : type;
				}}
			>
				<span class="chip" style={{ background: `var(--c-${type})` }} />
				{NOTE_TYPES[type].label}
				<small>{type}</small>
			</button>
		))}
	</div>
);
