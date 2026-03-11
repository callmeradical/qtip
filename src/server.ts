import express from 'express';
import cors from 'cors';
import evaluationRoutes from './routes/evaluation';

const app = express();
const port = process.env.PORT || 3000;

app.use(cors());
app.use(express.json());

app.use('/api/v1', evaluationRoutes);

app.get('/health', (req, res) => {
  res.json({ status: 'ok' });
});

if (process.env.NODE_ENV !== 'test') {
  app.listen(port, () => {
    console.log(`qtip runner platform listening at http://localhost:${port}`);
  });
}

export default app;
