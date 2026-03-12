import axios, { AxiosResponse } from 'axios';
import { SubjectManifest } from '../models/subject-manifest';

// Re-defining interface to avoid circular dependency with cli.ts for now
// Ideally these should be in a shared types file
export interface SubjectResult {
  projectId: string;
  results: any[];
  passedCount: number;
  failedCount: number;
}

export class RemoteEvaluator {
  async evaluate(manifest: SubjectManifest, remoteUrl: string): Promise<SubjectResult> {
    const url = `${remoteUrl.replace(/\/$/, '')}/evaluate`;
    
    try {
      const response: AxiosResponse = await axios({
        method: 'POST',
        url,
        data: manifest,
        validateStatus: () => true, // Don't throw on error status codes
        timeout: 60000,
      });

      if (response.status >= 400) {
        throw new Error(`Remote evaluation failed: ${response.status} - ${response.data.message || JSON.stringify(response.data)}`);
      }

      const data = response.data;

      return {
        projectId: manifest.projectId,
        results: data.results || [],
        passedCount: data.summary.passed,
        failedCount: data.summary.failed,
      };
    } catch (error: any) {
      if (error.message.startsWith('Remote evaluation failed')) {
        throw error;
      }
      throw new Error(`Remote evaluation request failed: ${error.message}`);
    }
  }
}
