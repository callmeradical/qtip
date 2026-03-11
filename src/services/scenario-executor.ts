import { JSONPath } from 'jsonpath-plus';
import { Scenario } from '../models/scenario';
import { SubjectManifest } from '../models/subject-manifest';
import { ApiAdapter } from '../adapters/api-adapter';
import { CliAdapter } from '../adapters/cli-adapter';
import { LogAdapter } from '../adapters/log-adapter';

export interface EvaluationResult {
  scenarioId: string;
  status: 'passed' | 'failed' | 'error';
  evidence: any;
  failures: string[];
}

export class ScenarioExecutor {
  private apiAdapter = new ApiAdapter();
  private cliAdapter = new CliAdapter();
  private logAdapter = new LogAdapter();

  async execute(manifest: SubjectManifest, scenario: Scenario): Promise<EvaluationResult> {
    try {
      let evidence: any;

      switch (scenario.interaction.type) {
        case 'api':
          evidence = await this.apiAdapter.execute(manifest, scenario);
          break;
        case 'cli':
          evidence = await this.cliAdapter.execute(manifest, scenario);
          break;
        case 'logs':
          evidence = await this.logAdapter.execute(manifest, scenario);
          break;
        case 'ui':
          throw new Error('UI Adapter not implemented in MVP');
        default:
          throw new Error(`Unsupported interaction type`);
      }

      const failures = this.evaluateChecks(scenario, evidence);

      return {
        scenarioId: scenario.id,
        status: failures.length === 0 ? 'passed' : 'failed',
        evidence,
        failures,
      };
    } catch (error: any) {
      return {
        scenarioId: scenario.id,
        status: 'error',
        evidence: { message: error.message },
        failures: [error.message],
      };
    }
  }

  private evaluateChecks(scenario: Scenario, evidence: any): string[] {
    const failures: string[] = [];

    for (const check of scenario.checks) {
      switch (check.type) {
        case 'status_code':
          if (evidence.status !== check.expected) {
            failures.push(
              `Check ${check.type} failed: expected ${check.expected}, got ${evidence.status} for AC ${check.acceptance_criteria}`
            );
          }
          break;
        case 'json_path':
          if (check.path && evidence.data) {
            const results = JSONPath({ path: check.path, json: evidence.data });
            if (check.exists && results.length === 0) {
              failures.push(
                `Check ${check.type} failed: path ${check.path} not found for AC ${check.acceptance_criteria}`
              );
            } else if (check.expected !== undefined && results[0] !== check.expected) {
              failures.push(
                `Check ${check.type} failed: expected ${check.expected} at ${check.path}, got ${results[0]} for AC ${check.acceptance_criteria}`
              );
            }
          }
          break;
        case 'stdout':
          if (evidence.stdout !== undefined && check.expected && !evidence.stdout.includes(check.expected)) {
            failures.push(
              `Check ${check.type} failed: stdout does not contain ${check.expected} for AC ${check.acceptance_criteria}`
            );
          }
          break;
        case 'log_contains':
          if (evidence.found === false) {
            failures.push(
              `Check ${check.type} failed: log event not found for AC ${check.acceptance_criteria}`
            );
          }
          break;
        case 'log_not_contains':
          if (evidence.found === true) {
            failures.push(
              `Check ${check.type} failed: log event found but should not exist for AC ${check.acceptance_criteria}`
            );
          }
          break;
      }
    }

    return failures;
  }
}
