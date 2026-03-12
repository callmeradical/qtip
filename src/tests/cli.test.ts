import { run } from '../cli';
import { ScenarioExecutor } from '../services/scenario-executor';
import path from 'path';

jest.mock('../services/scenario-executor');

describe('CLI Multi-Manifest Loading', () => {
  const scenariosPath = path.join(process.cwd(), 'scenarios');

  beforeEach(() => {
    jest.clearAllMocks();
    (ScenarioExecutor as jest.Mock).mockImplementation(() => ({
      execute: jest.fn().mockImplementation((manifest, scenario) => Promise.resolve({ 
        status: 'passed', 
        scenarioId: scenario.id, 
        failures: [] 
      }))
    }));
  });

  it('should load and merge two manifests from JSON strings', async () => {
    const manifest1 = JSON.stringify({
      projectId: 'service-a',
      environment: 'test',
      interfaces: [{ type: 'cli' }],
      capabilities: ['test']
    });

    const manifest2 = JSON.stringify({
      projectId: 'service-b',
      environment: 'test',
      interfaces: [{ type: 'api', baseUrl: 'http://service-b.svc' }],
      capabilities: ['auth']
    });

    const outcome = await run([manifest1, manifest2], scenariosPath);

    expect(outcome.results.length).toBeGreaterThan(1);
    
    // Should have resolved scenarios for BOTH 'test' (cli) and 'auth' (api) capabilities
    const resolvedIds = outcome.results.map(r => r.scenarioId);
    expect(resolvedIds).toContain('TEST-CLI-HELLO');
    expect(resolvedIds).toContain('AUTH-LOGIN-001');
  });

  it('should throw error if one of the manifests is invalid', async () => {
    const manifest1 = JSON.stringify({
      projectId: 'service-a',
      environment: 'test',
      interfaces: [{ type: 'cli' }],
      capabilities: ['test']
    });

    const manifest2 = 'invalid-json';

    await expect(run([manifest1, manifest2], scenariosPath))
      .rejects.toThrow('Invalid manifest JSON or file path');
  });
});
