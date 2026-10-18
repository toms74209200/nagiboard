import { useRef } from "preact/hooks";
import { Topbar } from "./components/Topbar";
import { BoardCanvas } from "./features/board/BoardCanvas";
import { RulesDialog } from "./features/board/RulesDialog";
import { dslPanelVisible } from "./features/board/store";
import { DslPanel } from "./features/dsl/DslPanel";

export const App = () => {
	const rulesDialogRef = useRef<HTMLDialogElement>(null);
	return (
		<div id="layout" class={dslPanelVisible.value ? "" : "dsl-hidden"}>
			<Topbar onShowRules={() => rulesDialogRef.current?.showModal()} />
			<BoardCanvas />
			<DslPanel />
			<RulesDialog dialogRef={rulesDialogRef} />
		</div>
	);
};
