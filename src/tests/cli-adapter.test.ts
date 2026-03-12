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
});
