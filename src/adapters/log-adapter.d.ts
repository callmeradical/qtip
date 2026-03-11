import { SubjectManifest } from '../models/subject-manifest';
import { Scenario } from '../models/scenario';
export interface LogEvidence {
    found: boolean;
    content: string;
}
export declare class LogAdapter {
    execute(manifest: SubjectManifest, scenario: Scenario): Promise<LogEvidence>;
}
//# sourceMappingURL=log-adapter.d.ts.map