import axios, { AxiosResponse, AxiosRequestConfig, RawAxiosRequestHeaders } from 'axios';
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
    
    const apiInterface = manifest.interfaces.find((i) => {
      if (interaction.service) {
        return i.type === 'api' && i.name === interaction.service;
      }
      return i.type === 'api';
    });

    if (!apiInterface || !apiInterface.baseUrl) {
      const serviceName = interaction.service ? `service '${interaction.service}'` : 'API interface';
      throw new Error(`${serviceName} not found in manifest for project ${manifest.projectId}`);
    }

    const url = `${apiInterface.baseUrl}${interaction.request.path}`;
    const start = Date.now();

    const config: AxiosRequestConfig = {
      method: interaction.request.method,
      url,
      data: interaction.request.body,
      headers: interaction.request.headers as RawAxiosRequestHeaders,
      validateStatus: () => true,
      timeout: 30000,
    };

    try {
      const response: AxiosResponse = await axios(config);

      return {
        status: response.status,
        data: response.data,
        headers: response.headers as Record<string, string>,
        responseTime: Date.now() - start,
      };
    } catch (error: any) {
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
