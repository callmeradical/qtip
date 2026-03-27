import * as fs from "fs";
import * as yaml from "js-yaml";
import { GuardConfig, GuardConfigSchema } from "./config";

export class ConfigLoader {
  load(configPath: string): GuardConfig {
    if (!fs.existsSync(configPath)) {
      throw new Error(`Configuration file not found: ${configPath}`);
    }

    const fileContent = fs.readFileSync(configPath, "utf-8");
    let parsed: unknown;
    try {
      parsed = yaml.load(fileContent);
    } catch (e: any) {
      throw new Error(`Failed to parse YAML configuration: ${e.message}`);
    }

    const result = GuardConfigSchema.safeParse(parsed);
    if (!result.success) {
      throw new Error(`Invalid configuration: ${result.error.message}`);
    }

    return result.data;
  }
}
