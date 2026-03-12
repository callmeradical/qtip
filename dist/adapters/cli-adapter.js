"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.CliAdapter = void 0;
const child_process_1 = require("child_process");
const util_1 = require("util");
const scenario_1 = require("../models/scenario");
const execAsync = (0, util_1.promisify)(child_process_1.exec);
class CliAdapter {
    async execute(_manifest, scenario) {
        const interaction = scenario_1.CliInteractionSchema.parse(scenario.interaction);
        try {
            const { stdout, stderr } = await execAsync(interaction.command, { timeout: 30000 });
            return {
                status: 0,
                stdout,
                stderr,
            };
        }
        catch (error) {
            return {
                status: error.code || 1,
                stdout: error.stdout || '',
                stderr: error.stderr || error.message,
            };
        }
    }
}
exports.CliAdapter = CliAdapter;
