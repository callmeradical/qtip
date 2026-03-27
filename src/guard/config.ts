import { z } from "zod";

export const ScenarioMappingSchema = z.object({
  path: z.string(),
  scenario_name: z.string(),
  acceptance_criteria: z.array(z.string()).min(1),
});

export const GuardConfigSchema = z.object({
  secondary_repo: z.string(),
  watched_paths: z.array(z.string()),
  scenarios_mapping: z.array(ScenarioMappingSchema),
});

export type GuardConfig = z.infer<typeof GuardConfigSchema>;
