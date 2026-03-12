import request from 'supertest';
import app from '../server';
import { run } from '../cli';
import path from 'path';

describe('CLI Remote E2E', () => {
  let server: any;
  const port = 3333;
  const remoteUrl = `http://localhost:${port}/api/v1`;

  beforeAll((done) => {
    server = app.listen(port, done);
  });

  afterAll((done) => {
    server.close(done);
  });

  it('should successfully evaluate a manifest via remote server', async () => {
    const manifest = JSON.stringify({
      projectId: 'e2e-test',
      environment: 'test',
      interfaces: [{ type: 'cli' }],
      capabilities: ['build']
    });

    const scenariosPath = path.join(process.cwd(), 'scenarios');
    
    // We expect the server to be running and handle the /evaluate request
    const outcome = await run([manifest], scenariosPath, remoteUrl);

    expect(outcome.totalSubjects).toBe(1);
    expect(outcome.subjects[0].projectId).toBe('e2e-test');
    expect(outcome.subjects[0].results.length).toBeGreaterThan(0);
    expect(outcome.totalPassed).toBeGreaterThan(0);
  });

  it('should report failure if remote server returns failure', async () => {
    // A manifest that will definitely fail (e.g., no capabilities)
    const manifest = JSON.stringify({
      projectId: 'e2e-fail',
      environment: 'test',
      interfaces: [{ type: 'cli' }],
      capabilities: ['non-existent']
    });

    const scenariosPath = path.join(process.cwd(), 'scenarios');
    const outcome = await run([manifest], scenariosPath, remoteUrl);

    expect(outcome.totalSubjects).toBe(1);
    expect(outcome.subjects[0].projectId).toBe('e2e-fail');
    expect(outcome.subjects[0].results.length).toBe(0); // No scenarios resolved
  });
});
