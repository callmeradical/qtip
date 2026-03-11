"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const path_1 = __importDefault(require("path"));
const express_1 = require("express");
const subject_manifest_1 = require("../models/subject-manifest");
const scenario_resolver_1 = require("../resolvers/scenario-resolver");
const scenario_loader_1 = require("../services/scenario-loader");
const scenario_executor_1 = require("../services/scenario-executor");
const router = (0, express_1.Router)();
const scenariosDir = path_1.default.join(process.cwd(), 'scenarios');
const scenarioLoader = new scenario_loader_1.ScenarioLoader(scenariosDir);
const scenarioExecutor = new scenario_executor_1.ScenarioExecutor();
router.post('/evaluate', async (req, res) => {
    try {
        const manifest = subject_manifest_1.SubjectManifestSchema.parse(req.body);
        const scenarios = await scenarioLoader.loadAll();
        const scenarioResolver = new scenario_resolver_1.ScenarioResolver(scenarios);
        const resolvedScenarios = scenarioResolver.resolve(manifest);
        const results = await Promise.all(resolvedScenarios.map((scenario) => scenarioExecutor.execute(manifest, scenario)));
        const passedCount = results.filter((r) => r.status === 'passed').length;
        const failedCount = results.filter((r) => r.status !== 'passed').length;
        const failures = results
            .filter((r) => r.status !== 'passed')
            .map((r) => ({
            scenarioId: r.scenarioId,
            acceptanceCriteria: r.failures, // Mapping failures to AC ids for now
        }));
        res.json({
            status: failedCount > 0 ? 'failed' : 'passed',
            summary: {
                total: results.length,
                passed: passedCount,
                failed: failedCount,
            },
            failures: failedCount > 0 ? failures : undefined,
            results, // Keeping full results for detail
        });
    }
    catch (error) {
        if (error instanceof Error) {
            res.status(400).json({ status: 'error', message: error.message });
        }
        else {
            res.status(500).json({ status: 'error', message: 'Internal Server Error' });
        }
    }
});
exports.default = router;
//# sourceMappingURL=evaluation.js.map