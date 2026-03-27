import { ScenarioMapping } from "./config";
import * as yaml from "js-yaml";

export class ScenarioGenerator {
  generate(mapping: ScenarioMapping): string {
    const isDefault = mapping.acceptance_criteria.length === 1 && 
                     mapping.acceptance_criteria[0] === "Verify the core functionality of the new module.";
    
    const scenario = {
      name: mapping.scenario_name,
      description: isDefault 
        ? `[DISCOVERY TEMPLATE] Automatically generated scenario for ${mapping.path}`
        : `Automatically generated scenario for ${mapping.path}`,
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
