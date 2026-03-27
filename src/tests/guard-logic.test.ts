import { Guard } from "../guard";
import { ConfigLoader } from "../guard/config-loader";
import { FileWatcher } from "../guard/file-watcher";
import * as fs from "fs";
import * as path from "path";
import * as yaml from "js-yaml";

describe("Guard Logic", () => {
  const configPath = path.join(process.cwd(), "qtip-guard-logic.yaml");
  const testDir = path.join(process.cwd(), "test-logic");
  const testFile = path.join(testDir, "auth.ts");

  beforeEach(() => {
    if (!fs.existsSync(testDir)) {
      fs.mkdirSync(testDir);
    }
  });

  afterEach(() => {
    if (fs.existsSync(configPath)) {
      fs.unlinkSync(configPath);
    }
    if (fs.existsSync(testFile)) {
      fs.unlinkSync(testFile);
    }
    if (fs.existsSync(testDir)) {
      fs.rmdirSync(testDir);
    }
  });

  it("should trigger generation when a watched file changes", (done) => {
    const config = {
      secondary_repo: "./scenarios-repo",
      watched_paths: [testDir],
      scenarios_mapping: [
        {
          path: "test-logic/auth.ts",
          scenario_name: "Auth Flow",
          acceptance_criteria: ["User can login"]
        }
      ]
    };
    fs.writeFileSync(configPath, yaml.dump(config));
    fs.writeFileSync(testFile, "initial");

    const guard = new Guard(configPath);
    
    // Mocking generator call for Task 3
    (guard as any).generate = jest.fn().mockImplementation(() => {
      guard.stop();
      done();
    });

    guard.start();

    // Wait a bit for chokidar to ready up
    setTimeout(() => {
      fs.writeFileSync(testFile, "changed");
    }, 100);
  });
});
