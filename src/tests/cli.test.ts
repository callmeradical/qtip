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

  it('should resolve scenarios across different services with shared capability', async () => {
    const authManifest = JSON.stringify({
      projectId: 'auth-service',
      environment: 'prod',
      interfaces: [{ type: 'api', name: 'auth-service', baseUrl: 'http://auth.svc' }],
      capabilities: ['multi-service']
    });

    const userManifest = JSON.stringify({
      projectId: 'user-service',
      environment: 'prod',
      interfaces: [{ type: 'api', name: 'user-service', baseUrl: 'http://user.svc' }],
      capabilities: ['multi-service']
    });

    const outcome = await run([authManifest, userManifest], scenariosPath);

    const resolvedIds = outcome.results.map(r => r.scenarioId);
    expect(resolvedIds).toContain('CROSS-SERVICE-001');
    expect(resolvedIds).toContain('CROSS-SERVICE-002');
    
    // Get the mock instance that was created inside 'run'
    const mockExecutorInstance = (ScenarioExecutor as jest.Mock).mock.results[0].value;
    expect(mockExecutorInstance.execute).toHaveBeenCalledWith(
      expect.objectContaining({
        interfaces: expect.arrayContaining([
          expect.objectContaining({ name: 'auth-service' }),
          expect.objectContaining({ name: 'user-service' })
        ])
      }),
      expect.anything()
    );
  });
});
