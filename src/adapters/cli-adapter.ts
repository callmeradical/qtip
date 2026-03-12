import { exec } from 'child_process';
import { promisify } from 'util';
import { SubjectManifest } from '../models/subject-manifest';
import { Scenario, CliInteractionSchema } from '../models/scenario';

const execAsync = promisify(exec);

export interface CliEvidence {
  status: number;
  stdout: string;
  stderr: string;
}

export class CliAdapter {
  async execute(_manifest: SubjectManifest, scenario: Scenario): Promise<CliEvidence> {
    const interaction = CliInteractionSchema.parse(scenario.interaction);

    try {
      const { stdout, stderr } = await execAsync(interaction.command, { timeout: 30000 });
      return {
        status: 0,
        stdout,
        stderr,
      };
    } catch (error: any) {
      return {
        status: error.code || 1,
        stdout: error.stdout || '',
        stderr: error.stderr || error.message,
      };
    }
  }
}
