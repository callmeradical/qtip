import { Scenario } from '../models/scenario';
import { SubjectManifest } from '../models/subject-manifest';

export class ScenarioResolver {
  private scenarios: Scenario[] = [];

  constructor(scenarios: Scenario[] = []) {
    this.scenarios = scenarios;
  }

  resolve(manifest: SubjectManifest): Scenario[] {
    return this.scenarios.filter((scenario) => {
      // Check capabilities overlap
      const hasCapability = scenario.applies_to.capabilities.some((cap) =>
        manifest.capabilities.includes(cap)
      );
      if (!hasCapability) return false;

      // Check interfaces overlap
      const manifestInterfaceTypes = manifest.interfaces.map((i) => i.type);
      const hasInterface = scenario.applies_to.interfaces.some((i) =>
        manifestInterfaceTypes.includes(i)
      );
      if (!hasInterface) return false;

      // Check environment (optional)
      if (
        scenario.applies_to.environments &&
        scenario.applies_to.environments.length > 0 &&
        !scenario.applies_to.environments.includes(manifest.environment)
      ) {
        return false;
      }

      return true;
    });
  }
}
