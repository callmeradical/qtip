"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.ScenarioExecutor = void 0;
const jsonpath_plus_1 = require("jsonpath-plus");
const scenario_1 = require("../models/scenario");
const subject_manifest_1 = require("../models/subject-manifest");
const api_adapter_1 = require("../adapters/api-adapter");
const cli_adapter_1 = require("../adapters/cli-adapter");
const log_adapter_1 = require("../adapters/log-adapter");
class ScenarioExecutor {
    apiAdapter = new api_adapter_1.ApiAdapter();
    cliAdapter = new cli_adapter_1.CliAdapter();
    logAdapter = new log_adapter_1.LogAdapter();
    async execute(manifest, scenario) {
        try {
            let evidence;
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
                default:
                    throw new Error(`Unsupported interaction type: ${scenario.interaction.type}`);
            }
            const failures = this.evaluateChecks(scenario, evidence);
            return {
                scenarioId: scenario.id,
                status: failures.length === 0 ? 'passed' : 'failed',
                evidence,
                failures,
            };
        }
        catch (error) {
            return {
                scenarioId: scenario.id,
                status: 'error',
                evidence: { message: error.message },
                failures: [error.message],
            };
        }
    }
    evaluateChecks(scenario, evidence) {
        const failures = [];
        for (const check of scenario.checks) {
            switch (check.type) {
                case 'status_code':
                    if (evidence.status !== check.expected) {
                        failures.push(`Check ${check.type} failed: expected ${check.expected}, got ${evidence.status} for AC ${check.acceptance_criteria}`);
                    }
                    break;
                case 'json_path':
                    if (check.path && evidence.data) {
                        const results = (0, jsonpath_plus_1.JSONPath)({ path: check.path, json: evidence.data });
                        if (check.exists && results.length === 0) {
                            failures.push(`Check ${check.type} failed: path ${check.path} not found for AC ${check.acceptance_criteria}`);
                        }
                        else if (check.expected !== undefined && results[0] !== check.expected) {
                            failures.push(`Check ${check.type} failed: expected ${check.expected} at ${check.path}, got ${results[0]} for AC ${check.acceptance_criteria}`);
                        }
                    }
                    break;
                case 'stdout':
                    if (evidence.stdout !== undefined && check.expected && !evidence.stdout.includes(check.expected)) {
                        failures.push(`Check ${check.type} failed: stdout does not contain ${check.expected} for AC ${check.acceptance_criteria}`);
                    }
                    break;
                case 'log_contains':
                    if (evidence.found === false) {
                        failures.push(`Check ${check.type} failed: log event not found for AC ${check.acceptance_criteria}`);
                    }
                    break;
                case 'log_not_contains':
                    if (evidence.found === true) {
                        failures.push(`Check ${check.type} failed: log event found but should not exist for AC ${check.acceptance_criteria}`);
                    }
                    break;
            }
        }
        return failures;
    }
}
exports.ScenarioExecutor = ScenarioExecutor;
//# sourceMappingURL=scenario-executor.js.map