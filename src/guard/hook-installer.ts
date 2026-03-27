import * as fs from "fs";
import * as path from "path";

export class HookInstaller {
  install(hookName: string): void {
    const validHooks = ["pre-commit", "pre-push"];
    if (!validHooks.includes(hookName)) {
      throw new Error(`Invalid hook name: ${hookName}. Valid options are: ${validHooks.join(", ")}`);
    }

    const gitDir = path.join(process.cwd(), ".git");
    if (!fs.existsSync(gitDir)) {
      throw new Error("Not a git repository (no .git directory found).");
    }

    const hooksDir = path.join(gitDir, "hooks");
    if (!fs.existsSync(hooksDir)) {
      fs.mkdirSync(hooksDir, { recursive: true });
    }

    const hookPath = path.join(hooksDir, hookName);
    const hookScript = `#!/bin/sh
# qtip generated hook
echo "Running qtip evaluation..."
npm run qtip:evaluate
if [ $? -ne 0 ]; then
  echo "qtip evaluation failed. Action blocked."
  exit 1
fi
`;

    fs.writeFileSync(hookPath, hookScript, { mode: 0o755 });
    console.log(`Successfully installed ${hookName} hook at ${hookPath}`);
  }
}
