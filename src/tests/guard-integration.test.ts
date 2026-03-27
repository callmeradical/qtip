import { FileWatcher } from "../guard/file-watcher";
import { ConfigLoader } from "../guard/config-loader";
import * as fs from "fs";
import * as path from "path";
import * as yaml from "js-yaml";

describe("Guard Integration - Task 2", () => {
  const configPath = path.join(process.cwd(), "qtip-guard-test.yaml");
  const testDir = path.join(process.cwd(), "test-watch-integration");

  beforeEach(() => {
    if (!fs.existsSync(testDir)) {
      fs.mkdirSync(testDir);
    }
  });

  afterEach(() => {
    if (fs.existsSync(configPath)) {
      fs.unlinkSync(configPath);
    }
    // Clean up testDir recursively if needed, but here we just rmdir
    if (fs.existsSync(testDir)) {
      fs.rmdirSync(testDir);
    }
  });

  it("should initialize FileWatcher with paths from ConfigLoader", () => {
    const config = {
      secondary_repo: "./scenarios-repo",
      watched_paths: [testDir],
      scenarios_mapping: []
    };
    fs.writeFileSync(configPath, yaml.dump(config));

    const loader = new ConfigLoader();
    const loadedConfig = loader.load(configPath);
    
    const watcher = new FileWatcher(loadedConfig.watched_paths);
    expect((watcher as any).paths).toEqual([testDir]);
  });
});
