import path from 'path';
import { Router } from 'express';
import { SubjectManifestSchema } from '../models/subject-manifest';
import { ScenarioResolver } from '../resolvers/scenario-resolver';
import { ScenarioLoader } from '../services/scenario-loader';
import { ScenarioExecutor } from '../services/scenario-executor';

const router = Router();

const scenariosDir = path.join(process.cwd(), 'scenarios');
const scenarioLoader = new ScenarioLoader(scenariosDir);
const scenarioExecutor = new ScenarioExecutor();

router.post('/evaluate', async (req, res) => {
  try {
    const manifest = SubjectManifestSchema.parse(req.body);
    const scenarios = await scenarioLoader.loadAll();
    const scenarioResolver = new ScenarioResolver(scenarios);
    const resolvedScenarios = scenarioResolver.resolve(manifest);

    const results = await Promise.all(
      resolvedScenarios.map((scenario) => scenarioExecutor.execute(manifest, scenario))
    );

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
  } catch (error) {
    if (error instanceof Error) {
      res.status(400).json({ status: 'error', message: error.message });
    } else {
      res.status(500).json({ status: 'error', message: 'Internal Server Error' });
    }
  }
});

export default router;
