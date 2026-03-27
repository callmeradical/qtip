import { Guard } from "../guard";
import * as fs from "fs";
import * as path from "path";

// We can't easily test the CLI entry point directly with unit tests without refactoring it
// But we can test if the CLI logic would correctly initialize the Guard

describe("Guard CLI Logic", () => {
  it("should be able to initialize Guard with a provided config path", () => {
    const configPath = "test-config.yaml";
    const guard = new Guard(configPath);
    expect((guard as any).configPath).toBe(configPath);
  });
});
