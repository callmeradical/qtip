import { run } from '../cli';
import { ScenarioExecutor } from '../services/scenario-executor';
import path from 'path';

jest.mock('../services/scenario-executor');

describe('CLI Multi-Manifest Independent Runs', () => {
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

  it('should run each manifest independently', async () => {
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

    // Get the mock instance that was created inside 'run'
    // Actually, currently 'run' creates ONE ScenarioExecutor for the merged manifest.
    // We want it to ideally handle multiple.
    
    // In our desired implementation, executor.execute should be called with:
    // 1. manifest1 and its scenarios
    // 2. manifest2 and its scenarios
    
    // For now, let's just check how many times execute was called and with what.
    const mockExecutorInstance = (ScenarioExecutor as jest.Mock).mock.results[0].value;
    
    // Verify that some calls were with service-a and some with service-b
    const callManifests = mockExecutorInstance.execute.mock.calls.map((call: any) => call[0].projectId);
    
    expect(callManifests).toContain('service-a');
    expect(callManifests).toContain('service-b');
    
    // And verify they were NOT merged (i.e. service-a call doesn't have service-b's interfaces)
    const serviceACalls = mockExecutorInstance.execute.mock.calls.filter((call: any) => call[0].projectId === 'service-a');
    serviceACalls.forEach((call: any) => {
        expect(call[0].interfaces.length).toBe(1);
        expect(call[0].interfaces[0].type).toBe('cli');
    });
  });
});
