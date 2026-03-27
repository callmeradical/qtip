import { ScenarioGenerator } from "../guard/scenario-generator";
import { ScenarioMappingSchema } from "../guard/config";

describe("ScenarioGenerator", () => {
  it("should generate a valid qtip scenario YAML from a mapping", () => {
    const mapping = {
      path: "src/auth.ts",
      scenario_name: "Auth Flow",
      acceptance_criteria: ["User can login", "Invalid login fails"]
    };

    const generator = new ScenarioGenerator();
    const yamlOutput = generator.generate(mapping);

    expect(yamlOutput).toContain("name: Auth Flow");
    expect(yamlOutput).toContain("- User can login");
    expect(yamlOutput).toContain("- Invalid login fails");
  });
});
