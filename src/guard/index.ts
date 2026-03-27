import { ConfigLoader } from "./config-loader";
import { FileWatcher } from "./file-watcher";
import { GuardConfig } from "./config";
import { ScenarioGenerator } from "./scenario-generator";
import { ScenarioStorage } from "./scenario-storage";
import * as path from "path";

export class Guard {
  private configLoader: ConfigLoader;
  private watcher?: FileWatcher;
  private config?: GuardConfig;
  private generator: ScenarioGenerator;
  private storage?: ScenarioStorage;

  constructor(private configPath: string) {
    this.configLoader = new ConfigLoader();
    this.generator = new ScenarioGenerator();
  }

  start(): void {
    console.log(`Starting qtip-guard with config: ${this.configPath}`);
    this.config = this.configLoader.load(this.configPath);
    
    this.storage = new ScenarioStorage(this.config.secondary_repo);
    this.watcher = new FileWatcher(this.config.watched_paths);
    
    this.watcher.start((filePath) => {
      console.log(`File changed: ${filePath}`);
      this.generate(filePath);
    });
  }

  async stop(): Promise<void> {
    if (this.watcher) {
      await this.watcher.stop();
    }
  }

  private generate(filePath: string): void {
    if (!this.config || !this.storage) return;

    // Normalize paths for comparison
    const relativePath = path.relative(process.cwd(), filePath);
    const mapping = this.config.scenarios_mapping.find(m => m.path === relativePath);

    if (mapping) {
      console.log(`Matching mapping found for ${relativePath}. Triggering generation...`);
      const yamlContent = this.generator.generate(mapping);
      this.storage.store(mapping.scenario_name, yamlContent);
    } else {
      console.log(`No mapping found for ${relativePath}. Skipping generation.`);
    }
  }
}
