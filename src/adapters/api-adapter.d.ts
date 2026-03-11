import { SubjectManifest } from '../models/subject-manifest';
import { Scenario } from '../models/scenario';
export interface ApiEvidence {
    status: number;
    data: any;
    headers: Record<string, string>;
    responseTime: number;
}
export declare class ApiAdapter {
    execute(manifest: SubjectManifest, scenario: Scenario): Promise<ApiEvidence>;
}
//# sourceMappingURL=api-adapter.d.ts.map