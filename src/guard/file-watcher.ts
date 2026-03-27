import * as chokidar from "chokidar";

export class FileWatcher {
  private watcher?: chokidar.FSWatcher;

  constructor(private paths: string[]) {}

  start(onChanged: (path: string) => void): void {
    this.watcher = chokidar.watch(this.paths, {
      ignoreInitial: true,
      persistent: true,
    });

    this.watcher.on("change", (path) => {
      onChanged(path);
    });

    this.watcher.on("error", (error) => {
      console.error(`FileWatcher error: ${error}`);
    });
  }

  async stop(): Promise<void> {
    if (this.watcher) {
      await this.watcher.close();
    }
  }
}
