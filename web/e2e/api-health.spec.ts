import { test, expect } from '@playwright/test';

test.describe('API Health Checks', () => {
  test('health endpoint responds', async ({ request }) => {
    const response = await request.get('/api/v1/health');
    expect(response.status()).toBe(200);
    
    const data = await response.json();
    expect(data).toHaveProperty('status');
    expect(data.status).toBe('ok');
  });

  test('detailed health endpoint responds', async ({ request }) => {
    const response = await request.get('/api/v1/health/detailed');
    expect(response.status()).toBe(200);
    
    const data = await response.json();
    expect(data).toHaveProperty('status');
    expect(data).toHaveProperty('version');
    expect(data).toHaveProperty('uptime');
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