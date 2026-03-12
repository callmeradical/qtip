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

  it('should load each manifest independently', async () => {
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

    expect(outcome.subjects.length).toBe(2);
    expect(outcome.totalSubjects).toBe(2);
    
    const projectIds = outcome.subjects.map(s => s.projectId);
    expect(projectIds).toContain('service-a');
    expect(projectIds).toContain('service-b');
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

  it('should execute scenarios for each service', async () => {
    const authManifest = JSON.stringify({
      projectId: 'auth-service',
      environment: 'prod',
      interfaces: [{ type: 'api', name: 'auth-service', baseUrl: 'http://auth.svc' }],
      capabilities: ['auth']
    });

    const userManifest = JSON.stringify({
      projectId: 'user-service',
      environment: 'prod',
      interfaces: [{ type: 'api', name: 'user-service', baseUrl: 'http://user.svc' }],
      capabilities: ['auth']
    });

    const outcome = await run([authManifest, userManifest], scenariosPath);

    expect(outcome.subjects.length).toBe(2);
    
    // Each service should have resolved its own scenarios
    const authResults = outcome.subjects.find(s => s.projectId === 'auth-service')?.results;
    expect(authResults?.length).toBeGreaterThan(0);
  });
});
