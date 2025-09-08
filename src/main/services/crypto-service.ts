import * as crypto from 'crypto'
import { ConsoleLogger } from '../utils/logger'

/**
 * Service for handling all cryptographic operations using AES-256-GCM encryption
 * Provides zero-knowledge encryption capabilities for sensitive user data
 */
export class CryptoService {
  private readonly logger = new ConsoleLogger('CryptoService')

  // Constants for encryption
  private readonly ALGORITHM = 'aes-256-gcm'
  private readonly SALT_LENGTH = 32
  private readonly IV_LENGTH = 12
  private readonly TAG_LENGTH = 16
  private readonly KEY_DERIVATION_ITERATIONS = 100000

  /**
   * Derive encryption key from master password using PBKDF2
   * @param masterPassword - The user's master password
   * @param salt - Optional salt (if not provided, a new one will be generated)
   * @returns Object containing derived key and salt
   */
  deriveKey(masterPassword: string, salt?: Buffer): { key: Buffer; salt: Buffer } {
    try {
      const actualSalt = salt || crypto.randomBytes(this.SALT_LENGTH)
      const key = crypto.pbkdf2Sync(
        masterPassword,
        actualSalt,
        this.KEY_DERIVATION_ITERATIONS,
        32,
        'sha256'
      )

      return { key, salt: actualSalt }
    } catch (error) {
      this.logger.error('Failed to derive key:', error as Error)
      throw new Error('Key derivation failed')
    }
  }

  /**
   * Encrypt plaintext using AES-256-GCM
   * @param plaintext - The data to encrypt
   * @param derivedKey - The derived encryption key
   * @returns Encrypted string in format: salt:iv:authTag:encryptedData (base64 encoded)
   */
  encrypt(plaintext: string, derivedKey: Buffer): string {
    try {
      // Generate random IV for this encryption
      const iv = crypto.randomBytes(this.IV_LENGTH)

      // Create cipher
      const cipher = crypto.createCipheriv(this.ALGORITHM, derivedKey, iv)
      cipher.setAAD(Buffer.alloc(0)) // No additional authenticated data

      // Encrypt the plaintext
      let encrypted = cipher.update(plaintext, 'utf8', 'base64')
      encrypted += cipher.final('base64')

      // Get the authentication tag
      const authTag = cipher.getAuthTag()

      // Generate a salt for this encryption (for additional security)
      const salt = crypto.randomBytes(this.SALT_LENGTH)

      // Combine all components: salt:iv:authTag:encryptedData
      const result = [
        salt.toString('base64'),
        iv.toString('base64'),
        authTag.toString('base64'),
        encrypted
      ].join(':')

      return result
    } catch (error) {
      this.logger.error('Encryption failed:', error as Error)
      throw new Error('Encryption failed')
    }
  }

  /**
   * Decrypt encrypted text using AES-256-GCM
   * @param encryptedText - The encrypted string in format: salt:iv:authTag:encryptedData
   * @param derivedKey - The derived encryption key
   * @returns Decrypted plaintext
   * @throws Error if decryption fails (wrong password, corrupted data, etc.)
   */
  decrypt(encryptedText: string, derivedKey: Buffer): string {
    try {
      // Parse the encrypted text components
      const parts = encryptedText.split(':')
      if (parts.length !== 4) {
        throw new Error('Invalid encrypted data format')
      }

      const [saltStr, ivStr, authTagStr, encryptedData] = parts

      // Convert from base64
      const salt = Buffer.from(saltStr, 'base64')
      const iv = Buffer.from(ivStr, 'base64')
      const authTag = Buffer.from(authTagStr, 'base64')

      // Validate buffer lengths
      if (salt.length !== this.SALT_LENGTH) {
        throw new Error('Invalid salt length')
      }
      if (iv.length !== this.IV_LENGTH) {
        throw new Error('Invalid IV length')
      }
      if (authTag.length !== this.TAG_LENGTH) {
        throw new Error('Invalid auth tag length')
      }

      // Create decipher
      const decipher = crypto.createDecipheriv(this.ALGORITHM, derivedKey, iv)
      decipher.setAAD(Buffer.alloc(0)) // No additional authenticated data
      decipher.setAuthTag(authTag)

      // Decrypt the data
      let decrypted = decipher.update(encryptedData, 'base64', 'utf8')
      decrypted += decipher.final('utf8')

      return decrypted
    } catch (error) {
      this.logger.error('Decryption failed:', error as Error)
      // Don't expose internal error details for security
      throw new Error('Decryption failed - invalid password or corrupted data')
    }
  }

  /**
   * Generate a secure random string for passwords
   * @param length - Length of the generated string
   * @returns Random string
   */
  generateSecureRandom(length: number = 32): string {
    return crypto.randomBytes(length).toString('base64')
  }

  /**
   * Hash a string using SHA-256
   * @param input - String to hash
   * @returns SHA-256 hash in hex format
   */
  hash(input: string): string {
    return crypto.createHash('sha256').update(input).digest('hex')
  }

  /**
   * Verify password against stored hash
   * @param password - Password to verify
   * @param storedHash - Previously stored password hash
   * @returns True if password matches
   */
  verifyPassword(password: string, storedHash: string): boolean {
    const passwordHash = this.hash(password)
    return crypto.timingSafeEqual(Buffer.from(passwordHash), Buffer.from(storedHash))
  }

  /**
   * Create a verification hash from master password for storage
   * This allows us to verify the master password without storing it
   * @param masterPassword - The master password
   * @param salt - Salt used for key derivation
   * @returns Verification hash
   */
  createVerificationHash(masterPassword: string, salt: Buffer): string {
    // Create a different salt for verification to avoid using the same salt as encryption
    const verificationSalt = crypto.createHash('sha256').update(salt).digest()
    const verificationKey = crypto.pbkdf2Sync(
      masterPassword,
      verificationSalt,
      this.KEY_DERIVATION_ITERATIONS,
      32,
      'sha256'
    )
    return verificationKey.toString('base64')
  }

  /**
   * Verify master password using verification hash
   * @param masterPassword - Password to verify
   * @param storedVerificationHash - Stored verification hash
   * @param salt - Original salt used for key derivation
   * @returns True if password is correct
   */
  verifyMasterPassword(
    masterPassword: string,
    storedVerificationHash: string,
    salt: Buffer
  ): boolean {
    try {
      const computedHash = this.createVerificationHash(masterPassword, salt)
      return crypto.timingSafeEqual(
        Buffer.from(computedHash, 'base64'),
        Buffer.from(storedVerificationHash, 'base64')
      )
    } catch (error) {
      this.logger.error('Master password verification failed:', error as Error)
      return false
    }
  }
}
