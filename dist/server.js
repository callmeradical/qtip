"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const express_1 = __importDefault(require("express"));
const cors_1 = __importDefault(require("cors"));
const evaluation_1 = __importDefault(require("./routes/evaluation"));
const app = (0, express_1.default)();
const port = process.env.PORT || 3000;
app.use((0, cors_1.default)());
app.use(express_1.default.json());
app.use('/api/v1', evaluation_1.default);
app.get('/health', (req, res) => {
    res.json({ status: 'ok' });
});
if (process.env.NODE_ENV !== 'test') {
    app.listen(port, () => {
        console.log(`qtip runner platform listening at http://localhost:${port}`);
    });
}
exports.default = app;
