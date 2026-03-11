import axios, { AxiosResponse } from 'axios';
import { SubjectManifest } from '../models/subject-manifest';
import { Scenario, ApiInteractionSchema } from '../models/scenario';

export interface ApiEvidence {
  status: number;
  data: any;
  headers: Record<string, string>;
  responseTime: number;
}

export class ApiAdapter {
  async execute(manifest: SubjectManifest, scenario: Scenario): Promise<ApiEvidence> {
    const interaction = ApiInteractionSchema.parse(scenario.interaction);
    const apiInterface = manifest.interfaces.find((i) => i.type === 'api');

    if (!apiInterface || !apiInterface.baseUrl) {
      throw new Error(`API interface not found in manifest for project ${manifest.projectId}`);
    }

    const url = `${apiInterface.baseUrl}${interaction.request.path}`;
    const start = Date.now();

    try {
      const response: AxiosResponse = await axios({
        method: interaction.request.method,
        url,
        data: interaction.request.body,
        headers: interaction.request.headers,
        validateStatus: () => true, // Don't throw on error status codes
      });

      return {
        status: response.status,
        data: response.data,
        headers: response.headers as Record<string, string>,
        responseTime: Date.now() - start,
      };
    } catch (error) {
      if (axios.isAxiosError(error)) {
        return {
          status: error.response?.status || 500,
          data: error.response?.data || error.message,
          headers: (error.response?.headers as Record<string, string>) || {},
          responseTime: Date.now() - start,
        };
      }
      throw error;
    }
  }
}
