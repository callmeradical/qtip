"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.ScenarioLoader = void 0;
const fs_1 = __importDefault(require("fs"));
const path_1 = __importDefault(require("path"));
const js_yaml_1 = __importDefault(require("js-yaml"));
const axios_1 = __importDefault(require("axios"));
const scenario_1 = require("../models/scenario");
class ScenarioLoader {
    constructor(scenariosSource) {
        this.scenariosSource = scenariosSource;
    }
    async loadAll() {
        if (this.scenariosSource.startsWith('http')) {
            return this.loadFromUrl(this.scenariosSource);
        }
        return this.loadFromLocal(this.scenariosSource);
    }
    async loadFromUrl(url) {
        try {
            const response = await axios_1.default.get(url);
            const data = typeof response.data === 'string' ? js_yaml_1.default.load(response.data) : response.data;
            // If the URL returns an array of scenarios
            if (Array.isArray(data)) {
                return data.map(s => scenario_1.ScenarioSchema.parse(s));
            }
            // If it's a single scenario
            return [scenario_1.ScenarioSchema.parse(data)];
        }
        catch (error) {
            console.error(`Error loading scenarios from URL ${url}:`, error.message);
            return [];
        }
    }
    loadFromLocal(dir) {
        const scenarios = [];
        if (!fs_1.default.existsSync(dir)) {
            return [];
        }
        const files = this.recursiveReaddir(dir).filter((file) => file.endsWith('.yaml') || file.endsWith('.yml'));
        for (const file of files) {
            try {
                const content = fs_1.default.readFileSync(file, 'utf8');
                const data = js_yaml_1.default.load(content);
                const scenario = scenario_1.ScenarioSchema.parse(data);
                scenarios.push(scenario);
            }
            catch (error) {
                console.error(`Error loading scenario from ${file}:`, error);
            }
        }
        return scenarios;
    }
    recursiveReaddir(dir) {
        const results = [];
        const list = fs_1.default.readdirSync(dir);
        for (const file of list) {
            const filePath = path_1.default.join(dir, file);
            const stat = fs_1.default.statSync(filePath);
            if (stat && stat.isDirectory()) {
                results.push(...this.recursiveReaddir(filePath));
            }
            else {
                results.push(filePath);
            }
        }
        return results;
    }
}
exports.ScenarioLoader = ScenarioLoader;
