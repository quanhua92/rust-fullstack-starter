import { test, expect } from '@playwright/test';

test.describe('API Health Checks', () => {
  test('health endpoint responds', async ({ request }) => {
    const response = await request.get('/api/v1/health');
    await expect(response).toBeOK();

    const data = await response.json();
    expect(data).toEqual(expect.objectContaining({
      success: true,
      data: expect.objectContaining({
        status: 'healthy',
      }),
    }));
  });

  test('detailed health endpoint responds', async ({ request }) => {
    const response = await request.get('/api/v1/health/detailed');
    await expect(response).toBeOK();
    
    const data = await response.json();
    expect(data).toEqual(expect.objectContaining({
      success: true,
      data: expect.objectContaining({
        status: 'healthy',
        checks: expect.any(Object),
      }),
    }));
  });

  test('API documentation is accessible', async ({ request }) => {
    const response = await request.get('/api-docs');
    await expect(response).toBeOK();
  });

  test('OpenAPI spec is accessible', async ({ request }) => {
    const response = await request.get('/api-docs/openapi.json');
    await expect(response).toBeOK();
    
    const data = await response.json();
    expect(data).toEqual(expect.objectContaining({
      openapi: expect.any(String),
      info: expect.any(Object),
    }));
  });
});