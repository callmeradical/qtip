import fs from 'fs';
import { SubjectManifest } from '../models/subject-manifest';
import { Scenario, LogInteractionSchema } from '../models/scenario';

export interface LogEvidence {
  found: boolean;
  content: string;
}

export class LogAdapter {
  async execute(manifest: SubjectManifest, scenario: Scenario): Promise<LogEvidence> {
    const interaction = LogInteractionSchema.parse(scenario.interaction);
    const logSource = manifest.observability?.logs;

    if (!logSource || logSource.type !== 'file' || !logSource.path) {
      throw new Error(`Log source (file) not found in manifest for project ${manifest.projectId}`);
    }

    if (!fs.existsSync(logSource.path)) {
      return {
        found: false,
        content: '',
      };
    }

    const content = fs.readFileSync(logSource.path, 'utf8');
    const lines = content.split('\n');
    const matchedLines = lines.filter((line) => line.includes(interaction.query));

    return {
      found: matchedLines.length > 0,
      content: matchedLines.join('\n'),
    };
  }
}
