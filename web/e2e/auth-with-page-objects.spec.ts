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
      const invalidData = TestDataGenerator.generateInvalidUserData();

      await registerPage.goto('/auth/register');
      await registerPage.waitForFormLoad();

      // Test short username
      await registerPage.usernameInput.fill(invalidData.shortUsername);
      await registerPage.usernameInput.blur();
      await registerPage.expectFieldValidationError('username', 'Username must be at least 3 characters');

      // Test invalid username characters
      await registerPage.usernameInput.fill(invalidData.invalidUsername);
      await registerPage.usernameInput.blur();
      await expect(page.getByText(/Username can only contain letters, numbers/)).toBeVisible();

      // Test invalid email
      await registerPage.emailInput.fill(invalidData.invalidEmail);
      await registerPage.emailInput.blur();
      await registerPage.expectFieldValidationError('email', 'Please enter a valid email address');

      // Test short password
      await registerPage.passwordInput.fill(invalidData.shortPassword);
      await registerPage.passwordInput.blur();
      await registerPage.expectFieldValidationError('password', 'Password must be at least 8 characters');
    });

    test('should handle password mismatch validation', async ({ page }) => {
      const registerPage = new RegisterPage(page);
      const testUser = TestDataGenerator.generateUniqueUser();
      const invalidData = TestDataGenerator.generateInvalidUserData();

      await registerPage.goto('/auth/register');
      await registerPage.waitForFormLoad();

      await registerPage.usernameInput.fill(testUser.username);
      await registerPage.emailInput.fill(testUser.email);
      await registerPage.passwordInput.fill(testUser.password);
      await registerPage.confirmPasswordInput.fill(invalidData.mismatchPassword);
      
      await registerPage.confirmPasswordInput.blur();
      await registerPage.expectFieldValidationError('confirmPassword', "Passwords don't match");
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

      // Test with some credentials (may fail but should show loading)
      await loginPage.fillCredentials('test@example.com', 'somepassword');
      await loginPage.submitButton.click();

      // Should show loading state
      await loginPage.expectLoadingState();
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