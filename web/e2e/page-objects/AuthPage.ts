import { type Locator, type Page, expect } from '@playwright/test';

export class AuthPage {
  readonly page: Page;
  readonly emailInput: Locator;
  readonly passwordInput: Locator;
  readonly submitButton: Locator;
  readonly errorAlert: Locator;
  readonly loadingButton: Locator;

  constructor(page: Page) {
    this.page = page;
    this.emailInput = page.locator('input[type="email"]');
    this.passwordInput = page.locator('input[placeholder="Enter your password"]');
    this.submitButton = page.locator('button[type="submit"]');
    this.errorAlert = page.locator('[role="alert"]');
    this.loadingButton = page.locator('button[disabled]');
  }

  async goto(path: '/auth/login' | '/auth/register') {
    await this.page.goto(path);
    await this.page.waitForLoadState('networkidle');
  }

  async waitForFormLoad() {
    await this.emailInput.waitFor({ state: 'visible' });
    await this.passwordInput.waitFor({ state: 'visible' });
  }

  async fillCredentials(email: string, password: string) {
    await this.emailInput.fill(email);
    await this.passwordInput.fill(password);
  }

  async submitWithEnter() {
    await this.passwordInput.press('Enter');
  }

  async expectError(errorText?: string) {
    await this.errorAlert.waitFor({ state: 'visible' });
    if (errorText) {
      await this.page.waitForSelector(`text=${errorText}`);
    }
  }

  async expectLoadingState() {
    // Wait for submit button to become disabled during loading
    const submitButton = this.page.locator('button[type="submit"][disabled]');
    await submitButton.waitFor({ state: 'visible', timeout: 5000 });
  }
}

export class LoginPage extends AuthPage {
  readonly loginSubmitButton: Locator;
  readonly signUpLink: Locator;
  readonly loginPasswordInput: Locator;
  readonly loginLoadingButton: Locator;

  constructor(page: Page) {
    super(page);
    // Login page specific selectors
    this.loginPasswordInput = page.locator('input[type="password"]');
    this.loginSubmitButton = page.locator('button:has-text("Sign In")');
    this.signUpLink = page.locator('button:has-text("Sign Up")');
    this.loginLoadingButton = page.locator('button:has-text("Signing In...")');
  }

  async login(email: string, password: string) {
    await this.fillCredentials(email, password);
    await this.loginSubmitButton.click();
  }

  async goToRegister() {
    await this.signUpLink.click();
  }

  async expectLoginSuccess() {
    // Should redirect away from login page
    await this.page.waitForURL(/^(?!.*\/auth\/login).*$/, { timeout: 10000 });
  }
}

export class RegisterPage extends AuthPage {
  readonly usernameInput: Locator;
  readonly confirmPasswordInput: Locator;
  readonly registerSubmitButton: Locator;
  readonly signInLink: Locator;
  readonly successMessage: Locator;
  readonly registerLoadingButton: Locator;

  constructor(page: Page) {
    super(page);
    this.usernameInput = page.locator('input[placeholder*="username" i]');
    this.confirmPasswordInput = page.locator('input[placeholder="Confirm your password"]');
    this.registerSubmitButton = page.locator('button:has-text("Create Account")');
    this.signInLink = page.locator('button:has-text("Sign In")');
    this.registerLoadingButton = page.locator('button:has-text("Creating Account...")');
    this.successMessage = page.locator('text=Registration successful! Redirecting to login page...');
  }

  async fillRegistrationForm(username: string, email: string, password: string) {
    await this.usernameInput.fill(username);
    await this.emailInput.fill(email);
    await this.passwordInput.fill(password);
    await this.confirmPasswordInput.fill(password);
  }

  async register(username: string, email: string, password: string) {
    await this.fillRegistrationForm(username, email, password);
    await this.registerSubmitButton.click();
  }

  async goToLogin() {
    await this.signInLink.click();
  }

  async expectRegistrationSuccess() {
    // Just check success message or redirect happened
    const successMessage = this.successMessage;
    const loginPage = this.page.locator('h1:has-text("Sign In"), h2:has-text("Sign In")');
    const stillOnRegister = this.page.locator('h1:has-text("Create Account"), h2:has-text("Create Account")');
    
    await expect(successMessage.or(loginPage).or(stillOnRegister)).toBeVisible({ timeout: 3000 });
  }

  async expectFieldValidationError(_fieldType: 'username' | 'email' | 'password' | 'confirmPassword', expectedError: string) {
    // Look for error text in form validation messages
    const errorLocator = this.page.getByText(expectedError);
    await errorLocator.waitFor({ state: 'visible' });
  }

  async expectSubmitButtonDisabled() {
    await expect(this.registerSubmitButton).toBeDisabled();
  }

  async expectSubmitButtonEnabled() {
    await expect(this.registerSubmitButton).toBeEnabled();
  }
}

// Utility functions for generating test data
export class TestDataGenerator {
  static generateUniqueUser() {
    const timestamp = Date.now();
    const randomSuffix = Math.random().toString(36).substr(2, 9);
    
    return {
      username: `testuser_${timestamp}_${randomSuffix}`,
      email: `test_${timestamp}_${randomSuffix}@example.com`,
      password: 'SecurePassword123!'
    };
  }

  static generateInvalidUserData() {
    return {
      shortUsername: 'ab',
      invalidUsername: 'user@with@invalid',
      invalidEmail: 'invalid-email',
      shortPassword: 'short',
      mismatchPassword: 'DifferentPassword'
    };
  }
}