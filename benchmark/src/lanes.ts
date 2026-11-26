import { readFileSync } from "node:fs";

import * as esHangul from "es-hangul";

import { Hangul } from "../../node/index";
import {
	disassemble as wasmDisassemble,
	getChoseong as wasmGetChoseong,
	initSync as initWasm,
} from "../../wasm/pkg/hangul.js";
import scenarios from "../fixtures/scenarios.json";

export interface Scenario {
	id: string;
	label: string;
	text: string;
}

export interface Lane {
	name: string;
	run: (text: string) => void;
}

initWasm({
	module: readFileSync(new URL("../../wasm/pkg/hangul_bg.wasm", import.meta.url)),
});

export const fixtureScenarios = scenarios as Scenario[];

export const lanes: Lane[] = [
	{
		name: "es-hangul",
		run: (text) => {
			esHangul.disassemble(text);
			esHangul.getChoseong(text);
		},
	},
	{
		name: "rusty-hangul/node",
		run: (text) => {
			const hangul = new Hangul(text);
			hangul.disassemble();
			hangul.getChoseong();
		},
	},
	{
		name: "rusty-hangul/wasm",
		run: (text) => {
			wasmDisassemble(text);
			wasmGetChoseong(text);
		},
	},
];

// Fail fast when a stale or broken wasm build would make the timings meaningless.
export function verifyLanes(fixtures: Scenario[]) {
	for (const { text } of fixtures) {
		const expected = new Hangul(text);

		if (wasmDisassemble(text) !== expected.disassemble()) {
			throw new Error(`wasm disassemble mismatch: ${text}`);
		}
		if (wasmGetChoseong(text) !== expected.getChoseong()) {
			throw new Error(`wasm getChoseong mismatch: ${text}`);
		}
	}
}
