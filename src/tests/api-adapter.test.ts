import axios from 'axios';
import { ApiAdapter } from '../adapters/api-adapter';
import { SubjectManifest } from '../models/subject-manifest';
import { Scenario } from '../models/scenario';

jest.mock('axios');
const mockedAxios = axios as jest.MockedFunction<typeof axios>;

describe('ApiAdapter', () => {
  let adapter: ApiAdapter;

  beforeEach(() => {
    adapter = new ApiAdapter();
    jest.clearAllMocks();
  });

  const baseManifest: SubjectManifest = {
    projectId: 'test-project',
    environment: 'test',
    capabilities: ['auth'],
    interfaces: [
      { type: 'api', name: 'auth-service', baseUrl: 'http://auth.svc' },
      { type: 'api', name: 'user-service', baseUrl: 'http://user.svc' },
    ],
  };

  const createScenario = (service?: string): Scenario => ({
    id: 'TEST-001',
    name: 'Test Scenario',
    applies_to: {
      capabilities: ['auth'],
      interfaces: ['api'],
    },
    acceptance_criteria: [{ id: 'AC-1', description: 'Success' }],
    interaction: {
      type: 'api',
      service,
      request: {
        method: 'GET',
        path: '/health',
        headers: {} // Ensure headers is present
      },
    },
    checks: [
      { type: 'status_code', expected: 200, acceptance_criteria: 'AC-1' },
    ],
  });

  it('should target the correct service by name', async () => {
    mockedAxios.mockResolvedValueOnce({
      status: 200,
      data: { status: 'ok' },
      headers: {},
    });

    const scenario = createScenario('user-service');
    const result = await adapter.execute(baseManifest, scenario);

    expect(mockedAxios).toHaveBeenCalledWith(expect.objectContaining({
      url: 'http://user.svc/health',
    }));
    expect(result.status).toBe(200);
  });

  it('should target the first api service if no service name provided', async () => {
    mockedAxios.mockResolvedValueOnce({
      status: 200,
      data: { status: 'ok' },
      headers: {},
    });

    const scenario = createScenario(undefined);
    const result = await adapter.execute(baseManifest, scenario);

    expect(mockedAxios).toHaveBeenCalledWith(expect.objectContaining({
      url: 'http://auth.svc/health',
    }));
    expect(result.status).toBe(200);
  });

  it('should throw error if named service is not found', async () => {
    const scenario = createScenario('missing-service');
    
    await expect(adapter.execute(baseManifest, scenario))
      .rejects.toThrow("service 'missing-service' not found in manifest");
  });

  it('should throw error if no api service exists', async () => {
    const manifestNoApi: SubjectManifest = {
        ...baseManifest,
        interfaces: [{ type: 'cli' }]
    };
    const scenario = createScenario(undefined);
    
    await expect(adapter.execute(manifestNoApi, scenario))
      .rejects.toThrow("API interface not found in manifest");
  });
});
