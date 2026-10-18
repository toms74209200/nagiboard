import { toastMessage } from "./toast";

export const ToastView = () => {
	const message = toastMessage.value;
	return (
		<div
			id="toast"
			class={message ? `show${message.kind === "error" ? " err" : ""}` : ""}
		>
			{message?.text}
		</div>
	);
};
