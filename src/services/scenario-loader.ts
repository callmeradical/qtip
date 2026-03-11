import fs from 'fs';
import path from 'path';
import yaml from 'js-yaml';
import { Scenario, ScenarioSchema } from '../models/scenario';

export class ScenarioLoader {
  private scenariosDir: string;

  constructor(scenariosDir: string) {
    this.scenariosDir = scenariosDir;
  }

  async loadAll(): Promise<Scenario[]> {
    const scenarios: Scenario[] = [];
    if (!fs.existsSync(this.scenariosDir)) {
      return [];
    }

    const files = this.recursiveReaddir(this.scenariosDir).filter(
      (file) => file.endsWith('.yaml') || file.endsWith('.yml')
    );

    for (const file of files) {
      try {
        const content = fs.readFileSync(file, 'utf8');
        const data = yaml.load(content);
        const scenario = ScenarioSchema.parse(data);
        scenarios.push(scenario);
      } catch (error) {
        console.error(`Error loading scenario from ${file}:`, error);
      }
    }

    return scenarios;
  }

  private recursiveReaddir(dir: string): string[] {
    const results: string[] = [];
    const list = fs.readdirSync(dir);

    for (const file of list) {
      const filePath = path.join(dir, file);
      const stat = fs.statSync(filePath);
      if (stat && stat.isDirectory()) {
        results.push(...this.recursiveReaddir(filePath));
      } else {
        results.push(filePath);
      }
    }

    return results;
  }
}
