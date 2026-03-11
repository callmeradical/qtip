"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.ScenarioResolver = void 0;
const scenario_1 = require("../models/scenario");
const subject_manifest_1 = require("../models/subject-manifest");
class ScenarioResolver {
    scenarios = [];
    constructor(scenarios = []) {
        this.scenarios = scenarios;
    }
    resolve(manifest) {
        return this.scenarios.filter((scenario) => {
            // Check capabilities overlap
            const hasCapability = scenario.applies_to.capabilities.some((cap) => manifest.capabilities.includes(cap));
            if (!hasCapability)
                return false;
            // Check interfaces overlap
            const manifestInterfaceTypes = manifest.interfaces.map((i) => i.type);
            const hasInterface = scenario.applies_to.interfaces.some((i) => manifestInterfaceTypes.includes(i));
            if (!hasInterface)
                return false;
            // Check environment (optional)
            if (scenario.applies_to.environments &&
                scenario.applies_to.environments.length > 0 &&
                !scenario.applies_to.environments.includes(manifest.environment)) {
                return false;
            }
            return true;
        });
    }
}
exports.ScenarioResolver = ScenarioResolver;
//# sourceMappingURL=scenario-resolver.js.map