import { ScenarioMapping } from "./config";
import * as yaml from "js-yaml";

export class ScenarioGenerator {
  generate(mapping: ScenarioMapping): string {
    const scenario = {
      name: mapping.scenario_name,
      description: `Automatically generated scenario for ${mapping.path}`,
      steps: [
        {
          check: "Acceptance Criteria Validation",
          criteria: mapping.acceptance_criteria,
        },
      ],
    };

    return yaml.dump(scenario);
  }
}
