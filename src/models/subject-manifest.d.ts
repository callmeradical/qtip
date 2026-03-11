import { z } from 'zod';
export declare const SubjectInterfaceSchema: z.ZodObject<{
    type: z.ZodEnum<{
        api: "api";
        cli: "cli";
        ui: "ui";
        logs: "logs";
    }>;
    baseUrl: z.ZodOptional<z.ZodString>;
}, z.core.$strip>;
export declare const ObservabilitySchema: z.ZodObject<{
    logs: z.ZodOptional<z.ZodObject<{
        type: z.ZodEnum<{
            file: "file";
            url: "url";
        }>;
        path: z.ZodOptional<z.ZodString>;
        url: z.ZodOptional<z.ZodString>;
    }, z.core.$strip>>;
}, z.core.$strip>;
export declare const SubjectManifestSchema: z.ZodObject<{
    projectId: z.ZodString;
    repoUrl: z.ZodOptional<z.ZodString>;
    commit: z.ZodOptional<z.ZodString>;
    environment: z.ZodString;
    interfaces: z.ZodArray<z.ZodObject<{
        type: z.ZodEnum<{
            api: "api";
            cli: "cli";
            ui: "ui";
            logs: "logs";
        }>;
        baseUrl: z.ZodOptional<z.ZodString>;
    }, z.core.$strip>>;
    observability: z.ZodOptional<z.ZodObject<{
        logs: z.ZodOptional<z.ZodObject<{
            type: z.ZodEnum<{
                file: "file";
                url: "url";
            }>;
            path: z.ZodOptional<z.ZodString>;
            url: z.ZodOptional<z.ZodString>;
        }, z.core.$strip>>;
    }, z.core.$strip>>;
    capabilities: z.ZodArray<z.ZodString>;
}, z.core.$strip>;
export type SubjectManifest = z.infer<typeof SubjectManifestSchema>;
//# sourceMappingURL=subject-manifest.d.ts.map