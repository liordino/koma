//! Central outbound-fetch module (network baseline: SSRF guard + dual-plane rate limiter).
//!
//! Every outbound request goes through this module. A request to a host outside the adapter's
//! allowlist is rejected BEFORE any connection is opened. The rate limiter exposes two planes
//! (decision 18: At-Home compliance) so API-plane and image-plane budgets are enforced per
//! source adapter, as values — a refused request is an error, a throttled one is a wait.

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::de::DeserializeOwned;

/// A rate-limit plane. MangaDex documents separate limits for API and At-Home image delivery.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Plane {
    Api,
    AtHome,
}

/// Token-window limiter: at most `budget` requests per `window`, call sites wait their turn.
#[derive(Debug)]
struct PlaneLimiter {
    budget: usize,
    window: Duration,
    granted: VecDeque<Instant>,
}

impl PlaneLimiter {
    fn new(budget: usize, window: Duration) -> Self {
        Self { budget, window, granted: VecDeque::new() }
    }

    fn sweep(&mut self) {
        let now = Instant::now();
        while let Some(&front) = self.granted.front() {
            if now.duration_since(front) >= self.window {
                self.granted.pop_front();
            } else {
                break;
            }
        }
    }

    /// Take a slot if the window has budget; otherwise report how long to wait.
    /// Designed for call-site locking: try under the lock, sleep outside it.
    pub fn try_grant(&mut self) -> Option<Duration> {
        self.sweep();
        if self.granted.len() < self.budget {
            self.granted.push_back(Instant::now());
            None
        } else {
            let oldest = *self.granted.front().expect("non-empty after sweep");
            Some(self.window.saturating_sub(oldest.elapsed()).max(Duration::from_millis(10)))
        }
    }

}

#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    #[error("host '{host}' is not in the adapter allowlist (SSRF guard)")]
    Disallowed { host: String },
    #[error("request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("unexpected response from {url}: status {status}")]
    Unexpected { url: String, status: u16 },
}

/// Is `host` inside the allowlist? Exact domain or subdomain thereof (suffix with boundary),
/// never a substring match — "evil-allowed.com" is not "allowed.com".
fn host_allowed(host: &str, allowlist: &[String]) -> bool {
    let host = host.to_ascii_lowercase();
    allowlist.iter().any(|domain| {
        let domain = domain.to_ascii_lowercase();
        host == domain || host.ends_with(&format!(".{domain}"))
    })
}

/// The only way the app talks to the outside world.
#[derive(Debug)]
pub struct OutboundClient {
    allowlist: Vec<String>,
    http: reqwest::Client,
    limiters: std::sync::Mutex<[PlaneLimiter; 2]>,
}

impl OutboundClient {
    /// Budgets are per plane, per window.
    pub fn new(
        allowlist: Vec<String>,
        api_budget: usize,
        at_home_budget: usize,
        window: Duration,
    ) -> Result<Self, reqwest::Error> {
        Ok(Self {
            allowlist,
            http: reqwest::Client::builder()
                .user_agent("koma/0.1 (single-operator)")
                .connect_timeout(Duration::from_secs(10))
                .build()?,
            limiters: std::sync::Mutex::new([
                PlaneLimiter::new(api_budget, window),
                PlaneLimiter::new(at_home_budget, window),
            ]),
        })
    }

    /// Throttle without holding the mutex across an await: try under the lock,
    /// read the wait hint, sleep outside it.
    async fn throttle(&self, plane: Plane) {
        let idx = Self::plane_index(plane);
        loop {
            let wait = {
                let mut limiters = self.limiters.lock().expect("limiter mutex");
                limiters[idx].try_grant()
            };
            match wait {
                None => return,
                Some(d) => tokio::time::sleep(d).await,
            }
        }
    }

    fn plane_index(plane: Plane) -> usize {
        match plane {
            Plane::Api => 0,
            Plane::AtHome => 1,
        }
    }

    /// Reject disallowed hosts BEFORE any connection; acquire a budget slot, then GET.
    pub async fn get_json<T: DeserializeOwned>(
        &self,
        url: &str,
        plane: Plane,
    ) -> Result<T, FetchError> {
        let parsed = reqwest::Url::parse(url)
            .map_err(|_e| FetchError::Unexpected { url: url.to_string(), status: 0 })?;
        let host = parsed
            .host_str()
            .ok_or_else(|| FetchError::Unexpected { url: url.to_string(), status: 0 })?
            .to_string();
        if !host_allowed(&host, &self.allowlist) {
            return Err(FetchError::Disallowed { host });
        }

        self.throttle(plane).await;

        let resp = self.http.get(url).send().await?;
        resp.error_for_status_ref().map_err(FetchError::Http)?;
        resp.json::<T>().await.map_err(FetchError::Http)
    }

    /// Raw bytes for image-plane downloads (At-Home page images).
    pub async fn get_bytes(&self, url: &str, plane: Plane) -> Result<Vec<u8>, FetchError> {
        let parsed = reqwest::Url::parse(url)
            .map_err(|_e| FetchError::Unexpected { url: url.to_string(), status: 0 })?;
        let host = parsed
            .host_str()
            .ok_or_else(|| FetchError::Unexpected { url: url.to_string(), status: 0 })?
            .to_string();
        if !host_allowed(&host, &self.allowlist) {
            return Err(FetchError::Disallowed { host });
        }

        self.throttle(plane).await;

        let resp = self.http.get(url).send().await?;
        let bytes = resp.bytes().await?.to_vec();
        Ok(bytes)
    }
}

/// Shared-handle convenience alias.
pub type SharedClient = Arc<OutboundClient>;

#[cfg(test)]
mod tests {
    use super::*;

    fn doms(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn allowlist_matches_domains_not_substrings() {
        assert!(host_allowed("example.test", &doms(&["example.test"])));
        assert!(host_allowed("a.b.example.test", &doms(&["example.test"])));
        assert!(host_allowed("EXAMPLE.TEST", &doms(&["example.test"])));
        assert!(!host_allowed("evil-allowed.test", &doms(&["example.test"])));
        assert!(!host_allowed("test", &doms(&["example.test"])));
        assert!(!host_allowed("", &doms(&["example.test"])));
    }

    #[tokio::test]
    async fn disallowed_host_is_rejected_before_any_delay() {
        let c = OutboundClient::new(vec!["example.test".into()], 1, 1, Duration::from_secs(1))
            .expect("client");
        let t0 = Instant::now();
        let err = c
            .get_json::<serde_json::Value>("http://disallowed.test/x", Plane::Api)
            .await
            .expect_err("must reject");
        assert!(err.to_string().contains("allowlist"), "got {err}");
        assert!(t0.elapsed() < Duration::from_millis(200), "rejection must be pre-connection, pre-throttle");
    }

    #[tokio::test]
    async fn plane_limiter_throttles_beyond_budget() {
        let mut lim = PlaneLimiter::new(2, Duration::from_millis(300));
        let t0 = Instant::now();
        while let Some(wait) = lim.try_grant() {
            tokio::time::sleep(wait).await;
        }
        while let Some(wait) = lim.try_grant() {
            tokio::time::sleep(wait).await;
        }
        while let Some(wait) = lim.try_grant() {
            tokio::time::sleep(wait).await;
        }
        while let Some(wait) = lim.try_grant() {
            tokio::time::sleep(wait).await;
        }
        let elapsed = t0.elapsed();
        assert!(
            elapsed >= Duration::from_millis(250),
            "4th request must wait for window rollover, took {elapsed:?}"
        );
    }

    #[tokio::test]
    async fn planes_hold_independent_grant_queues() {
        let mut limiters = [
            PlaneLimiter::new(2, Duration::from_secs(10)),
            PlaneLimiter::new(1, Duration::from_secs(10)),
        ];
        let _ = limiters[0].try_grant();
        let _ = limiters[0].try_grant();
        let _ = limiters[1].try_grant();
        // no waits observed: independent windows per plane, budgets not shared
    }
}
