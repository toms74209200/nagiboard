import { defineConfig } from "orval";

export default defineConfig({
	nagiboard: {
		input: "../spec/dist/openapi.yaml",
		output: {
			target: "src/openapi/nagiboard.ts",
			client: "fetch",
			mode: "single",
		},
	},
});
