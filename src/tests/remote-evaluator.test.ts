import axios from 'axios';
import { RemoteEvaluator } from '../services/remote-evaluator';
import { SubjectManifest } from '../models/subject-manifest';

jest.mock('axios');
const mockedAxios = axios as jest.MockedFunction<typeof axios>;

describe('RemoteEvaluator', () => {
  let evaluator: RemoteEvaluator;

  beforeEach(() => {
    evaluator = new RemoteEvaluator();
    jest.clearAllMocks();
  });

  const baseManifest: SubjectManifest = {
    projectId: 'test-project',
    environment: 'test',
    capabilities: ['auth'],
    interfaces: [{ type: 'cli' }],
  };

  const remoteUrl = 'http://remote-qtip.svc/api/v1';

  it('should send the manifest to the remote server', async () => {
    mockedAxios.mockResolvedValueOnce({
      status: 200,
      data: {
        status: 'passed',
        summary: { total: 1, passed: 1, failed: 0 },
        results: [{ scenarioId: 'TEST-1', status: 'passed', failures: [] }],
      },
    });

    const result = await evaluator.evaluate(baseManifest, remoteUrl);

    expect(mockedAxios).toHaveBeenCalledWith(expect.objectContaining({
      method: 'POST',
      url: 'http://remote-qtip.svc/api/v1/evaluate',
      data: baseManifest,
    }));

    expect(result.projectId).toBe('test-project');
    expect(result.passedCount).toBe(1);
    expect(result.failedCount).toBe(0);
    expect(result.results.length).toBe(1);
  });

  it('should throw an error if the server returns an error status', async () => {
    mockedAxios.mockResolvedValueOnce({
      status: 400,
      data: { message: 'Invalid manifest' },
    });

    await expect(evaluator.evaluate(baseManifest, remoteUrl))
      .rejects.toThrow('Remote evaluation failed: 400 - Invalid manifest');
  });

  it('should throw an error if the request fails', async () => {
    mockedAxios.mockRejectedValueOnce(new Error('Network error'));

    await expect(evaluator.evaluate(baseManifest, remoteUrl))
      .rejects.toThrow('Remote evaluation request failed: Network error');
  });
});
