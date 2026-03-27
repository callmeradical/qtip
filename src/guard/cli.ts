import { Guard } from "./index";
import * as path from "path";

function main() {
  const args = process.argv.slice(2);
  const configPathArg = args[0] || "qtip-guard.yaml";
  const configPath = path.isAbsolute(configPathArg) 
    ? configPathArg 
    : path.join(process.cwd(), configPathArg);

  console.log(`qtip-guard version 0.1.0`);
  const guard = new Guard(configPath);
  
  try {
    guard.start();
    
    process.on("SIGINT", async () => {
      console.log("\nStopping qtip-guard...");
      await guard.stop();
      process.exit(0);
    });
  } catch (e: any) {
    console.error(`Failed to start qtip-guard: ${e.message}`);
    process.exit(1);
  }
}

if (require.main === module) {
  main();
}
