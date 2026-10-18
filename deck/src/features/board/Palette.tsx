import { NOTE_TYPE_ORDER, NOTE_TYPES } from "./_models/noteType";
import { activeTool } from "./store";

export const Palette = () => (
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
