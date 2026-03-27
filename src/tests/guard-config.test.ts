import { GuardConfigSchema } from "../guard/config";

describe("GuardConfigSchema", () => {
  it("should validate a correct configuration", () => {
    const validConfig = {
      secondary_repo: "./scenarios-repo",
      watched_paths: ["src/**/*.ts"],
      scenarios_mapping: [
        {
          path: "src/auth.ts",
          scenario_name: "Auth Flow",
          acceptance_criteria: ["User can login", "Invalid login fails"]
        }
      ]
    };

    const result = GuardConfigSchema.safeParse(validConfig);
    expect(result.success).toBe(true);
  });

  it("should fail validation if secondary_repo is missing", () => {
    const invalidConfig = {
      watched_paths: ["src/**/*.ts"],
      scenarios_mapping: []
    };

    const result = GuardConfigSchema.safeParse(invalidConfig);
    expect(result.success).toBe(false);
  });
});
