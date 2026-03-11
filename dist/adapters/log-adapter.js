"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.LogAdapter = void 0;
const fs_1 = __importDefault(require("fs"));
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
        const content = fs_1.default.readFileSync(logSource.path, 'utf8');
        const lines = content.split('\n');
        const matchedLines = lines.filter((line) => line.includes(interaction.query));
        return {
            found: matchedLines.length > 0,
            content: matchedLines.join('\n'),
        };
    }
}
exports.LogAdapter = LogAdapter;
