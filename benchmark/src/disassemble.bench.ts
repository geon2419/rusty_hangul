import { describe, bench as vitestBench } from "vitest";

import { fixtureScenarios, lanes, verifyLanes } from "./lanes";

const benchOption = {
	iterations: 10000,
	time: 0,
};

verifyLanes(fixtureScenarios);

for (const scenario of fixtureScenarios) {
	describe(scenario.label, () => {
		for (const lane of lanes) {
			vitestBench(lane.name, () => lane.run(scenario.text), benchOption);
		}
	});
}
