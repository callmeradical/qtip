import { Scenario } from '../models/scenario';
import { SubjectManifest } from '../models/subject-manifest';
export declare class ScenarioResolver {
    private scenarios;
    constructor(scenarios?: Scenario[]);
    resolve(manifest: SubjectManifest): Scenario[];
}
//# sourceMappingURL=scenario-resolver.d.ts.map