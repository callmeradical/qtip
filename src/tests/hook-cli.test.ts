import { HookInstaller } from "../guard/hook-installer";
import * as fs from "fs";
import * as path from "path";

describe("HookInstaller", () => {
  const gitDir = path.join(process.cwd(), ".git");
  const hooksDir = path.join(gitDir, "hooks");
  const preCommitPath = path.join(hooksDir, "pre-commit");

  beforeEach(() => {
    if (!fs.existsSync(gitDir)) fs.mkdirSync(gitDir);
    if (!fs.existsSync(hooksDir)) fs.mkdirSync(hooksDir);
  });

  afterEach(() => {
    if (fs.existsSync(preCommitPath)) fs.unlinkSync(preCommitPath);
  });

  it("should install a pre-commit hook", () => {
    const installer = new HookInstaller();
    installer.install("pre-commit");

    expect(fs.existsSync(preCommitPath)).toBe(true);
    const content = fs.readFileSync(preCommitPath, "utf-8");
    expect(content).toContain("npm run qtip:evaluate");
  });

  it("should throw an error for invalid hook name", () => {
    const installer = new HookInstaller();
    expect(() => installer.install("invalid-hook")).toThrow("Invalid hook name");
  });
});
