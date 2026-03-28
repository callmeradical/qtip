import { HookInstaller } from "./hook-installer";

function main() {
  const args = process.argv.slice(2);
  const command = args[0];
  const hookName = args[1];

  if (command !== "install") {
    console.error("Usage: qtip-hook install [pre-commit|pre-push]");
    process.exit(1);
  }

  if (!hookName) {
    console.error("Please specify a hook name (pre-commit or pre-push)");
    process.exit(1);
  }

  const installer = new HookInstaller();
  try {
    installer.install(hookName);
  } catch (e: any) {
    console.error(`Installation failed: ${e.message}`);
    process.exit(1);
  }
}

if (require.main === module) {
  main();
}
