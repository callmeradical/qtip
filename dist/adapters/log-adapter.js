"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.LogAdapter = void 0;
const fs_1 = __importDefault(require("fs"));
const readline_1 = __importDefault(require("readline"));
const scenario_1 = require("../models/scenario");
class LogAdapter {
    async execute(manifest, scenario) {
        const interaction = scenario_1.LogInteractionSchema.parse(scenario.interaction);
        const logSource = manifest.observability?.logs;
        if (!logSource || logSource.type !== 'file' || !logSource.path) {
            throw new Error(`Log source (file) not found in manifest for project ${manifest.projectId}`);
        }
        if (!fs_1.default.existsSync(logSource.path)) {
            return {
                found: false,
                content: '',
            };
        }
        const matchedLines = [];
        const fileStream = fs_1.default.createReadStream(logSource.path);
        const rl = readline_1.default.createInterface({
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
exports.LogAdapter = LogAdapter;
