import { SubjectInterfaceSchema } from '../models/subject-manifest';
import { ApiInteractionSchema } from '../models/scenario';

describe('Models Schema Validation', () => {
  describe('SubjectInterfaceSchema', () => {
    it('should allow optional name field', () => {
      const validInterface = {
        type: 'api',
        name: 'auth-service',
        baseUrl: 'http://auth.svc'
      };
      const result = SubjectInterfaceSchema.safeParse(validInterface);
      expect(result.success).toBe(true);
      if (result.success) {
        expect(result.data.name).toBe('auth-service');
      }
    });

    it('should work without name field', () => {
      const validInterface = {
        type: 'api',
        baseUrl: 'http://auth.svc'
      };
      const result = SubjectInterfaceSchema.safeParse(validInterface);
      expect(result.success).toBe(true);
    });
  });

  describe('ApiInteractionSchema', () => {
    it('should allow optional service field', () => {
      const validInteraction = {
        type: 'api',
        service: 'user-service',
        request: {
          method: 'GET',
          path: '/users'
        }
      };
      const result = ApiInteractionSchema.safeParse(validInteraction);
      expect(result.success).toBe(true);
      if (result.success) {
        expect(result.data.service).toBe('user-service');
      }
    });

    it('should work without service field', () => {
      const validInteraction = {
        type: 'api',
        request: {
          method: 'GET',
          path: '/users'
        }
      };
      const result = ApiInteractionSchema.safeParse(validInteraction);
      expect(result.success).toBe(true);
    });
  });
});
