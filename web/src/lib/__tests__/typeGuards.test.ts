import { describe, it, expect } from 'vitest';

// Import type guards - we'll create them as utility functions
interface ProbeResponse {
  probe?: string;
  status?: string;
  timestamp?: string;
  checks?: Record<string, any>;
}

// Type guards for testing
const isProbeResponse = (data: unknown): data is ProbeResponse => {
  return (
    typeof data === 'object' &&
    data !== null &&
    typeof (data as ProbeResponse).status === 'string'
  );
};

const getProbeStatus = (data: unknown): string => {
  if (isProbeResponse(data)) {
    return data.status || 'unknown';
  }
  return 'unknown';
};

const getProbeTimestamp = (data: unknown): string | undefined => {
  if (isProbeResponse(data)) {
    return data.timestamp;
  }
  return undefined;
};

const getProbeChecks = (data: unknown): Record<string, any> | undefined => {
  if (isProbeResponse(data) && data.checks) {
    return data.checks;
  }
  return undefined;
};

describe('Type Guards', () => {
  describe('isProbeResponse', () => {
    it('returns true for valid probe response', () => {
      const validResponse = {
        status: 'healthy',
        probe: 'liveness',
        timestamp: '2024-01-01T00:00:00Z'
      };

      expect(isProbeResponse(validResponse)).toBe(true);
    });

    it('returns false for null', () => {
      expect(isProbeResponse(null)).toBe(false);
    });

    it('returns false for undefined', () => {
      expect(isProbeResponse(undefined)).toBe(false);
    });

    it('returns false for string', () => {
      expect(isProbeResponse('not an object')).toBe(false);
    });

    it('returns false for object without status', () => {
      const invalidResponse = {
        probe: 'liveness',
        timestamp: '2024-01-01T00:00:00Z'
      };

      expect(isProbeResponse(invalidResponse)).toBe(false);
    });

    it('returns false for object with non-string status', () => {
      const invalidResponse = {
        status: 123,
        probe: 'liveness'
      };

      expect(isProbeResponse(invalidResponse)).toBe(false);
    });
  });

  describe('getProbeStatus', () => {
    it('returns status from valid probe response', () => {
      const response = {
        status: 'healthy',
        probe: 'liveness'
      };

      expect(getProbeStatus(response)).toBe('healthy');
    });

    it('returns "unknown" for null', () => {
      expect(getProbeStatus(null)).toBe('unknown');
    });

    it('returns "unknown" for invalid data', () => {
      expect(getProbeStatus('invalid')).toBe('unknown');
    });

    it('returns "unknown" when status is empty', () => {
      const response = {
        status: '',
        probe: 'liveness'
      };

      expect(getProbeStatus(response)).toBe('unknown');
    });
  });

  describe('getProbeTimestamp', () => {
    it('returns timestamp from valid probe response', () => {
      const response = {
        status: 'healthy',
        timestamp: '2024-01-01T00:00:00Z'
      };

      expect(getProbeTimestamp(response)).toBe('2024-01-01T00:00:00Z');
    });

    it('returns undefined for invalid data', () => {
      expect(getProbeTimestamp(null)).toBeUndefined();
      expect(getProbeTimestamp('invalid')).toBeUndefined();
    });

    it('returns undefined when timestamp is missing', () => {
      const response = {
        status: 'healthy',
        probe: 'liveness'
      };

      expect(getProbeTimestamp(response)).toBeUndefined();
    });
  });

  describe('getProbeChecks', () => {
    it('returns checks from valid probe response', () => {
      const checks = { database: { status: 'healthy' } };
      const response = {
        status: 'healthy',
        checks
      };

      expect(getProbeChecks(response)).toEqual(checks);
    });

    it('returns undefined for invalid data', () => {
      expect(getProbeChecks(null)).toBeUndefined();
      expect(getProbeChecks('invalid')).toBeUndefined();
    });

    it('returns undefined when checks is missing', () => {
      const response = {
        status: 'healthy',
        probe: 'liveness'
      };

      expect(getProbeChecks(response)).toBeUndefined();
    });
  });
});