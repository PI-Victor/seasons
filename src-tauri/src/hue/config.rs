use crate::hue::error::HueError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HueBridgeConfig {
    bridge_ip: String,
    username: Option<String>,
}

impl HueBridgeConfig {
    pub fn new(bridge_ip: String, username: Option<String>) -> Result<Self, HueError> {
        let bridge_ip = bridge_ip.trim().to_string();
        if bridge_ip.is_empty() {
            return Err(HueError::InvalidConfig("bridge IP is required"));
        }

        let username = username
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        Ok(Self {
            bridge_ip,
            username,
        })
    }

    pub fn authenticated(bridge_ip: String, username: String) -> Result<Self, HueError> {
        let config = Self::new(bridge_ip, Some(username))?;
        if config.username.is_none() {
            return Err(HueError::InvalidConfig("username is required"));
        }

        Ok(config)
    }

    pub fn api_base_url(&self) -> String {
        format!("https://{}/api", self.bridge_ip)
    }

    pub fn clip_v2_base_url(&self) -> String {
        format!("https://{}/clip/v2", self.bridge_ip)
    }

    pub fn auth_v1_url(&self) -> String {
        format!("https://{}/auth/v1", self.bridge_ip)
    }

    pub fn authenticated_api_base_url(&self) -> Result<String, HueError> {
        let username = self.username.as_deref().ok_or(HueError::MissingUsername)?;

        Ok(format!("{}/{}", self.api_base_url(), username))
    }

    pub fn application_key(&self) -> Result<&str, HueError> {
        self.username.as_deref().ok_or(HueError::MissingUsername)
    }
}

#[cfg(test)]
mod tests {
    use super::HueBridgeConfig;

    #[test]
    fn trims_and_keeps_optional_username() {
        let config = HueBridgeConfig::new(
            " 192.168.1.2 ".to_string(),
            Some(" user-token ".to_string()),
        )
        .unwrap();

        assert_eq!(config.api_base_url(), "https://192.168.1.2/api");
        assert_eq!(
            config.authenticated_api_base_url().unwrap(),
            "https://192.168.1.2/api/user-token"
        );
    }

    #[test]
    fn rejects_empty_bridge_ip() {
        let error = HueBridgeConfig::new("   ".to_string(), None).unwrap_err();

        assert_eq!(
            error.to_string(),
            "invalid Hue bridge configuration: bridge IP is required"
        );
    }
}
