"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.ScenarioSchema = exports.CheckSchema = exports.CheckTypeSchema = exports.InteractionSchema = exports.LogInteractionSchema = exports.CliInteractionSchema = exports.ApiInteractionSchema = exports.InteractionTypeSchema = exports.AcceptanceCriterionSchema = void 0;
const zod_1 = require("zod");
exports.AcceptanceCriterionSchema = zod_1.z.object({
    id: zod_1.z.string(),
    description: zod_1.z.string(),
});
exports.InteractionTypeSchema = zod_1.z.enum(['api', 'cli', 'ui', 'logs']);
exports.ApiInteractionSchema = zod_1.z.object({
    type: zod_1.z.literal('api'),
    request: zod_1.z.object({
        method: zod_1.z.string(),
        path: zod_1.z.string(),
        body: zod_1.z.any().optional(),
        headers: zod_1.z.record(zod_1.z.string()).optional(),
    }),
});
exports.CliInteractionSchema = zod_1.z.object({
    type: zod_1.z.literal('cli'),
    command: zod_1.z.string(),
});
exports.LogInteractionSchema = zod_1.z.object({
    type: zod_1.z.literal('logs'),
    query: zod_1.z.string(),
});
exports.InteractionSchema = zod_1.z.discriminatedUnion('type', [
    exports.ApiInteractionSchema,
    exports.CliInteractionSchema,
    exports.LogInteractionSchema,
]);
exports.CheckTypeSchema = zod_1.z.enum(['status_code', 'json_path', 'stdout', 'stderr', 'log_contains', 'log_not_contains']);
exports.CheckSchema = zod_1.z.object({
    type: exports.CheckTypeSchema,
    expected: zod_1.z.any().optional(),
    path: zod_1.z.string().optional(),
    exists: zod_1.z.boolean().optional(),
    acceptance_criteria: zod_1.z.string(),
});
exports.ScenarioSchema = zod_1.z.object({
    id: zod_1.z.string(),
    name: zod_1.z.string(),
    applies_to: zod_1.z.object({
        capabilities: zod_1.z.array(zod_1.z.string()),
        interfaces: zod_1.z.array(exports.InteractionTypeSchema),
        environments: zod_1.z.array(zod_1.z.string()).optional(),
    }),
    acceptance_criteria: zod_1.z.array(exports.AcceptanceCriterionSchema),
    interaction: exports.InteractionSchema,
    checks: zod_1.z.array(exports.CheckSchema),
});
//# sourceMappingURL=scenario.js.map