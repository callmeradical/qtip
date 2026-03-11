import { z } from 'zod';

export const AcceptanceCriterionSchema = z.object({
  id: z.string(),
  description: z.string(),
});

export const InteractionTypeSchema = z.enum(['api', 'cli', 'ui', 'logs']);

export const ApiInteractionSchema = z.object({
  type: z.literal('api'),
  request: z.object({
    method: z.string(),
    path: z.string(),
    body: z.any().optional(),
    headers: z.record(z.string(), z.string()).optional(),
  }),
});

export const CliInteractionSchema = z.object({
  type: z.literal('cli'),
  command: z.string(),
});

export const LogInteractionSchema = z.object({
  type: z.literal('logs'),
  query: z.string(),
});

export const UiInteractionSchema = z.object({
  type: z.literal('ui'),
});

export const InteractionSchema = z.discriminatedUnion('type', [
  ApiInteractionSchema,
  CliInteractionSchema,
  LogInteractionSchema,
  UiInteractionSchema,
]);

export const CheckTypeSchema = z.enum(['status_code', 'json_path', 'stdout', 'stderr', 'log_contains', 'log_not_contains']);

export const CheckSchema = z.object({
  type: CheckTypeSchema,
  expected: z.any().optional(),
  path: z.string().optional(),
  exists: z.boolean().optional(),
  acceptance_criteria: z.string(),
});

export const ScenarioSchema = z.object({
  id: z.string(),
  name: z.string(),
  applies_to: z.object({
    capabilities: z.array(z.string()),
    interfaces: z.array(InteractionTypeSchema),
    environments: z.array(z.string()).optional(),
  }),
  acceptance_criteria: z.array(AcceptanceCriterionSchema),
  interaction: InteractionSchema,
  checks: z.array(CheckSchema),
});

export type Scenario = z.infer<typeof ScenarioSchema>;
export type AcceptanceCriterion = z.infer<typeof AcceptanceCriterionSchema>;
export type Interaction = z.infer<typeof InteractionSchema>;
export type Check = z.infer<typeof CheckSchema>;
