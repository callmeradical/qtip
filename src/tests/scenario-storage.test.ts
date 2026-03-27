import { ScenarioStorage } from "../guard/scenario-storage";
import * as fs from "fs";
import * as path from "path";

describe("ScenarioStorage", () => {
  const secondaryRepo = path.join(process.cwd(), "test-secondary-repo");
  const scenarioName = "Auth Flow";
  const scenarioContent = "name: Auth Flow\nsteps: []";

  beforeEach(() => {
    if (!fs.existsSync(secondaryRepo)) {
      fs.mkdirSync(secondaryRepo);
    }
  });

  afterEach(() => {
    const filePath = path.join(secondaryRepo, "auth-flow.yaml");
    if (fs.existsSync(filePath)) {
      fs.unlinkSync(filePath);
    }
    if (fs.existsSync(secondaryRepo)) {
      fs.rmdirSync(secondaryRepo);
    }
  });

  it("should write scenario content to the secondary repository", () => {
    const storage = new ScenarioStorage(secondaryRepo);
    storage.store(scenarioName, scenarioContent);

    const filePath = path.join(secondaryRepo, "auth-flow.yaml");
    expect(fs.existsSync(filePath)).toBe(true);
    expect(fs.readFileSync(filePath, "utf-8")).toBe(scenarioContent);
  });
});
