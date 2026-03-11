import { Scenario } from '../models/scenario';
export declare class ScenarioLoader {
    private scenariosSource;
    constructor(scenariosSource: string);
    loadAll(): Promise<Scenario[]>;
    private loadFromUrl;
    private loadFromLocal;
    private recursiveReaddir;
}
//# sourceMappingURL=scenario-loader.d.ts.map