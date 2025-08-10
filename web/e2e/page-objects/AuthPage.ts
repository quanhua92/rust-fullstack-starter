import { type Locator, type Page } from '@playwright/test';

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
    this.passwordInput = page.locator('input[type="password"]');
    this.errorAlert = page.locator('[role="alert"]');
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
    // Wait for button to show loading text or become disabled
    const loadingButton = this.page.locator('button[disabled], button:has-text("ing...")');
    await loadingButton.waitFor({ state: 'visible', timeout: 5000 });
  }
}

export class LoginPage extends AuthPage {
  readonly submitButton: Locator;
  readonly signUpLink: Locator;

  constructor(page: Page) {
    super(page);
    this.submitButton = page.locator('button:has-text("Sign In")');
    this.signUpLink = page.locator('button:has-text("Sign Up")');
    this.loadingButton = page.locator('button:has-text("Signing In...")');
  }

  async login(email: string, password: string) {
    await this.fillCredentials(email, password);
    await this.submitButton.click();
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
  readonly submitButton: Locator;
  readonly signInLink: Locator;
  readonly successMessage: Locator;

  constructor(page: Page) {
    super(page);
    this.usernameInput = page.locator('input[placeholder*="username" i]');
    this.confirmPasswordInput = page.locator('input[placeholder="Confirm your password"]');
    this.submitButton = page.locator('button:has-text("Create Account")');
    this.signInLink = page.locator('button:has-text("Sign In")');
    this.loadingButton = page.locator('button:has-text("Creating Account...")');
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
    await this.submitButton.click();
  }

  async goToLogin() {
    await this.signInLink.click();
  }

  async expectRegistrationSuccess() {
    await this.successMessage.waitFor({ state: 'visible', timeout: 10000 });
    // Should redirect to login page
    await this.page.waitForURL(/.*\/auth\/login/, { timeout: 8000 });
  }

  async expectFieldValidationError(fieldType: 'username' | 'email' | 'password' | 'confirmPassword', expectedError: string) {
    const errorLocator = this.page.locator(`text=${expectedError}`);
    await errorLocator.waitFor({ state: 'visible' });
  }

  async expectSubmitButtonDisabled() {
    await this.page.waitForFunction(() => {
      const button = document.querySelector('button:has-text("Create Account")') as HTMLButtonElement;
      return button?.disabled === true;
    });
  }

  async expectSubmitButtonEnabled() {
    await this.page.waitForFunction(() => {
      const button = document.querySelector('button:has-text("Create Account")') as HTMLButtonElement;
      return button?.disabled === false;
    });
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