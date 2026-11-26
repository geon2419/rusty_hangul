import { readFileSync } from "node:fs";

import * as esHangul from "es-hangul";

import { assemble as nodeAssemble, Hangul } from "../../node/index";
import * as wasm from "../../wasm/pkg/hangul.js";
import scenarioFixtures from "../fixtures/scenarios.json";
import workloadFixtures from "../fixtures/workloads.json";

export interface Scenario {
	id: string;
	label: string;
	text: string;
}

export interface WorkloadFixture {
	id: string;
	label: string;
	api?: string;
	scenarios: string[];
	args?: string[];
}

export type LaneName = "es-hangul" | "rusty-hangul/node" | "rusty-hangul/wasm";

export interface Lane {
	name: LaneName;
	run: () => unknown;
}

export interface Benchmark {
	label: string;
	lanes: Lane[];
}

wasm.initSync({
	module: readFileSync(new URL("../../wasm/pkg/hangul_bg.wasm", import.meta.url)),
});

const scenarios = scenarioFixtures as Scenario[];
const workloads = workloadFixtures as WorkloadFixture[];
const scenarioById = new Map(scenarios.map((scenario) => [scenario.id, scenario]));

// es-hangul assembles fragments instead of a single string, so the split is precomputed
// to keep the comparison about assembly rather than about string iteration.
const fragmentCache = new Map<string, string[]>();
const fragmentsOf = (text: string) => {
	let fragments = fragmentCache.get(text);
	if (!fragments) {
		fragments = Array.from(text);
		fragmentCache.set(text, fragments);
	}
	return fragments;
};

type Runner = (text: string, arg: string) => unknown;

const runners: Record<string, Partial<Record<LaneName, Runner>>> = {
	"disassemble-choseong": {
		"es-hangul": (text) => [esHangul.disassemble(text), esHangul.getChoseong(text)],
		"rusty-hangul/node": (text) => {
			const hangul = new Hangul(text);
			return [hangul.disassemble(), hangul.getChoseong()];
		},
		"rusty-hangul/wasm": (text) => [wasm.disassemble(text), wasm.getChoseong(text)],
	},
	"disassemble-groups": {
		"es-hangul": (text) => esHangul.disassembleToGroups(text),
		"rusty-hangul/node": (text) => new Hangul(text).disassembleToGroups(),
		"rusty-hangul/wasm": (text) => wasm.disassembleToGroups(text),
	},
	"has-batchim": {
		"es-hangul": (text) => esHangul.hasBatchim(text),
		"rusty-hangul/node": (text) => new Hangul(text).hasBatchim(),
		"rusty-hangul/wasm": (text) => wasm.hasBatchim(text),
	},
	josa: {
		"es-hangul": (text, arg) => esHangul.josa(text, arg as Parameters<typeof esHangul.josa>[1]),
		"rusty-hangul/node": (text, arg) => new Hangul(text).josa(arg as "을/를"),
		"rusty-hangul/wasm": (text, arg) => wasm.josa(text, arg),
	},
	assemble: {
		"es-hangul": (text) => esHangul.assemble(fragmentsOf(text)),
		"rusty-hangul/node": (text) => nodeAssemble(text),
		"rusty-hangul/wasm": (text) => wasm.assemble(text),
	},
	// es-hangul has no choseong search, so these workloads only run the rusty-hangul lanes.
	"contains-choseong": {
		"rusty-hangul/node": (text, arg) => new Hangul(text).containsChoseong(arg),
		"rusty-hangul/wasm": (text, arg) => wasm.containsChoseong(text, arg),
	},
	"find-choseong": {
		"rusty-hangul/node": (text, arg) => new Hangul(text).findChoseong(arg),
		"rusty-hangul/wasm": (text, arg) => {
			const match = wasm.findChoseong(text, arg);
			if (!match) return null;
			const { start, end } = match;
			match.free();
			return { start, end };
		},
	},
};

// Criterion group names mirror these labels, which is what merges both harnesses into
// a single report.
const labelOf = (workload: WorkloadFixture, scenario: Scenario) =>
	`${workload.label} · ${scenario.label}`;

export const benchmarks: Benchmark[] = workloads.flatMap((workload) => {
	const workloadRunners = runners[workload.id];
	if (!workloadRunners) {
		throw new Error(`unknown workload id in fixtures/workloads.json: ${workload.id}`);
	}

	return workload.scenarios.map((scenarioId) => {
		const scenario = scenarioById.get(scenarioId);
		if (!scenario) {
			throw new Error(`unknown scenario id ${scenarioId} used by workload ${workload.id}`);
		}

		const arg = workload.args?.[0] ?? "";
		const lanes: Lane[] = Object.entries(workloadRunners).map(([name, run]) => ({
			name: name as LaneName,
			run: () => run(scenario.text, arg),
		}));

		if (lanes.length === 0) {
			throw new Error(`workload ${workload.id} has no lanes`);
		}

		return { label: labelOf(workload, scenario), lanes };
	});
});

const normalize = (value: unknown) =>
	JSON.stringify(value, (_key, item) =>
		typeof item === "object" && item !== null && "start" in item && "end" in item
			? { start: item.start, end: item.end }
			: item,
	);

// Fail fast when a stale or broken wasm build would make the timings meaningless.
export function verifyLanes(benchmarksToVerify: Benchmark[] = benchmarks) {
	for (const benchmark of benchmarksToVerify) {
		const nodeLane = benchmark.lanes.find((lane) => lane.name === "rusty-hangul/node");
		const wasmLane = benchmark.lanes.find((lane) => lane.name === "rusty-hangul/wasm");

		if (!nodeLane || !wasmLane) continue;

		const expected = normalize(nodeLane.run());
		const actual = normalize(wasmLane.run());

		if (expected !== actual) {
			throw new Error(`wasm mismatch in ${benchmark.label}: ${expected} !== ${actual}`);
		}
	}
}
