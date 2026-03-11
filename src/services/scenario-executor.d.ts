import { Scenario } from '../models/scenario';
import { SubjectManifest } from '../models/subject-manifest';
export interface EvaluationResult {
    scenarioId: string;
    status: 'passed' | 'failed' | 'error';
    evidence: any;
    failures: string[];
}
export declare class ScenarioExecutor {
    private apiAdapter;
    private cliAdapter;
    private logAdapter;
    execute(manifest: SubjectManifest, scenario: Scenario): Promise<EvaluationResult>;
    private evaluateChecks;
}
//# sourceMappingURL=scenario-executor.d.ts.map