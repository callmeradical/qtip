import fs from 'fs';
import path from 'path';
import { SubjectManifest, SubjectManifestSchema } from './models/subject-manifest';
import { ScenarioLoader } from './services/scenario-loader';
import { ScenarioResolver } from './resolvers/scenario-resolver';
import { ScenarioExecutor } from './services/scenario-executor';

export interface SubjectResult {
  projectId: string;
  results: any[];
  passedCount: number;
  failedCount: number;
}

export interface EvaluationOutcome {
  subjects: SubjectResult[];
  totalPassed: number;
  totalFailed: number;
  totalSubjects: number;
}

export async function run(manifestInputs: string[], scenariosPath: string): Promise<EvaluationOutcome> {
  const manifests: SubjectManifest[] = [];
  
  for (const input of manifestInputs) {
    // Check if input is a directory
    if (fs.existsSync(input) && fs.statSync(input).isDirectory()) {
      const files = fs.readdirSync(input).filter(f => f.endsWith('.json'));
      for (const file of files) {
        const fullPath = path.join(input, file);
        const data = JSON.parse(fs.readFileSync(fullPath, 'utf8'));
        manifests.push(SubjectManifestSchema.parse(data));
      }
      continue;
    }

    // Otherwise treat as file or JSON string
    let manifestData;
    if (fs.existsSync(input)) {
      manifestData = JSON.parse(fs.readFileSync(input, 'utf8'));
    } else {
      try {
        manifestData = JSON.parse(input);
      } catch (e) {
        throw new Error(`Invalid manifest JSON or file path: ${input}`);
      }
    }
    manifests.push(SubjectManifestSchema.parse(manifestData));
  }

  if (manifests.length === 0) {
    throw new Error('No manifests provided');
  }

  const loader = new ScenarioLoader(scenariosPath);
  const scenarios = await loader.loadAll();
  const executor = new ScenarioExecutor();
  
  const subjects: SubjectResult[] = [];
  let totalPassed = 0;
  let totalFailed = 0;

  for (const manifest of manifests) {
    console.log(`\n🔍 Evaluating Subject: ${manifest.projectId}`);
    
    const resolver = new ScenarioResolver(scenarios);
    const resolved = resolver.resolve(manifest);

    console.log(`🚀 qtip: Resolved ${resolved.length} scenarios for project ${manifest.projectId}\n`);

    const results = [];
    let passedCount = 0;
    let failedCount = 0;

    for (const scenario of resolved) {
      process.stdout.write(`  - Executing ${scenario.id}: ${scenario.name}... `);
      const result = await executor.execute(manifest, scenario);
      results.push(result);
      if (result.status === 'passed') {
        process.stdout.write('✅ PASSED\n');
        passedCount++;
        totalPassed++;
      } else {
        process.stdout.write('❌ FAILED\n');
        result.failures.forEach((f: string) => console.log(`      └─ ${f}`));
        failedCount++;
        totalFailed++;
      }
    }

    console.log(`\n📊 Subject Summary (${manifest.projectId}): ${passedCount} passed, ${failedCount} failed, ${resolved.length} total.`);
    
    subjects.push({
      projectId: manifest.projectId,
      results,
      passedCount,
      failedCount
    });

    // Generate GitHub Step Summary if running in GH Actions
    if (process.env.GITHUB_STEP_SUMMARY) {
      generateSummary(manifest.projectId, results);
    }
  }

  console.log(`\n🏁 Global Summary: ${totalPassed} passed, ${totalFailed} failed across ${manifests.length} subjects.`);

  return {
    subjects,
    totalPassed,
    totalFailed,
    totalSubjects: manifests.length
  };
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
  const args = process.argv.slice(2);
  let scenariosPath = path.join(process.cwd(), 'scenarios');
  const manifestInputs: string[] = [];

  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--scenarios' && i + 1 < args.length) {
      scenariosPath = args[i + 1];
      i++;
    } else {
      manifestInputs.push(args[i]);
    }
  }

  if (manifestInputs.length === 0) {
    console.error('Usage: qtip-cli <manifest-json-or-path> [manifest2 ...] [--scenarios <scenarios-dir>]');
    process.exit(1);
  }

  run(manifestInputs, scenariosPath)
    .then((outcome) => {
      if (outcome.totalFailed > 0) {
        process.exit(1);
      }
    })
    .catch((error) => {
      console.error(`❌ ${error.message}`);
      process.exit(1);
    });
}
