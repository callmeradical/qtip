import { z } from 'zod';

export const SubjectInterfaceSchema = z.object({
  type: z.enum(['api', 'cli', 'ui', 'logs']),
  baseUrl: z.string().url().optional(),
});

export const ObservabilitySchema = z.object({
  logs: z.object({
    type: z.enum(['file', 'url']),
    path: z.string().optional(),
    url: z.string().url().optional(),
  }).optional(),
});

export const SubjectManifestSchema = z.object({
  projectId: z.string(),
  repoUrl: z.string().url().optional(),
  commit: z.string().optional(),
  environment: z.string(),
  interfaces: z.array(SubjectInterfaceSchema),
  observability: ObservabilitySchema.optional(),
  capabilities: z.array(z.string()),
});

export type SubjectManifest = z.infer<typeof SubjectManifestSchema>;
