import { test, expect } from '@playwright/test';

test.describe('API Health Checks', () => {
  test('health endpoint responds', async ({ request }) => {
    const response = await request.get('/api/v1/health');
    expect(response.status()).toBe(200);
    
    const data = await response.json();
    expect(data).toHaveProperty('data');
    expect(data.data).toHaveProperty('status');
    expect(data.data.status).toBe('healthy');
    expect(data.success).toBe(true);
  });

  test('detailed health endpoint responds', async ({ request }) => {
    const response = await request.get('/api/v1/health/detailed');
    expect(response.status()).toBe(200);
    
    const data = await response.json();
    expect(data).toHaveProperty('data');
    expect(data.data).toHaveProperty('status');
    expect(data.data.status).toBe('healthy');
    expect(data.data).toHaveProperty('checks');
    expect(data.success).toBe(true);
  });

  test('API documentation is accessible', async ({ request }) => {
    const response = await request.get('/api-docs');
    expect(response.status()).toBe(200);
  });

  test('OpenAPI spec is accessible', async ({ request }) => {
    const response = await request.get('/api-docs/openapi.json');
    expect(response.status()).toBe(200);
    
    const data = await response.json();
    expect(data).toHaveProperty('openapi');
    expect(data).toHaveProperty('info');
  });
});