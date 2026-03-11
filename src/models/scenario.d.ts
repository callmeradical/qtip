import { z } from 'zod';
export declare const AcceptanceCriterionSchema: z.ZodObject<{
    id: z.ZodString;
    description: z.ZodString;
}, z.core.$strip>;
export declare const InteractionTypeSchema: z.ZodEnum<{
    api: "api";
    cli: "cli";
    ui: "ui";
    logs: "logs";
}>;
export declare const ApiInteractionSchema: z.ZodObject<{
    type: z.ZodLiteral<"api">;
    request: z.ZodObject<{
        method: z.ZodString;
        path: z.ZodString;
        body: z.ZodOptional<z.ZodAny>;
        headers: z.ZodOptional<z.ZodRecord<z.ZodString, z.core.SomeType>>;
    }, z.core.$strip>;
}, z.core.$strip>;
export declare const CliInteractionSchema: z.ZodObject<{
    type: z.ZodLiteral<"cli">;
    command: z.ZodString;
}, z.core.$strip>;
export declare const LogInteractionSchema: z.ZodObject<{
    type: z.ZodLiteral<"logs">;
    query: z.ZodString;
}, z.core.$strip>;
export declare const InteractionSchema: z.ZodDiscriminatedUnion<[z.ZodObject<{
    type: z.ZodLiteral<"api">;
    request: z.ZodObject<{
        method: z.ZodString;
        path: z.ZodString;
        body: z.ZodOptional<z.ZodAny>;
        headers: z.ZodOptional<z.ZodRecord<z.ZodString, z.core.SomeType>>;
    }, z.core.$strip>;
}, z.core.$strip>, z.ZodObject<{
    type: z.ZodLiteral<"cli">;
    command: z.ZodString;
}, z.core.$strip>, z.ZodObject<{
    type: z.ZodLiteral<"logs">;
    query: z.ZodString;
}, z.core.$strip>], "type">;
export declare const CheckTypeSchema: z.ZodEnum<{
    status_code: "status_code";
    json_path: "json_path";
    stdout: "stdout";
    stderr: "stderr";
    log_contains: "log_contains";
    log_not_contains: "log_not_contains";
}>;
export declare const CheckSchema: z.ZodObject<{
    type: z.ZodEnum<{
        status_code: "status_code";
        json_path: "json_path";
        stdout: "stdout";
        stderr: "stderr";
        log_contains: "log_contains";
        log_not_contains: "log_not_contains";
    }>;
    expected: z.ZodOptional<z.ZodAny>;
    path: z.ZodOptional<z.ZodString>;
    exists: z.ZodOptional<z.ZodBoolean>;
    acceptance_criteria: z.ZodString;
}, z.core.$strip>;
export declare const ScenarioSchema: z.ZodObject<{
    id: z.ZodString;
    name: z.ZodString;
    applies_to: z.ZodObject<{
        capabilities: z.ZodArray<z.ZodString>;
        interfaces: z.ZodArray<z.ZodEnum<{
            api: "api";
            cli: "cli";
            ui: "ui";
            logs: "logs";
        }>>;
        environments: z.ZodOptional<z.ZodArray<z.ZodString>>;
    }, z.core.$strip>;
    acceptance_criteria: z.ZodArray<z.ZodObject<{
        id: z.ZodString;
        description: z.ZodString;
    }, z.core.$strip>>;
    interaction: z.ZodDiscriminatedUnion<[z.ZodObject<{
        type: z.ZodLiteral<"api">;
        request: z.ZodObject<{
            method: z.ZodString;
            path: z.ZodString;
            body: z.ZodOptional<z.ZodAny>;
            headers: z.ZodOptional<z.ZodRecord<z.ZodString, z.core.SomeType>>;
        }, z.core.$strip>;
    }, z.core.$strip>, z.ZodObject<{
        type: z.ZodLiteral<"cli">;
        command: z.ZodString;
    }, z.core.$strip>, z.ZodObject<{
        type: z.ZodLiteral<"logs">;
        query: z.ZodString;
    }, z.core.$strip>], "type">;
    checks: z.ZodArray<z.ZodObject<{
        type: z.ZodEnum<{
            status_code: "status_code";
            json_path: "json_path";
            stdout: "stdout";
            stderr: "stderr";
            log_contains: "log_contains";
            log_not_contains: "log_not_contains";
        }>;
        expected: z.ZodOptional<z.ZodAny>;
        path: z.ZodOptional<z.ZodString>;
        exists: z.ZodOptional<z.ZodBoolean>;
        acceptance_criteria: z.ZodString;
    }, z.core.$strip>>;
}, z.core.$strip>;
export type Scenario = z.infer<typeof ScenarioSchema>;
export type AcceptanceCriterion = z.infer<typeof AcceptanceCriterionSchema>;
export type Interaction = z.infer<typeof InteractionSchema>;
export type Check = z.infer<typeof CheckSchema>;
//# sourceMappingURL=scenario.d.ts.map