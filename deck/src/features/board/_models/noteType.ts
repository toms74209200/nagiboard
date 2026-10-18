export type NoteType =
	| "event"
	| "command"
	| "actor"
	| "aggregate"
	| "policy"
	| "readmodel"
	| "system"
	| "hotspot";

export type NoteTypeMeta = {
	readonly label: string;
	readonly color: string;
	readonly oklch: string;
	readonly textColor: string;
	readonly width: number;
	readonly height: number;
};

const NOTE_WIDTH = 130;
const NOTE_HEIGHT = 84;

export const NOTE_TYPES: Record<NoteType, NoteTypeMeta> = {
	event: {
		label: "ドメインイベント",
		color: "#ffb798",
		oklch: "oklch(0.84 0.094 45)",
		textColor: "#2e2e2e",
		width: NOTE_WIDTH,
		height: NOTE_HEIGHT,
	},
	command: {
		label: "コマンド",
		color: "#a2cfff",
		oklch: "oklch(0.84 0.082 250)",
		textColor: "#2e2e2e",
		width: NOTE_WIDTH,
		height: NOTE_HEIGHT,
	},
	actor: {
		label: "アクター",
		color: "#f9e361",
		oklch: "oklch(0.91 0.15 100)",
		textColor: "#2e2e2e",
		width: NOTE_WIDTH,
		height: NOTE_HEIGHT,
	},
	aggregate: {
		label: "集約",
		color: "#f3c443",
		oklch: "oklch(0.84 0.15 88)",
		textColor: "#2e2e2e",
		width: NOTE_WIDTH,
		height: NOTE_HEIGHT,
	},
	policy: {
		label: "ポリシー",
		color: "#d9bbfe",
		oklch: "oklch(0.84 0.098 305)",
		textColor: "#2e2e2e",
		width: NOTE_WIDTH,
		height: NOTE_HEIGHT,
	},
	readmodel: {
		label: "リードモデル",
		color: "#89e29d",
		oklch: "oklch(0.84 0.13 150)",
		textColor: "#2e2e2e",
		width: NOTE_WIDTH,
		height: NOTE_HEIGHT,
	},
	system: {
		label: "外部システム",
		color: "#ffafd1",
		oklch: "oklch(0.84 0.102 352)",
		textColor: "#2e2e2e",
		width: NOTE_WIDTH,
		height: NOTE_HEIGHT,
	},
	hotspot: {
		label: "ホットスポット",
		color: "#c13a46",
		oklch: "oklch(0.55 0.17 20)",
		textColor: "#ffffff",
		width: 110,
		height: 110,
	},
};

export const NOTE_TYPE_ORDER: readonly NoteType[] = [
	"event",
	"command",
	"actor",
	"aggregate",
	"policy",
	"readmodel",
	"system",
	"hotspot",
];

export const CONNECTION_RULES: Record<NoteType, readonly NoteType[]> = {
	actor: ["command"],
	command: ["aggregate", "system"],
	aggregate: ["event"],
	system: ["event"],
	event: ["policy", "readmodel"],
	policy: ["command"],
	readmodel: ["actor"],
	hotspot: [],
};

export const ID_PREFIX: Record<NoteType, string> = {
	event: "e",
	command: "c",
	actor: "a",
	aggregate: "g",
	policy: "p",
	readmodel: "r",
	system: "s",
	hotspot: "h",
};

export const isAllowedConnection = (from: NoteType, to: NoteType): boolean =>
	CONNECTION_RULES[from].includes(to);

export const parseNoteType = (value: string): NoteType | null =>
	Object.hasOwn(NOTE_TYPES, value) ? (value as NoteType) : null;
