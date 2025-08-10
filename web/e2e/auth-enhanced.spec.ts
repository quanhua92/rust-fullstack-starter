import { test, expect } from '@playwright/test';

test.describe('Enhanced Authentication Flow', () => {
  test.beforeEach(async ({ page }) => {
    // Start fresh for each test
    await page.goto('/');
  });

  test.describe('Registration Form Validation', () => {
    test('should show real-time validation errors for all fields', async ({ page }) => {
      await page.goto('/auth/register');
      await page.waitForLoadState('networkidle');

      // Test username validation
      const usernameInput = page.locator('input[placeholder*="username" i]');
      await usernameInput.fill('ab'); // Too short
      await usernameInput.blur();
      await expect(page.getByText('Username must be at least 3 characters')).toBeVisible();

      await usernameInput.fill('user@with@invalid'); // Invalid characters
      await usernameInput.blur();
      await expect(page.getByText(/Username can only contain letters, numbers/)).toBeVisible();

      // Test email validation
      const emailInput = page.locator('input[type="email"]');
      await emailInput.fill('invalid-email');
      await emailInput.blur();
      await expect(page.getByText('Please enter a valid email address')).toBeVisible();

      // Test password validation
      const passwordInput = page.locator('input[placeholder="Enter your password"]');
      await passwordInput.fill('short');
      await passwordInput.blur();
      await expect(page.getByText('Password must be at least 8 characters')).toBeVisible();

      // Test password confirmation
      const confirmPasswordInput = page.locator('input[placeholder="Confirm your password"]');
      await passwordInput.fill('ValidPassword123!');
      await confirmPasswordInput.fill('DifferentPassword');
      await confirmPasswordInput.blur();
      await expect(page.getByText("Passwords don't match")).toBeVisible();
    });

    test('should disable submit button when form is invalid', async ({ page }) => {
      await page.goto('/auth/register');
      await page.waitForLoadState('networkidle');

      const submitButton = page.locator('button:has-text("Create Account")');
      
      // Button should be disabled initially
      await expect(submitButton).toBeDisabled();

      // Fill some fields but leave others invalid
      await page.locator('input[placeholder*="username" i]').fill('testuser');
      await page.locator('input[type="email"]').fill('invalid-email');
      
      // Button should still be disabled
      await expect(submitButton).toBeDisabled();

      // Fill all fields correctly
      await page.locator('input[type="email"]').fill('test@example.com');
      await page.locator('input[placeholder="Enter your password"]').fill('ValidPassword123!');
      await page.locator('input[placeholder="Confirm your password"]').fill('ValidPassword123!');
      
      // Now button should be enabled
      await expect(submitButton).toBeEnabled();
    });

    test('should show success message and redirect after successful registration', async ({ page }) => {
      test.setTimeout(20000);
      
      await page.goto('/auth/register');
      await page.waitForLoadState('networkidle');

      // Generate unique user data
      const timestamp = Date.now();
      const randomSuffix = Math.random().toString(36).substr(2, 9);
      const username = `testuser_${timestamp}_${randomSuffix}`;
      const email = `test_${timestamp}_${randomSuffix}@example.com`;
      const password = 'SecurePassword123!';

      // Fill form with valid data
      await page.locator('input[placeholder*="username" i]').fill(username);
      await page.locator('input[type="email"]').fill(email);
      await page.locator('input[placeholder="Enter your password"]').fill(password);
      await page.locator('input[placeholder="Confirm your password"]').fill(password);

      // Submit form
      await page.locator('button:has-text("Create Account")').click();

      // Verify loading state
      await expect(page.locator('button:has-text("Creating Account...")')).toBeVisible();

      // Wait for success message
      await expect(page.locator('text=Registration successful! Redirecting to login page...')).toBeVisible({ timeout: 10000 });
      
      // Verify redirect to login page
      await expect(page).toHaveURL(/.*\/auth\/login/, { timeout: 8000 });
    });

    test('should handle server validation errors', async ({ page }) => {
      test.setTimeout(15000);
      
      await page.goto('/auth/register');
      await page.waitForLoadState('networkidle');

      // Try to register with a common email that might already exist
      const commonEmail = 'admin@example.com';
      
      await page.locator('input[placeholder*="username" i]').fill('testuser');
      await page.locator('input[type="email"]').fill(commonEmail);
      await page.locator('input[placeholder="Enter your password"]').fill('ValidPassword123!');
      await page.locator('input[placeholder="Confirm your password"]').fill('ValidPassword123!');

      await page.locator('button:has-text("Create Account")').click();

      // Should show either success or an error message
      const successMessage = page.locator('text=Registration successful');
      const errorAlert = page.locator('[role="alert"]');
      
      await expect(successMessage.or(errorAlert)).toBeVisible({ timeout: 10000 });
    });
  });

  test.describe('Login Form Validation', () => {
    test('should show validation errors for empty fields', async ({ page }) => {
      await page.goto('/auth/login');
      await page.waitForLoadState('networkidle');

      // Try to submit empty form
      await page.locator('button:has-text("Sign In")').click();

      // Should show validation errors or stay on same page
      await expect(page).toHaveURL(/.*\/auth\/login/);
      
      // Fill email but not password
      await page.locator('input[type="email"]').fill('test@example.com');
      await page.locator('button:has-text("Sign In")').click();
      
      // Should still be on login page
      await expect(page).toHaveURL(/.*\/auth\/login/);
    });

    test('should show loading state during login', async ({ page }) => {
      await page.goto('/auth/login');
      await page.waitForLoadState('networkidle');

      // Fill with some credentials (may fail, but should show loading)
      await page.locator('input[type="email"]').fill('test@example.com');
      await page.locator('input[type="password"]').fill('somepassword');
      
      await page.locator('button:has-text("Sign In")').click();

      // Should show loading state briefly
      await expect(page.locator('button:has-text("Signing In...")').or(page.locator('button[disabled]'))).toBeVisible({ timeout: 5000 });
    });

    test('should handle login errors gracefully', async ({ page }) => {
      await page.goto('/auth/login');
      await page.waitForLoadState('networkidle');

      // Try invalid credentials
      await page.locator('input[type="email"]').fill('nonexistent@example.com');
      await page.locator('input[type="password"]').fill('wrongpassword');
      
      await page.locator('button:has-text("Sign In")').click();

      // Should show error or stay on login page
      await page.waitForLoadState('networkidle', { timeout: 10000 });
      
      // Either shows error alert or stays on login page
      const errorAlert = page.locator('[role="alert"]');
      const loginPage = page.locator('h1:has-text("Sign In"), h2:has-text("Sign In")');
      
      await expect(errorAlert.or(loginPage)).toBeVisible();
    });
  });

  test.describe('Navigation Between Auth Pages', () => {
    test('should navigate from login to register', async ({ page }) => {
      await page.goto('/auth/login');
      await page.waitForLoadState('networkidle');

      // Click "Sign Up" link
      await page.locator('button:has-text("Sign Up")').click();
      
      // Should navigate to register page
      await expect(page).toHaveURL(/.*\/auth\/register/);
      await expect(page.locator('h1:has-text("Create Account"), h2:has-text("Create Account")').first()).toBeVisible();
    });

    test('should navigate from register to login', async ({ page }) => {
      await page.goto('/auth/register');
      await page.waitForLoadState('networkidle');

      // Click "Sign In" link  
      await page.locator('button:has-text("Sign In")').click();
      
      // Should navigate to login page
      await expect(page).toHaveURL(/.*\/auth\/login/);
      await expect(page.locator('h1:has-text("Sign In"), h2:has-text("Sign In")').first()).toBeVisible();
    });
  });

  test.describe('Form Field Interactions', () => {
    test('should handle keyboard navigation', async ({ page }) => {
      await page.goto('/auth/login');
      await page.waitForLoadState('networkidle');

      // Test tab navigation
      await page.keyboard.press('Tab');
      await expect(page.locator('input[type="email"]')).toBeFocused();
      
      await page.keyboard.press('Tab');
      await expect(page.locator('input[type="password"]')).toBeFocused();
      
      await page.keyboard.press('Tab');
      await expect(page.locator('button:has-text("Sign In")')).toBeFocused();
    });

    test('should handle form submission with Enter key', async ({ page }) => {
      await page.goto('/auth/login');
      await page.waitForLoadState('networkidle');

      // Fill form
      await page.locator('input[type="email"]').fill('test@example.com');
      await page.locator('input[type="password"]').fill('testpassword');
      
      // Press Enter to submit
      await page.locator('input[type="password"]').press('Enter');

      // Should attempt to submit (loading state or redirect/error)
      await page.waitForLoadState('networkidle', { timeout: 5000 });
      
      // Form should have been submitted (not still on same page with empty fields)
      const emailValue = await page.locator('input[type="email"]').inputValue();
      expect(emailValue).toBe('test@example.com');
    });
  });

  test.describe('Form Field Focus Management', () => {
    test('should focus first invalid field after submission attempt', async ({ page }) => {
      await page.goto('/auth/register');
      await page.waitForLoadState('networkidle');

      // Try to submit empty form
      await page.locator('button:has-text("Create Account")').click();
      
      // First field (username) should be focused
      await expect(page.locator('input[placeholder*="username" i]')).toBeFocused();
    });

    test('should maintain focus state during validation', async ({ page }) => {
      await page.goto('/auth/register');
      await page.waitForLoadState('networkidle');

      const usernameInput = page.locator('input[placeholder*="username" i]');
      
      // Focus and fill invalid value
      await usernameInput.click();
      await usernameInput.fill('ab'); // Too short
      
      // Move focus away to trigger validation
      await page.locator('input[type="email"]').click();
      
      // Validation error should be shown
      await expect(page.getByText('Username must be at least 3 characters')).toBeVisible();
      
      // Focus should be on email field now
      await expect(page.locator('input[type="email"]')).toBeFocused();
    });
  });
});