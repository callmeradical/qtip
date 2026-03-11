import request from 'supertest';
import app from '../server';

describe('POST /api/v1/evaluate', () => {
  it('should execute and pass for a valid cli hello scenario', async () => {
    const manifest = {
      projectId: 'test-project',
      environment: 'local',
      interfaces: [
        {
          type: 'cli',
        },
      ],
      capabilities: ['test'],
    };

    const response = await request(app)
      .post('/api/v1/evaluate')
      .send(manifest);

    expect(response.status).toBe(200);
    expect(response.body.status).toBe('passed');
    expect(response.body.summary.total).toBe(1);
    expect(response.body.results[0].scenarioId).toBe('TEST-CLI-HELLO');
    expect(response.body.results[0].status).toBe('passed');
    expect(response.body.results[0].evidence.stdout).toContain('hello world');
  });

  it('should fail for unreachable api scenario', async () => {
    const manifest = {
      projectId: 'identity-service',
      environment: 'qa',
      interfaces: [
        {
          type: 'api',
          baseUrl: 'http://non-existent-url-12345.com',
        },
      ],
      capabilities: ['auth'],
    };

    const response = await request(app)
      .post('/api/v1/evaluate')
      .send(manifest);

    expect(response.status).toBe(200);
    expect(response.body.status).toBe('failed');
    expect(response.body.results[0].scenarioId).toBe('AUTH-LOGIN-001');
    expect(response.body.results[0].status).toBe('failed');
  });

  it('should pass for log check with no errors', async () => {
    const logPath = '/tmp/qtip-test.log';
    require('fs').writeFileSync(logPath, 'INFO: All good\nINFO: Login successful\n');

    const manifest = {
      projectId: 'test-project',
      environment: 'local',
      interfaces: [
        {
          type: 'logs',
        },
      ],
      capabilities: ['auth'],
      observability: {
        logs: {
          type: 'file',
          path: logPath,
        },
      },
    };

    const response = await request(app)
      .post('/api/v1/evaluate')
      .send(manifest);

    expect(response.status).toBe(200);
    expect(response.body.status).toBe('passed');
    expect(response.body.results.some((r: any) => r.scenarioId === 'LOG-ERROR-001')).toBe(true);
  });

  it('should fail for log check with errors', async () => {
    const logPath = '/tmp/qtip-test-error.log';
    require('fs').writeFileSync(logPath, 'INFO: All good\nERROR: Something went wrong\n');

    const manifest = {
      projectId: 'test-project',
      environment: 'local',
      interfaces: [
        {
          type: 'logs',
        },
      ],
      capabilities: ['auth'],
      observability: {
        logs: {
          type: 'file',
          path: logPath,
        },
      },
    };

    const response = await request(app)
      .post('/api/v1/evaluate')
      .send(manifest);

    expect(response.status).toBe(200);
    expect(response.body.status).toBe('failed');
    const logResult = response.body.results.find((r: any) => r.scenarioId === 'LOG-ERROR-001');
    expect(logResult.status).toBe('failed');
    expect(logResult.failures[0]).toContain('log event found but should not exist');
  });

  it('should return error for invalid subject manifest', async () => {
    const manifest = {
      projectId: 'identity-service',
      capabilities: ['auth'],
    };

    const response = await request(app)
      .post('/api/v1/evaluate')
      .send(manifest);

    expect(response.status).toBe(400);
    expect(response.body.status).toBe('error');
  });
});
