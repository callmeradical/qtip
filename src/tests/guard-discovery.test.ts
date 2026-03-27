import { Guard } from "../guard";
import * as fs from "fs";
import * as path from "path";
import * as yaml from "js-yaml";

describe("Guard Discovery Integration", () => {
  const configPath = path.join(process.cwd(), "qtip-guard-discovery.yaml");
  const testDir = path.join(process.cwd(), "test-discovery-sut");
  const secondaryRepo = path.join(process.cwd(), "test-discovery-secondary");

  beforeEach(() => {
    if (!fs.existsSync(testDir)) fs.mkdirSync(testDir);
    if (!fs.existsSync(secondaryRepo)) fs.mkdirSync(secondaryRepo);
  });

  afterEach(() => {
    if (fs.existsSync(configPath)) fs.unlinkSync(configPath);
    if (fs.existsSync(testDir)) fs.rmSync(testDir, { recursive: true, force: true });
    if (fs.existsSync(secondaryRepo)) fs.rmSync(secondaryRepo, { recursive: true, force: true });
  });

  it("should generate a scenario when a new file is added (even without mapping)", (done) => {
    const config = {
      secondary_repo: secondaryRepo,
      watched_paths: [testDir],
      scenarios_mapping: [] // No mapping for the new file
    };
    fs.writeFileSync(configPath, yaml.dump(config));

    const guard = new Guard(configPath);
    guard.start();

    const newFile = path.join(testDir, "new-feature.ts");
    const scenarioFile = path.join(secondaryRepo, "new-feature.yaml");

    // Wait for chokidar and then trigger add
    setTimeout(() => {
      fs.writeFileSync(newFile, "new feature content");
      
      // Wait for generation and storage
      setTimeout(async () => {
        try {
          expect(fs.existsSync(scenarioFile)).toBe(true);
          const content = fs.readFileSync(scenarioFile, "utf-8");
          expect(content).toContain("name: new-feature.ts");
          await guard.stop();
          done();
        } catch (e) {
          await guard.stop();
          done(e);
        }
      }, 500);
    }, 200);
  });
});
