import { apiClient } from './base'
import type { ApiResponse } from './base'

/**
 * Authentication API service
 */

export interface LoginCredentials {
  email: string
  password: string
  remember?: boolean
}

export interface RegisterData {
  email: string
  password: string
  confirmPassword: string
  firstName: string
  lastName: string
  acceptTerms: boolean
}

export interface User {
  id: string
  email: string
  firstName: string
  lastName: string
  avatar?: string
  role: 'user' | 'admin' | 'premium'
  emailVerified: boolean
  twoFactorEnabled: boolean
  preferences: UserPreferences
  subscription?: UserSubscription
  createdAt: string
  updatedAt: string
}

export interface UserPreferences {
  theme: 'light' | 'dark' | 'system'
  language: string
  timezone: string
  currency: string
  notifications: {
    email: boolean
    push: boolean
    sms: boolean
    whaleAlerts: boolean
    priceAlerts: boolean
    portfolioUpdates: boolean
  }
  privacy: {
    profileVisible: boolean
    portfolioVisible: boolean
    tradingDataVisible: boolean
  }
}

export interface UserSubscription {
  plan: 'free' | 'pro' | 'whale'
  status: 'active' | 'cancelled' | 'expired' | 'trial'
  startDate: string
  endDate?: string
  features: string[]
  usage: {
    apiCalls: number
    maxApiCalls: number
    portfolios: number
    maxPortfolios: number
  }
}

export interface AuthTokens {
  accessToken: string
  refreshToken: string
  expiresIn: number
  tokenType: 'Bearer'
}

export interface AuthResponse {
  user: User
  tokens: AuthTokens
}

export interface ResetPasswordData {
  email: string
}

export interface ConfirmResetPasswordData {
  token: string
  password: string
  confirmPassword: string
}

export interface ChangePasswordData {
  currentPassword: string
  newPassword: string
  confirmPassword: string
}

export interface TwoFactorSetupData {
  secret: string
  qrCode: string
  backupCodes: string[]
}

export interface TwoFactorVerifyData {
  code: string
}

export interface SessionInfo {
  id: string
  device: string
  browser: string
  ip: string
  location: string
  current: boolean
  lastActive: string
  createdAt: string
}

class AuthService {
  private readonly basePath = '/auth'

  /**
   * User registration
   */
  async register(data: RegisterData): Promise<ApiResponse<AuthResponse>> {
    return apiClient.post(`${this.basePath}/register`, data, {
      skipAuth: true
    })
  }

  /**
   * User login
   */
  async login(credentials: LoginCredentials): Promise<ApiResponse<AuthResponse>> {
    return apiClient.post(`${this.basePath}/login`, credentials, {
      skipAuth: true
    })
  }

  /**
   * User logout
   */
  async logout(): Promise<ApiResponse<void>> {
    return apiClient.post(`${this.basePath}/logout`)
  }

  /**
   * Logout from all devices
   */
  async logoutAll(): Promise<ApiResponse<void>> {
    return apiClient.post(`${this.basePath}/logout-all`)
  }

  /**
   * Refresh access token
   */
  async refreshToken(refreshToken: string): Promise<ApiResponse<AuthTokens>> {
    return apiClient.post(`${this.basePath}/refresh`, {
      refreshToken
    }, {
      skipAuth: true
    })
  }

  /**
   * Get current user profile
   */
  async getProfile(): Promise<ApiResponse<User>> {
    return apiClient.get(`${this.basePath}/profile`)
  }

  /**
   * Update user profile
   */
  async updateProfile(data: Partial<User>): Promise<ApiResponse<User>> {
    return apiClient.patch(`${this.basePath}/profile`, data)
  }

  /**
   * Change password
   */
  async changePassword(data: ChangePasswordData): Promise<ApiResponse<void>> {
    return apiClient.post(`${this.basePath}/change-password`, data)
  }

  /**
   * Request password reset
   */
  async requestPasswordReset(data: ResetPasswordData): Promise<ApiResponse<void>> {
    return apiClient.post(`${this.basePath}/reset-password`, data, {
      skipAuth: true
    })
  }

  /**
   * Confirm password reset
   */
  async confirmPasswordReset(data: ConfirmResetPasswordData): Promise<ApiResponse<void>> {
    return apiClient.post(`${this.basePath}/reset-password/confirm`, data, {
      skipAuth: true
    })
  }

  /**
   * Verify email address
   */
  async verifyEmail(token: string): Promise<ApiResponse<void>> {
    return apiClient.post(`${this.basePath}/verify-email`, { token }, {
      skipAuth: true
    })
  }

  /**
   * Resend email verification
   */
  async resendEmailVerification(): Promise<ApiResponse<void>> {
    return apiClient.post(`${this.basePath}/verify-email/resend`)
  }

  /**
   * Setup two-factor authentication
   */
  async setupTwoFactor(): Promise<ApiResponse<TwoFactorSetupData>> {
    return apiClient.post(`${this.basePath}/2fa/setup`)
  }

  /**
   * Verify and enable two-factor authentication
   */
  async enableTwoFactor(data: TwoFactorVerifyData): Promise<ApiResponse<{ backupCodes: string[] }>> {
    return apiClient.post(`${this.basePath}/2fa/enable`, data)
  }

  /**
   * Disable two-factor authentication
   */
  async disableTwoFactor(data: TwoFactorVerifyData): Promise<ApiResponse<void>> {
    return apiClient.post(`${this.basePath}/2fa/disable`, data)
  }

  /**
   * Generate new backup codes
   */
  async generateBackupCodes(): Promise<ApiResponse<{ backupCodes: string[] }>> {
    return apiClient.post(`${this.basePath}/2fa/backup-codes`)
  }

  /**
   * Get user preferences
   */
  async getPreferences(): Promise<ApiResponse<UserPreferences>> {
    return apiClient.get(`${this.basePath}/preferences`)
  }

  /**
   * Update user preferences
   */
  async updatePreferences(preferences: Partial<UserPreferences>): Promise<ApiResponse<UserPreferences>> {
    return apiClient.patch(`${this.basePath}/preferences`, preferences)
  }

  /**
   * Get active sessions
   */
  async getSessions(): Promise<ApiResponse<SessionInfo[]>> {
    return apiClient.get(`${this.basePath}/sessions`)
  }

  /**
   * Revoke a specific session
   */
  async revokeSession(sessionId: string): Promise<ApiResponse<void>> {
    return apiClient.delete(`${this.basePath}/sessions/${sessionId}`)
  }

  /**
   * Upload avatar image
   */
  async uploadAvatar(file: File): Promise<ApiResponse<{ avatarUrl: string }>> {
    return apiClient.upload(`${this.basePath}/avatar`, file)
  }

  /**
   * Delete avatar
   */
  async deleteAvatar(): Promise<ApiResponse<void>> {
    return apiClient.delete(`${this.basePath}/avatar`)
  }

  /**
   * Delete user account
   */
  async deleteAccount(password: string): Promise<ApiResponse<void>> {
    return apiClient.post(`${this.basePath}/delete-account`, { password })
  }

  /**
   * Export user data (GDPR compliance)
   */
  async exportData(): Promise<ApiResponse<{ downloadUrl: string }>> {
    return apiClient.post(`${this.basePath}/export-data`)
  }

  /**
   * Get subscription info
   */
  async getSubscription(): Promise<ApiResponse<UserSubscription>> {
    return apiClient.get(`${this.basePath}/subscription`)
  }

  /**
   * Update subscription plan
   */
  async updateSubscription(plan: string): Promise<ApiResponse<UserSubscription>> {
    return apiClient.post(`${this.basePath}/subscription`, { plan })
  }

  /**
   * Cancel subscription
   */
  async cancelSubscription(): Promise<ApiResponse<void>> {
    return apiClient.delete(`${this.basePath}/subscription`)
  }

  /**
   * Get API usage statistics
   */
  async getUsageStats(): Promise<ApiResponse<{
    current: {
      apiCalls: number
      maxApiCalls: number
      portfolios: number
      maxPortfolios: number
      storage: number
      maxStorage: number
    }
    history: Array<{
      date: string
      apiCalls: number
      requests: number
    }>
  }>> {
    return apiClient.get(`${this.basePath}/usage`)
  }

  /**
   * Check if email is available
   */
  async checkEmailAvailability(email: string): Promise<ApiResponse<{ available: boolean }>> {
    return apiClient.get(`${this.basePath}/check-email`, {
      params: { email },
      skipAuth: true
    })
  }

  /**
   * Social login (OAuth)
   */
  async socialLogin(provider: string, code: string, state?: string): Promise<ApiResponse<AuthResponse>> {
    return apiClient.post(`${this.basePath}/social/${provider}`, {
      code,
      state
    }, {
      skipAuth: true
    })
  }

  /**
   * Link social account
   */
  async linkSocialAccount(provider: string, code: string): Promise<ApiResponse<void>> {
    return apiClient.post(`${this.basePath}/social/${provider}/link`, {
      code
    })
  }

  /**
   * Unlink social account
   */
  async unlinkSocialAccount(provider: string): Promise<ApiResponse<void>> {
    return apiClient.delete(`${this.basePath}/social/${provider}/unlink`)
  }

  /**
   * Get linked social accounts
   */
  async getLinkedAccounts(): Promise<ApiResponse<Array<{
    provider: string
    email: string
    linkedAt: string
  }>>> {
    return apiClient.get(`${this.basePath}/social/linked`)
  }

  /**
   * Validate session token
   */
  async validateToken(): Promise<ApiResponse<{ valid: boolean, user?: User }>> {
    return apiClient.get(`${this.basePath}/validate`)
  }

  /**
   * Get account security status
   */
  async getSecurityStatus(): Promise<ApiResponse<{
    emailVerified: boolean
    twoFactorEnabled: boolean
    strongPassword: boolean
    recentLogin: boolean
    suspiciousActivity: boolean
    score: number
    recommendations: string[]
  }>> {
    return apiClient.get(`${this.basePath}/security-status`)
  }

  /**
   * Enable/disable account security features
   */
  async updateSecuritySettings(settings: {
    loginNotifications?: boolean
    deviceTracking?: boolean
    ipWhitelist?: string[]
    sessionTimeout?: number
  }): Promise<ApiResponse<void>> {
    return apiClient.patch(`${this.basePath}/security-settings`, settings)
  }

  /**
   * Report suspicious activity
   */
  async reportSuspiciousActivity(data: {
    type: string
    description: string
    timestamp: string
  }): Promise<ApiResponse<void>> {
    return apiClient.post(`${this.basePath}/report-suspicious`, data)
  }
}

// Export singleton instance
export const authService = new AuthService()

// Export class for testing
export { AuthService }