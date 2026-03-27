import { Guard } from "../guard";
import * as fs from "fs";
import * as path from "path";
import * as yaml from "js-yaml";

describe("Guard Complete Flow", () => {
  const configPath = path.join(process.cwd(), "qtip-guard-complete.yaml");
  const testDir = path.join(process.cwd(), "test-complete-sut");
  const secondaryRepo = path.join(process.cwd(), "test-complete-secondary");
  const testFile = path.join(testDir, "auth.ts");

  beforeEach(() => {
    if (!fs.existsSync(testDir)) fs.mkdirSync(testDir);
    if (!fs.existsSync(secondaryRepo)) fs.mkdirSync(secondaryRepo);
  });

  afterEach(() => {
    if (fs.existsSync(configPath)) fs.unlinkSync(configPath);
    if (fs.existsSync(testFile)) fs.unlinkSync(testFile);
    if (fs.existsSync(testDir)) fs.rmdirSync(testDir);
    // Cleanup secondaryRepo recursively
    if (fs.existsSync(secondaryRepo)) {
      const files = fs.readdirSync(secondaryRepo);
      for (const file of files) {
        fs.unlinkSync(path.join(secondaryRepo, file));
      }
      fs.rmdirSync(secondaryRepo);
    }
  });

  it("should generate and store a scenario when a file changes", (done) => {
    const config = {
      secondary_repo: secondaryRepo,
      watched_paths: [testDir],
      scenarios_mapping: [
        {
          path: "test-complete-sut/auth.ts",
          scenario_name: "Auth Flow",
          acceptance_criteria: ["User can login"]
        }
      ]
    };
    fs.writeFileSync(configPath, yaml.dump(config));
    fs.writeFileSync(testFile, "initial");

    const guard = new Guard(configPath);
    guard.start();

    // Overriding the private generate method to signal done or just watch the fs
    const scenarioFile = path.join(secondaryRepo, "auth-flow.yaml");

    // Wait for chokidar and then trigger change
    setTimeout(() => {
      fs.writeFileSync(testFile, "changed");
      
      // Wait for generation and storage
      setTimeout(async () => {
        try {
          expect(fs.existsSync(scenarioFile)).toBe(true);
          const content = fs.readFileSync(scenarioFile, "utf-8");
          expect(content).toContain("name: Auth Flow");
          expect(content).toContain("- User can login");
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
