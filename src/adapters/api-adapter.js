"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.ApiAdapter = void 0;
const axios_1 = __importStar(require("axios"));
const subject_manifest_1 = require("../models/subject-manifest");
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
        try {
            const response = await (0, axios_1.default)({
                method: interaction.request.method,
                url,
                data: interaction.request.body,
                headers: interaction.request.headers,
                validateStatus: () => true, // Don't throw on error status codes
            });
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
//# sourceMappingURL=api-adapter.js.map