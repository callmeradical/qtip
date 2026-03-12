import fs from 'fs';
import readline from 'readline';
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

    const matchedLines: string[] = [];
    const fileStream = fs.createReadStream(logSource.path);
    const rl = readline.createInterface({
      input: fileStream,
      crlfDelay: Infinity,
    });

    for await (const line of rl) {
      if (line.includes(interaction.query)) {
        matchedLines.push(line);
      }
    }

    return {
      found: matchedLines.length > 0,
      content: matchedLines.join('\n'),
    };
  }
}
