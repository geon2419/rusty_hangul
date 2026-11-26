import { describe, bench as vitestBench } from "vitest";

import { benchmarks, verifyLanes } from "./lanes";

const benchOption = {
	iterations: 10000,
	time: 0,
};

verifyLanes();

for (const benchmark of benchmarks) {
	describe(benchmark.label, () => {
		for (const lane of benchmark.lanes) {
			vitestBench(lane.name, () => lane.run(), benchOption);
		}
	});
}
