import fs from 'fs';
import path from 'path';
import { SubjectManifest, SubjectManifestSchema } from './models/subject-manifest';
import { ScenarioLoader } from './services/scenario-loader';
import { ScenarioResolver } from './resolvers/scenario-resolver';
import { ScenarioExecutor } from './services/scenario-executor';

export async function run(manifestPaths: string[], scenariosPath: string) {
  const manifests: SubjectManifest[] = [];
  
  for (const manifestPath of manifestPaths) {
    let manifestData;
    if (fs.existsSync(manifestPath)) {
      manifestData = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
    } else {
      try {
        manifestData = JSON.parse(manifestPath);
      } catch (e) {
        throw new Error(`Invalid manifest JSON or file path: ${manifestPath}`);
      }
    }
    manifests.push(SubjectManifestSchema.parse(manifestData));
  }

  if (manifests.length === 0) {
    throw new Error('No manifests provided');
  }

  // Merge manifests (simple version for now: use first one as base)
  const primaryManifest = manifests[0];
  const mergedManifest: SubjectManifest = {
    ...primaryManifest,
    interfaces: manifests.flatMap(m => m.interfaces),
    capabilities: Array.from(new Set(manifests.flatMap(m => m.capabilities))),
  };

  try {
    const loader = new ScenarioLoader(scenariosPath);
    const scenarios = await loader.loadAll();
    const resolver = new ScenarioResolver(scenarios);
    const resolved = resolver.resolve(mergedManifest);

    console.log(`🚀 qtip: Resolved ${resolved.length} scenarios for project ${mergedManifest.projectId}\n`);

    const executor = new ScenarioExecutor();
    const results = [];
    for (const scenario of resolved) {
      process.stdout.write(`  - Executing ${scenario.id}: ${scenario.name}... `);
      const result = await executor.execute(mergedManifest, scenario);
      results.push(result);
      if (result.status === 'passed') {
        process.stdout.write('✅ PASSED\n');
      } else {
        process.stdout.write('❌ FAILED\n');
        result.failures.forEach((f: string) => console.log(`      └─ ${f}`));
      }
    }

    const passedCount = results.filter((r) => r.status === 'passed').length;
    const failedCount = results.length - passedCount;

    console.log(`\n📊 Summary: ${passedCount} passed, ${failedCount} failed, ${results.length} total.`);

    // Generate GitHub Step Summary if running in GH Actions
    if (process.env.GITHUB_STEP_SUMMARY) {
      generateSummary(mergedManifest.projectId, results);
    }

    return {
      results,
      passedCount,
      failedCount
    };
  } catch (error: any) {
    throw new Error(`Evaluation failed: ${error.message}`);
  }
}

function generateSummary(projectId: string, results: any[]) {
  const summaryPath = process.env.GITHUB_STEP_SUMMARY!;
  let markdown = `### 📋 qtip Evaluation Results: ${projectId}\n\n`;
  markdown += `| Scenario | Status | Failures |\n`;
  markdown += `| :--- | :---: | :--- |\n`;

  results.forEach((r) => {
    const statusIcon = r.status === 'passed' ? '✅' : '❌';
    const failures = r.failures.join('<br>') || '-';
    markdown += `| ${r.scenarioId} | ${statusIcon} | ${failures} |\n`;
  });

  fs.appendFileSync(summaryPath, markdown);
}

if (require.main === module) {
  const manifestPath = process.argv[2];
  const scenariosPath = process.argv[3] || path.join(process.cwd(), 'scenarios');

  if (!manifestPath) {
    console.error('Usage: qtip-cli <manifest-json-or-path> [scenarios-dir]');
    process.exit(1);
  }

  // Support comma-separated manifest paths for now in CLI args
  const manifestPaths = manifestPath.split(',');

  run(manifestPaths, scenariosPath)
    .then((outcome) => {
      if (outcome.failedCount > 0) {
        process.exit(1);
      }
    })
    .catch((error) => {
      console.error(`❌ ${error.message}`);
      process.exit(1);
    });
}
