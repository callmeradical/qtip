import { ConfigLoader } from "./config-loader";
import { FileWatcher } from "./file-watcher";
import { GuardConfig } from "./config";
import * as path from "path";

export class Guard {
  private configLoader: ConfigLoader;
  private watcher?: FileWatcher;
  private config?: GuardConfig;

  constructor(private configPath: string) {
    this.configLoader = new ConfigLoader();
  }

  start(): void {
    console.log(`Starting qtip-guard with config: ${this.configPath}`);
    this.config = this.configLoader.load(this.configPath);
    
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
    // Logic for triggering generation based on config mapping
    if (!this.config) return;

    // Normalize paths for comparison
    const relativePath = path.relative(process.cwd(), filePath);
    const mapping = this.config.scenarios_mapping.find(m => m.path === relativePath);

    if (mapping) {
      console.log(`Matching mapping found for ${relativePath}. Triggering generation...`);
      // Generation logic will be implemented in Phase 3
    } else {
      console.log(`No mapping found for ${relativePath}. Skipping generation.`);
    }
  }
}
