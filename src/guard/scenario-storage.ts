import * as fs from "fs";
import * as path from "path";

export class ScenarioStorage {
  constructor(private secondaryRepoPath: string) {
    if (!fs.existsSync(this.secondaryRepoPath)) {
      fs.mkdirSync(this.secondaryRepoPath, { recursive: true });
    }
  }

  store(scenarioName: string, content: string): void {
    const fileName = `${this.normalizeName(scenarioName)}.yaml`;
    const filePath = path.join(this.secondaryRepoPath, fileName);
    
    fs.writeFileSync(filePath, content, "utf-8");
    console.log(`Scenario stored: ${filePath}`);
  }

  private normalizeName(name: string): string {
    return name.toLowerCase().replace(/\s+/g, "-").replace(/[^a-z0-9-]/g, "");
  }
}
