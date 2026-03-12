import { run } from '../cli';
import { RemoteEvaluator } from '../services/remote-evaluator';
import path from 'path';

jest.mock('../services/remote-evaluator');

describe('CLI Remote Evaluation', () => {
  const scenariosPath = path.join(process.cwd(), 'scenarios');
  const remoteUrl = 'http://remote-qtip.svc';

  beforeEach(() => {
    jest.clearAllMocks();
    (RemoteEvaluator as jest.Mock).mockImplementation(() => ({
      evaluate: jest.fn().mockImplementation((manifest) => Promise.resolve({
        projectId: manifest.projectId,
        results: [],
        passedCount: 0,
        failedCount: 0
      }))
    }));
  });

  it('should delegate to RemoteEvaluator when remoteUrl is provided', async () => {
    const manifest1 = JSON.stringify({
      projectId: 'service-a',
      environment: 'test',
      interfaces: [{ type: 'cli' }],
      capabilities: ['test']
    });

    const outcome = await run([manifest1], scenariosPath, remoteUrl);

    const mockEvaluatorInstance = (RemoteEvaluator as jest.Mock).mock.results[0].value;
    expect(mockEvaluatorInstance.evaluate).toHaveBeenCalledWith(expect.anything(), remoteUrl);
    expect(outcome.subjects.length).toBe(1);
    expect(outcome.subjects[0].projectId).toBe('service-a');
  });

  it('should call RemoteEvaluator for each manifest', async () => {
    const manifest1 = JSON.stringify({
        projectId: 'service-a',
        environment: 'test',
        interfaces: [{ type: 'cli' }],
        capabilities: ['test']
    });
    const manifest2 = JSON.stringify({
        projectId: 'service-b',
        environment: 'test',
        interfaces: [{ type: 'cli' }],
        capabilities: ['test']
    });

    await run([manifest1, manifest2], scenariosPath, remoteUrl);

    const mockEvaluatorInstance = (RemoteEvaluator as jest.Mock).mock.results[0].value;
    expect(mockEvaluatorInstance.evaluate).toHaveBeenCalledTimes(2);
  });
});
