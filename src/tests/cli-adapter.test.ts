import { CliAdapter } from '../adapters/cli-adapter';
import { SubjectManifest } from '../models/subject-manifest';
import { Scenario } from '../models/scenario';
import * as child_process from 'child_process';
import { promisify } from 'util';

jest.mock('child_process');
const mockedExec = child_process.exec as unknown as jest.Mock;

describe('CliAdapter', () => {
  let adapter: CliAdapter;
  let manifest: SubjectManifest;

  beforeEach(() => {
    adapter = new CliAdapter();
    manifest = {
      projectId: 'test-project',
      environment: 'local',
      interfaces: [{ type: 'cli' }],
      capabilities: ['test']
    } as SubjectManifest;
    jest.clearAllMocks();
  });

  it('should be defined', () => {
    expect(adapter).toBeDefined();
  });

  it('should return 0 status and stdout on successful command', async () => {
    const scenario: Scenario = {
      id: 'TEST-1',
      name: 'Test Scenario',
      applies_to: { capabilities: ['test'], interfaces: ['cli'] },
      interaction: {
        type: 'cli',
        command: 'echo "hello"'
      },
      acceptance_criteria: [
        { id: 'AC-1', description: 'Should print hello' }
      ],
      checks: [
        { type: 'status_code', expected: 0, acceptance_criteria: 'AC-1' }
      ]
    };

    // Mock successful execution
    mockedExec.mockImplementation((cmd, arg2, arg3) => {
      const callback = typeof arg2 === 'function' ? arg2 : arg3;
      if (callback) {
        callback(null, { stdout: 'hello\n', stderr: '' });
      }
      return {} as any;
    });

    const result = await adapter.execute(manifest, scenario);

    expect(result.status).toBe(0);
    expect(result.stdout).toBe('hello\n');
    expect(result.stderr).toBe('');
    expect(mockedExec).toHaveBeenCalled();
    expect(mockedExec.mock.calls[0][0]).toBe('echo "hello"');
  });

  it('should return error status and stderr on failing command', async () => {
    const scenario: Scenario = {
      id: 'TEST-FAIL',
      name: 'Fail Scenario',
      applies_to: { capabilities: ['test'], interfaces: ['cli'] },
      interaction: {
        type: 'cli',
        command: 'ls non-existent-file'
      },
      acceptance_criteria: [
        { id: 'AC-1', description: 'Should fail' }
      ],
      checks: [
        { type: 'status_code', expected: 1, acceptance_criteria: 'AC-1' }
      ]
    };

    // Mock failed execution
    mockedExec.mockImplementation((cmd, arg2, arg3) => {
      const callback = typeof arg2 === 'function' ? arg2 : arg3;
      if (callback) {
        callback({ code: 1, stderr: 'No such file', stdout: '' } as any, { stdout: '', stderr: 'No such file' });
      }
      return {} as any;
    });

    const result = await adapter.execute(manifest, scenario);

    expect(result.status).toBe(1);
    expect(result.stdout).toBe('');
    expect(result.stderr).toBe('No such file');
  });

  it('should use default values in catch block when error properties are missing', async () => {
    const scenario: Scenario = {
      id: 'TEST-FAIL-MINIMAL',
      name: 'Fail Minimal Scenario',
      applies_to: { capabilities: ['test'], interfaces: ['cli'] },
      interaction: {
        type: 'cli',
        command: 'ls'
      },
      acceptance_criteria: [
        { id: 'AC-1', description: 'Should fail' }
      ],
      checks: [
        { type: 'status_code', expected: 1, acceptance_criteria: 'AC-1' }
      ]
    };

    // Mock failed execution with minimal error object
    mockedExec.mockImplementation((cmd, arg2, arg3) => {
      const callback = typeof arg2 === 'function' ? arg2 : arg3;
      if (callback) {
        callback({ message: 'Unknown error' } as any, { stdout: '', stderr: '' });
      }
      return {} as any;
    });

    const result = await adapter.execute(manifest, scenario);

    expect(result.status).toBe(1);
    expect(result.stdout).toBe('');
    expect(result.stderr).toBe('Unknown error');
  });

  it('should capture both stdout and stderr when both are populated', async () => {
    const scenario: Scenario = {
      id: 'TEST-MIXED',
      name: 'Mixed Scenario',
      applies_to: { capabilities: ['test'], interfaces: ['cli'] },
      interaction: {
        type: 'cli',
        command: 'my-command'
      },
      acceptance_criteria: [
        { id: 'AC-1', description: 'Should capture both' }
      ],
      checks: [
        { type: 'status_code', expected: 0, acceptance_criteria: 'AC-1' }
      ]
    };

    mockedExec.mockImplementation((cmd, arg2, arg3) => {
      const callback = typeof arg2 === 'function' ? arg2 : arg3;
      if (callback) {
        callback(null, { stdout: 'Partial success', stderr: 'Minor warning' });
      }
      return {} as any;
    });

    const result = await adapter.execute(manifest, scenario);

    expect(result.status).toBe(0);
    expect(result.stdout).toBe('Partial success');
    expect(result.stderr).toBe('Minor warning');
  });
});
