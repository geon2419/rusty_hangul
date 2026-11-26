#!/usr/bin/env bun
import { spawnSync } from "node:child_process";
import { existsSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import scenarios from "../fixtures/scenarios.json";
import workloads from "../fixtures/workloads.json";

const benchmarkDir = fileURLToPath(new URL("..", import.meta.url));
const repoRoot = fileURLToPath(new URL("../..", import.meta.url));
const criterionDir = join(repoRoot, "target", "criterion");

const CORE_LANE = "rusty-hangul/core";
const JS_LANES = ["rusty-hangul/node", "rusty-hangul/wasm", "es-hangul"] as const;
const LANES = [CORE_LANE, ...JS_LANES] as const;

type LaneMetrics = Partial<Record<(typeof LANES)[number], number>>;

interface Row {
	label: string;
	workloadId: string;
	scenarioId: string;
	api?: string;
	opsPerSecond: LaneMetrics;
}

// Both harnesses name their benchmarks after these fixture labels, so the report owns
// the row set and can ignore leftover criterion groups from earlier revisions.
const rows: Row[] = workloads.flatMap((workload) =>
	workload.scenarios.map((scenarioId) => {
		const scenario = scenarios.find((candidate) => candidate.id === scenarioId);

		if (!scenario) {
			fail(`fixtures/workloads.json references an unknown scenario id: ${scenarioId}`);
		}

		return {
			label: `${workload.label} · ${scenario.label}`,
			workloadId: workload.id,
			scenarioId: scenario.id,
			api: workload.api,
			opsPerSecond: {},
		};
	}),
);

const args = process.argv.slice(2);
const rustOnly = args.includes("--rust-only");
const jsOnly = args.includes("--js-only");
const verbose = args.includes("--verbose");
const jsonIndex = args.indexOf("--json");
const jsonPath = jsonIndex >= 0 ? args[jsonIndex + 1] : undefined;

if (rustOnly && jsOnly) {
	fail("--rust-only and --js-only cannot be combined");
}

function fail(message: string): never {
	console.error(`report: ${message}`);
	process.exit(1);
}

function run(command: string, commandArgs: string[], cwd: string) {
	const result = spawnSync(command, commandArgs, {
		cwd,
		encoding: "utf8",
		env: { ...process.env, FORCE_COLOR: "0", NO_COLOR: "1" },
	});

	const output = `${result.stdout ?? ""}${result.stderr ?? ""}`;

	if (verbose || result.status !== 0) {
		process.stderr.write(output);
	}

	if (result.status !== 0) {
		fail(`\`${command} ${commandArgs.join(" ")}\` failed with exit code ${result.status}`);
	}

	return output;
}

// Benchmarks need the native addon and the wasm package; build them when a fresh
// checkout would otherwise crash before measuring anything.
function ensureArtifacts() {
	if (!existsSync(join(repoRoot, "wasm", "pkg", "hangul_bg.wasm"))) {
		console.error("report: building wasm package");
		run("bun", ["run", "wasm:build"], repoRoot);
	}

	const hasNativeAddon = readdirSync(join(repoRoot, "node")).some((name) => name.endsWith(".node"));

	if (!hasNativeAddon) {
		console.error("report: building node addon");
		run("bun", ["run", "node:build"], repoRoot);
	}
}

function collectCriterion() {
	// criterion nests results as <workload id>/<scenario id>/new/estimates.json, which keeps
	// the paths ASCII and short no matter how the display labels are worded.
	for (const row of rows) {
		const path = join(criterionDir, row.workloadId, row.scenarioId, "new", "estimates.json");

		if (!existsSync(path)) {
			if (verbose) {
				console.error(`report: no criterion result for ${row.workloadId}/${row.scenarioId}`);
			}
			continue;
		}

		const meanNanoseconds: number = JSON.parse(readFileSync(path, "utf8")).mean.point_estimate;
		row.opsPerSecond[CORE_LANE] = 1e9 / meanNanoseconds;
	}

	if (verbose) {
		const known = new Set(rows.map((row) => row.workloadId));
		for (const entry of readdirSync(criterionDir, { withFileTypes: true })) {
			if (entry.isDirectory() && !known.has(entry.name)) {
				console.error(`report: ignoring leftover criterion group: ${entry.name}`);
			}
		}
	}
}

const groupHeader = /^\s*✓\s+\S+\s+>\s+(.+?)(?:\s+\d+(?:\.\d+)?(?:ms|s))?\s*$/;
const laneRow = /^\s*·\s+(.+?)\s+([\d,]+\.\d+)\s+/;

function collectVitest(output: string) {
	let currentLabel: string | undefined;

	for (const line of output.split("\n")) {
		const header = groupHeader.exec(line);
		if (header) {
			currentLabel = header[1];
			continue;
		}

		const row = laneRow.exec(line);
		if (!row || !currentLabel) continue;

		const lane = row[1] as (typeof JS_LANES)[number];
		if (!JS_LANES.includes(lane)) continue;

		const metrics = findRow(currentLabel);
		if (!metrics) continue;

		metrics[lane] = Number(row[2].replaceAll(",", ""));
	}
}

function findRow(label: string) {
	const row = rows.find((candidate) => candidate.label === label);

	if (!row && verbose) {
		console.error(`report: ignoring benchmark outside the current fixtures: ${label}`);
	}

	return row?.opsPerSecond;
}

function formatOps(ops: number | undefined) {
	if (ops === undefined) return "-";
	if (ops >= 1e6) return `${(ops / 1e6).toFixed(2)}M`;
	if (ops >= 1e3) return `${(ops / 1e3).toFixed(1)}K`;
	return ops.toFixed(0);
}

function printReport() {
	const measured = rows.filter((row) => Object.keys(row.opsPerSecond).length > 0);
	const presentLanes = LANES.filter((lane) =>
		measured.some((row) => row.opsPerSecond[lane] !== undefined),
	);

	const header = ["Benchmark", ...presentLanes.map((lane) => lane.replace("rusty-hangul/", ""))];
	const table = [
		`| ${header.join(" | ")} |`,
		`| ${header.map((_, index) => (index === 0 ? "---" : "---:")).join(" | ")} |`,
		...measured.map(
			(row) =>
				`| ${[
					row.label,
					...presentLanes.map((lane) => formatOps(row.opsPerSecond[lane])),
				].join(" | ")} |`,
		),
	];

	console.log("\nMeasured mean throughput, operations per second (higher is better).\n");
	console.log(table.join("\n"));
}

function main() {
	if (!jsOnly) {
		console.error("report: running cargo bench");
		run("cargo", ["bench", "-p", "rusty_hangul_benchmark"], repoRoot);

		if (!existsSync(criterionDir)) {
			fail(`criterion output missing: ${criterionDir}`);
		}
		collectCriterion();
	}

	if (!rustOnly) {
		ensureArtifacts();
		console.error("report: running vitest bench");
		collectVitest(run(process.execPath, ["x", "vitest", "bench"], benchmarkDir));
	}

	const measured = rows.filter((row) => Object.keys(row.opsPerSecond).length > 0);

	if (measured.length === 0) {
		fail("no benchmark results were collected");
	}

	if (measured.length !== rows.length) {
		console.error(
			`report: ${rows.length - measured.length} configured benchmark(s) produced no result`,
		);
	}

	printReport();

	if (jsonPath) {
		writeFileSync(
			jsonPath,
			`${JSON.stringify(
				{
					generatedAt: new Date().toISOString(),
					platform: `${process.platform}-${process.arch}`,
					bun: process.versions.bun,
					rows: measured,
				},
				null,
				2,
			)}\n`,
		);
		console.error(`report: wrote ${jsonPath}`);
	}
}

main();
