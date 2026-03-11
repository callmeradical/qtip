"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.SubjectManifestSchema = exports.ObservabilitySchema = exports.SubjectInterfaceSchema = void 0;
const zod_1 = require("zod");
exports.SubjectInterfaceSchema = zod_1.z.object({
    type: zod_1.z.enum(['api', 'cli', 'ui', 'logs']),
    baseUrl: zod_1.z.string().url().optional(),
});
exports.ObservabilitySchema = zod_1.z.object({
    logs: zod_1.z.object({
        type: zod_1.z.enum(['file', 'url']),
        path: zod_1.z.string().optional(),
        url: zod_1.z.string().url().optional(),
    }).optional(),
});
exports.SubjectManifestSchema = zod_1.z.object({
    projectId: zod_1.z.string(),
    repoUrl: zod_1.z.string().url().optional(),
    commit: zod_1.z.string().optional(),
    environment: zod_1.z.string(),
    interfaces: zod_1.z.array(exports.SubjectInterfaceSchema),
    observability: exports.ObservabilitySchema.optional(),
    capabilities: zod_1.z.array(zod_1.z.string()),
});
//# sourceMappingURL=subject-manifest.js.map