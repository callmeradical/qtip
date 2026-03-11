"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.ApiAdapter = void 0;
const axios_1 = __importDefault(require("axios"));
const scenario_1 = require("../models/scenario");
class ApiAdapter {
    async execute(manifest, scenario) {
        const interaction = scenario_1.ApiInteractionSchema.parse(scenario.interaction);
        const apiInterface = manifest.interfaces.find((i) => i.type === 'api');
        if (!apiInterface || !apiInterface.baseUrl) {
            throw new Error(`API interface not found in manifest for project ${manifest.projectId}`);
        }
        const url = `${apiInterface.baseUrl}${interaction.request.path}`;
        const start = Date.now();
        const config = {
            method: interaction.request.method,
            url,
            data: interaction.request.body,
            headers: interaction.request.headers,
            validateStatus: () => true,
        };
        try {
            const response = await (0, axios_1.default)(config);
            return {
                status: response.status,
                data: response.data,
                headers: response.headers,
                responseTime: Date.now() - start,
            };
        }
        catch (error) {
            if (axios_1.default.isAxiosError(error)) {
                return {
                    status: error.response?.status || 500,
                    data: error.response?.data || error.message,
                    headers: error.response?.headers || {},
                    responseTime: Date.now() - start,
                };
            }
            throw error;
        }
    }
}
exports.ApiAdapter = ApiAdapter;
