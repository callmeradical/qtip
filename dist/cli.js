"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const fs_1 = __importDefault(require("fs"));
const path_1 = __importDefault(require("path"));
const subject_manifest_1 = require("./models/subject-manifest");
const scenario_loader_1 = require("./services/scenario-loader");
const scenario_resolver_1 = require("./resolvers/scenario-resolver");
const scenario_executor_1 = require("./services/scenario-executor");
async function main() {
    const manifestPath = process.argv[2];
    const scenariosPath = process.argv[3] || path_1.default.join(process.cwd(), 'scenarios');
    if (!manifestPath) {
        console.error('Usage: qtip-cli <manifest-json-or-path> [scenarios-dir]');
        process.exit(1);
    }
    let manifestData;
    if (fs_1.default.existsSync(manifestPath)) {
        manifestData = JSON.parse(fs_1.default.readFileSync(manifestPath, 'utf8'));
    }
    else {
        try {
            manifestData = JSON.parse(manifestPath);
        }
        catch (e) {
            console.error('Error: Invalid manifest JSON or file path.');
            process.exit(1);
        }
    }
    try {
        const manifest = subject_manifest_1.SubjectManifestSchema.parse(manifestData);
        const loader = new scenario_loader_1.ScenarioLoader(scenariosPath);
        const scenarios = await loader.loadAll();
        const resolver = new scenario_resolver_1.ScenarioResolver(scenarios);
        const resolved = resolver.resolve(manifest);
        console.log(`🚀 qtip: Resolved ${resolved.length} scenarios for project ${manifest.projectId}\n`);
        const executor = new scenario_executor_1.ScenarioExecutor();
        const results = [];
        for (const scenario of resolved) {
            process.stdout.write(`  - Executing ${scenario.id}: ${scenario.name}... `);
            const result = await executor.execute(manifest, scenario);
            results.push(result);
            if (result.status === 'passed') {
                process.stdout.write('✅ PASSED\n');
            }
            else {
                process.stdout.write('❌ FAILED\n');
                result.failures.forEach((f) => console.log(`      └─ ${f}`));
            }
        }
        const passedCount = results.filter((r) => r.status === 'passed').length;
        const failedCount = results.length - passedCount;
        console.log(`\n📊 Summary: ${passedCount} passed, ${failedCount} failed, ${results.length} total.`);
        // Generate GitHub Step Summary if running in GH Actions
        if (process.env.GITHUB_STEP_SUMMARY) {
            generateSummary(manifest.projectId, results);
        }
        if (failedCount > 0) {
            process.exit(1);
        }
    }
    catch (error) {
        console.error('❌ Evaluation failed:', error.message);
        process.exit(1);
    }
}
function generateSummary(projectId, results) {
    const summaryPath = process.env.GITHUB_STEP_SUMMARY;
    let markdown = `### 📋 qtip Evaluation Results: ${projectId}\n\n`;
    markdown += `| Scenario | Status | Failures |\n`;
    markdown += `| :--- | :---: | :--- |\n`;
    results.forEach((r) => {
        const statusIcon = r.status === 'passed' ? '✅' : '❌';
        const failures = r.failures.join('<br>') || '-';
        markdown += `| ${r.scenarioId} | ${statusIcon} | ${failures} |\n`;
    });
    fs_1.default.appendFileSync(summaryPath, markdown);
}
main();
