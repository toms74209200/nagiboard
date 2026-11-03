import preact from "@preact/preset-vite";
import { defineConfig } from "vite";

// https://vite.dev/config/
export default defineConfig({
	plugins: [preact()],
	server: {
		proxy: {
			"/rooms": "http://localhost:4010",
			"/boards": "http://localhost:4010",
		},
	},
});
