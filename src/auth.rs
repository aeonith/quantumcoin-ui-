use serde::{Serialize, Deserialize};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{DateTime, Utc, Duration};
use thiserror::Error;
use std::collections::HashMap;
use uuid::Uuid;
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Token expired")]
    TokenExpired,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Permission denied")]
    PermissionDenied,
    #[error("Rate limit exceeded")]
    RateLimited,
    #[error("Account locked")]
    AccountLocked,
    #[error("Hash error: {0}")]
    HashError(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user id
    pub exp: usize,  // expiration
    pub iat: usize,  // issued at
    pub role: UserRole,
    pub permissions: Vec<Permission>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum UserRole {
    Admin,
    User,
    Miner,
    Validator,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Permission {
    ReadWallet,
    WriteWallet,
    SendTransaction,
    MineBlocks,
    AdminFunctions,
    RevStopControl,
    NetworkAccess,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub salt: String,
    pub role: UserRole,
    pub permissions: Vec<Permission>,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub failed_login_attempts: u32,
    pub locked_until: Option<DateTime<Utc>>,
    pub two_factor_enabled: bool,
    pub two_factor_secret: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub two_factor_code: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub user_info: UserInfo,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub permissions: Vec<Permission>,
}

pub struct AuthService {
    secret_key: Vec<u8>,
    users: Arc<RwLock<HashMap<String, User>>>,
    active_sessions: Arc<RwLock<HashMap<String, String>>>, // token -> user_id
    rate_limiter: Arc<RwLock<HashMap<String, (u32, DateTime<Utc>)>>>, // ip -> (attempts, last_attempt)
}

impl AuthService {
    pub fn new(secret_key: &str) -> Self {
        Self {
            secret_key: secret_key.as_bytes().to_vec(),
            users: Arc::new(RwLock::new(HashMap::new())),
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            rate_limiter: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn hash_password(&self, password: &str) -> Result<String, AuthError> {
        hash(password, DEFAULT_COST)
            .map_err(|e| AuthError::HashError(e.to_string()))
    }

    pub fn verify_password(&self, password: &str, hash: &str) -> bool {
        verify(password, hash).unwrap_or(false)
    }

    pub fn create_user(
        &self,
        username: String,
        email: String,
        password: String,
        role: UserRole,
    ) -> Result<String, AuthError> {
        let password_hash = self.hash_password(&password)?;
        let user_id = Uuid::new_v4().to_string();
        
        let permissions = match role {
            UserRole::Admin => vec![
                Permission::ReadWallet,
                Permission::WriteWallet,
                Permission::SendTransaction,
                Permission::MineBlocks,
                Permission::AdminFunctions,
                Permission::RevStopControl,
                Permission::NetworkAccess,
            ],
            UserRole::User => vec![
                Permission::ReadWallet,
                Permission::WriteWallet,
                Permission::SendTransaction,
            ],
            UserRole::Miner => vec![
                Permission::MineBlocks,
                Permission::NetworkAccess,
                Permission::ReadWallet,
            ],
            UserRole::Validator => vec![
                Permission::NetworkAccess,
                Permission::ReadWallet,
                Permission::MineBlocks,
            ],
        };

        let user = User {
            id: user_id.clone(),
            username: username.clone(),
            email,
            password_hash,
            salt: Uuid::new_v4().to_string(),
            role,
            permissions,
            created_at: Utc::now(),
            last_login: None,
            failed_login_attempts: 0,
            locked_until: None,
            two_factor_enabled: false,
            two_factor_secret: None,
        };

        {
            let mut users = self.users.write();
            users.insert(username, user);
        }

        Ok(user_id)
    }

    pub fn authenticate(&self, login: LoginRequest, client_ip: &str) -> Result<AuthResponse, AuthError> {
        // Check rate limiting
        self.check_rate_limit(client_ip)?;

        let user = {
            let users = self.users.read();
            users.get(&login.username).cloned()
                .ok_or(AuthError::InvalidCredentials)?
        };

        // Check if account is locked
        if let Some(locked_until) = user.locked_until {
            if Utc::now() < locked_until {
                return Err(AuthError::AccountLocked);
            }
        }

        // Verify password
        if !self.verify_password(&login.password, &user.password_hash) {
            self.record_failed_login(&login.username, client_ip);
            return Err(AuthError::InvalidCredentials);
        }

        // Verify 2FA if enabled
        if user.two_factor_enabled {
            if let Some(code) = login.two_factor_code {
                if !self.verify_2fa_code(&user, &code)? {
                    return Err(AuthError::InvalidCredentials);
                }
            } else {
                return Err(AuthError::InvalidCredentials);
            }
        }

        // Generate tokens
        let access_token = self.generate_access_token(&user)?;
        let refresh_token = self.generate_refresh_token(&user)?;

        // Update user login info
        self.update_last_login(&user.username);

        // Store active session
        {
            let mut sessions = self.active_sessions.write();
            sessions.insert(access_token.clone(), user.id.clone());
        }

        Ok(AuthResponse {
            access_token,
            refresh_token,
            expires_in: 3600, // 1 hour
            user_info: UserInfo {
                id: user.id,
                username: user.username,
                email: user.email,
                role: user.role,
                permissions: user.permissions,
            },
        })
    }

    fn check_rate_limit(&self, client_ip: &str) -> Result<(), AuthError> {
        let mut rate_limiter = self.rate_limiter.write();
        let now = Utc::now();
        
        if let Some((attempts, last_attempt)) = rate_limiter.get_mut(client_ip) {
            // Reset counter if more than 15 minutes passed
            if now.signed_duration_since(*last_attempt).num_minutes() > 15 {
                *attempts = 0;
            }
            
            if *attempts >= 5 {
                return Err(AuthError::RateLimited);
            }
            
            *attempts += 1;
            *last_attempt = now;
        } else {
            rate_limiter.insert(client_ip.to_string(), (1, now));
        }
        
        Ok(())
    }

    fn record_failed_login(&self, username: &str, _client_ip: &str) {
        let mut users = self.users.write();
        if let Some(user) = users.get_mut(username) {
            user.failed_login_attempts += 1;
            
            // Lock account after 5 failed attempts
            if user.failed_login_attempts >= 5 {
                user.locked_until = Some(Utc::now() + Duration::hours(1));
            }
        }
    }

    fn update_last_login(&self, username: &str) {
        let mut users = self.users.write();
        if let Some(user) = users.get_mut(username) {
            user.last_login = Some(Utc::now());
            user.failed_login_attempts = 0;
            user.locked_until = None;
        }
    }

    fn generate_access_token(&self, user: &User) -> Result<String, AuthError> {
        let expiration = Utc::now() + Duration::hours(1);
        
        let claims = Claims {
            sub: user.id.clone(),
            exp: expiration.timestamp() as usize,
            iat: Utc::now().timestamp() as usize,
            role: user.role.clone(),
            permissions: user.permissions.clone(),
        };

        encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(&self.secret_key),
        )
        .map_err(|_| AuthError::InvalidToken)
    }

    fn generate_refresh_token(&self, user: &User) -> Result<String, AuthError> {
        let expiration = Utc::now() + Duration::days(7);
        
        let claims = Claims {
            sub: user.id.clone(),
            exp: expiration.timestamp() as usize,
            iat: Utc::now().timestamp() as usize,
            role: user.role.clone(),
            permissions: vec![], // Refresh tokens have no permissions
        };

        encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(&self.secret_key),
        )
        .map_err(|_| AuthError::InvalidToken)
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, AuthError> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(&self.secret_key),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|_| AuthError::InvalidToken)?;

        // Check if token is in active sessions
        {
            let sessions = self.active_sessions.read();
            if !sessions.contains_key(token) {
                return Err(AuthError::InvalidToken);
            }
        }

        Ok(token_data.claims)
    }

    pub fn has_permission(&self, claims: &Claims, permission: Permission) -> bool {
        claims.permissions.contains(&permission)
    }

    pub fn logout(&self, token: &str) {
        let mut sessions = self.active_sessions.write();
        sessions.remove(token);
    }

    fn verify_2fa_code(&self, user: &User, code: &str) -> Result<bool, AuthError> {
        if let Some(secret) = &user.two_factor_secret {
            // Use TOTP verification (simplified)
            let auth = otpauth::TOTP::new(secret);
            Ok(auth.verify(code, 30, 1))
        } else {
            Ok(false)
        }
    }

    pub fn cleanup_expired_sessions(&self) {
        // This would run periodically to clean up expired tokens
        let mut sessions = self.active_sessions.write();
        // Implementation would verify each token's expiration
        sessions.retain(|token, _| {
            self.verify_token(token).is_ok()
        });
    }
}

// Middleware for protecting routes
pub fn require_auth(required_permission: Permission) -> impl Fn(&str) -> Result<Claims, AuthError> {
    move |auth_header: &str| {
        let token = auth_header.strip_prefix("Bearer ")
            .ok_or(AuthError::InvalidToken)?;
        
        // This would use the global auth service
        let auth_service = AuthService::new("your-secret-key"); // In real app, get from config
        let claims = auth_service.verify_token(token)?;
        
        if !auth_service.has_permission(&claims, required_permission.clone()) {
            return Err(AuthError::PermissionDenied);
        }
        
        Ok(claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let auth = AuthService::new("test-secret");
        let password = "test123";
        let hash = auth.hash_password(password).unwrap();
        
        assert!(auth.verify_password(password, &hash));
        assert!(!auth.verify_password("wrong", &hash));
    }

    #[test]
    fn test_user_creation() {
        let auth = AuthService::new("test-secret");
        let user_id = auth.create_user(
            "testuser".to_string(),
            "test@example.com".to_string(),
            "password123".to_string(),
            UserRole::User,
        ).unwrap();
        
        assert!(!user_id.is_empty());
    }
}
