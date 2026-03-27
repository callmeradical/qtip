import { ConfigLoader } from "../guard/config-loader";
import * as fs from "fs";
import * as path from "path";
import * as yaml from "js-yaml";

describe("ConfigLoader", () => {
  const configPath = path.join(process.cwd(), "qtip-guard.yaml");

  afterEach(() => {
    if (fs.existsSync(configPath)) {
      fs.unlinkSync(configPath);
    }
  });

  it("should load and parse a valid YAML config file", () => {
    const validConfig = {
      secondary_repo: "./scenarios-repo",
      watched_paths: ["src/**/*.ts"],
      scenarios_mapping: [
        {
          path: "src/auth.ts",
          scenario_name: "Auth Flow",
          acceptance_criteria: ["User can login"]
        }
      ]
    };
    fs.writeFileSync(configPath, yaml.dump(validConfig));

    const loader = new ConfigLoader();
    const config = loader.load(configPath);

    expect(config).toEqual(validConfig);
  });

  it("should throw an error if the config file is missing", () => {
    const loader = new ConfigLoader();
    expect(() => loader.load("non-existent.yaml")).toThrow();
  });

  it("should throw an error if the config file is invalid", () => {
    fs.writeFileSync(configPath, "invalid: yaml: content:");
    const loader = new ConfigLoader();
    expect(() => loader.load(configPath)).toThrow();
  });
});
