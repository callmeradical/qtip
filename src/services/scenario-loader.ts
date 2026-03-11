import fs from 'fs';
import path from 'path';
import yaml from 'js-yaml';
import axios from 'axios';
import { Scenario, ScenarioSchema } from '../models/scenario';

export class ScenarioLoader {
  private scenariosSource: string;

  constructor(scenariosSource: string) {
    this.scenariosSource = scenariosSource;
  }

  async loadAll(): Promise<Scenario[]> {
    if (this.scenariosSource.startsWith('http')) {
      return this.loadFromUrl(this.scenariosSource);
    }
    return this.loadFromLocal(this.scenariosSource);
  }

  private async loadFromUrl(url: string): Promise<Scenario[]> {
    try {
      const response = await axios.get(url);
      const data = typeof response.data === 'string' ? yaml.load(response.data) : response.data;

      // If the URL returns an array of scenarios
      if (Array.isArray(data)) {
        return data.map(s => ScenarioSchema.parse(s));
      }
      // If it's a single scenario
      return [ScenarioSchema.parse(data)];
    } catch (error: any) {
      console.error(`Error loading scenarios from URL ${url}:`, error.message);
      return [];
    }
  }

  private loadFromLocal(dir: string): Scenario[] {
    const scenarios: Scenario[] = [];
    if (!fs.existsSync(dir)) {
      return [];
    }

    const files = this.recursiveReaddir(dir).filter(
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
