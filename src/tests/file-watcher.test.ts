import { FileWatcher } from "../guard/file-watcher";
import * as fs from "fs";
import * as path from "path";

describe("FileWatcher", () => {
  const testDir = path.join(process.cwd(), "test-watch");
  const testFile = path.join(testDir, "test.ts");

  beforeEach(() => {
    if (!fs.existsSync(testDir)) {
      fs.mkdirSync(testDir);
    }
  });

  afterEach(() => {
    if (fs.existsSync(testFile)) {
      fs.unlinkSync(testFile);
    }
    if (fs.existsSync(testDir)) {
      fs.rmdirSync(testDir);
    }
  });

  it("should trigger callback when a file is changed", (done) => {
    fs.writeFileSync(testFile, "initial");
    const watcher = new FileWatcher([testDir]);
    
    watcher.start(async (filePath) => {
      expect(filePath).toContain("test.ts");
      await watcher.stop();
      done();
    });

    // Wait a bit for chokidar to ready up
    setTimeout(() => {
      fs.writeFileSync(testFile, "changed");
    }, 100);
  });
});
