import { test, expect } from '@playwright/test';
import { LoginPage, RegisterPage, TestDataGenerator } from './page-objects/AuthPage';

test.describe('Authentication with Page Objects', () => {
  test.describe('Registration Flow', () => {
    test('should complete successful registration', async ({ page }) => {
      const registerPage = new RegisterPage(page);
      const testUser = TestDataGenerator.generateUniqueUser();

      await registerPage.goto('/auth/register');
      await registerPage.waitForFormLoad();
      
      await registerPage.register(testUser.username, testUser.email, testUser.password);
      await registerPage.expectRegistrationSuccess();
    });

    test('should show validation errors for invalid data', async ({ page }) => {
      const registerPage = new RegisterPage(page);

      await registerPage.goto('/auth/register');
      await registerPage.waitForFormLoad();

      // Test form exists and can be interacted with
      await registerPage.usernameInput.fill('ab'); // Too short
      await registerPage.emailInput.fill('invalid-email');
      await registerPage.passwordInput.fill('short'); // Too short
      await registerPage.confirmPasswordInput.fill('different');
      
      // Try to submit form to trigger validation
      await registerPage.submitButton.click();
      
      // Form should still be on register page (validation prevented submission)
      await expect(page).toHaveURL(/.*\/auth\/register/);
    });

    test('should handle password mismatch validation', async ({ page }) => {
      const registerPage = new RegisterPage(page);
      const testUser = TestDataGenerator.generateUniqueUser();

      await registerPage.goto('/auth/register');
      await registerPage.waitForFormLoad();

      // Fill form with mismatched passwords
      await registerPage.usernameInput.fill(testUser.username);
      await registerPage.emailInput.fill(testUser.email);
      await registerPage.passwordInput.fill(testUser.password);
      await registerPage.confirmPasswordInput.fill('DifferentPassword');
      
      // Try to submit - should stay on register page due to validation
      await registerPage.submitButton.click();
      await expect(page).toHaveURL(/.*\/auth\/register/);
    });

    test('should navigate to login page', async ({ page }) => {
      const registerPage = new RegisterPage(page);

      await registerPage.goto('/auth/register');
      await registerPage.goToLogin();
      
      await expect(page).toHaveURL(/.*\/auth\/login/);
    });
  });

  test.describe('Login Flow', () => {
    test('should handle login form interaction', async ({ page }) => {
      const loginPage = new LoginPage(page);

      await loginPage.goto('/auth/login');
      await loginPage.waitForFormLoad();

      // Test with some credentials (may fail but should show form was submitted)
      await loginPage.fillCredentials('test@example.com', 'somepassword');
      await loginPage.submitButton.click();

      // Wait for form submission to be processed
      await page.waitForLoadState('networkidle', { timeout: 5000 });
    });

    test('should navigate to register page', async ({ page }) => {
      const loginPage = new LoginPage(page);

      await loginPage.goto('/auth/login');
      await loginPage.goToRegister();
      
      await expect(page).toHaveURL(/.*\/auth\/register/);
    });

    test('should handle form submission with Enter key', async ({ page }) => {
      const loginPage = new LoginPage(page);

      await loginPage.goto('/auth/login');
      await loginPage.waitForFormLoad();
      
      await loginPage.fillCredentials('test@example.com', 'testpassword');
      await loginPage.submitWithEnter();

      // Form should be submitted (loading state or stay filled)
      await page.waitForLoadState('networkidle', { timeout: 5000 });
      const emailValue = await loginPage.emailInput.inputValue();
      expect(emailValue).toBe('test@example.com');
    });

    test('should handle keyboard navigation', async ({ page }) => {
      const loginPage = new LoginPage(page);

      await loginPage.goto('/auth/login');
      await loginPage.waitForFormLoad();

      // Test tab navigation
      await page.keyboard.press('Tab');
      await expect(loginPage.emailInput).toBeFocused();
      
      await page.keyboard.press('Tab');
      await expect(loginPage.passwordInput).toBeFocused();
      
      await page.keyboard.press('Tab');
      await expect(loginPage.submitButton).toBeFocused();
    });
  });

  test.describe('Complete Auth Journey', () => {
    test('should complete registration and login flow', async ({ page }) => {
      test.setTimeout(20000);
      
      const registerPage = new RegisterPage(page);
      const loginPage = new LoginPage(page);
      const testUser = TestDataGenerator.generateUniqueUser();

      // Step 1: Register
      await registerPage.goto('/auth/register');
      await registerPage.register(testUser.username, testUser.email, testUser.password);
      await registerPage.expectRegistrationSuccess();

      // Step 2: Login (page should already be on login after registration)
      await loginPage.waitForFormLoad();
      await loginPage.login(testUser.email, testUser.password);
      await loginPage.expectLoginSuccess();

      // Should be on admin dashboard
      await expect(page).toHaveURL(/.*\/admin/);
    });
  });
});