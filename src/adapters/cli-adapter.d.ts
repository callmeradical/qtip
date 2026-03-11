import { SubjectManifest } from '../models/subject-manifest';
import { Scenario } from '../models/scenario';
export interface CliEvidence {
    status: number;
    stdout: string;
    stderr: string;
}
export declare class CliAdapter {
    execute(_manifest: SubjectManifest, scenario: Scenario): Promise<CliEvidence>;
}
//# sourceMappingURL=cli-adapter.d.ts.map