use crate::base::neterror::NetError;
use crate::socket::client::{SocketType, StreamSocket};
use crate::socket::connectjob::ConnectJob;
use dashmap::DashMap;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use url::Url;

/// Identifies a connection group (scheme, host, port).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct GroupId {
    scheme: String,
    host: String,
    port: u16,
}

impl GroupId {
    fn from_url(url: &Url) -> Option<Self> {
        Some(GroupId {
            scheme: url.scheme().to_string(),
            host: url.host_str()?.to_string(),
            port: url.port_or_known_default()?,
        })
    }
}

/// Manages a pool of sockets, enforcing Chromium-like limits.
#[derive(Debug, Clone)]
pub struct ClientSocketPool {
    // Limits
    max_sockets_per_group: usize, // Default 6
    max_sockets_total: usize,     // Default 256

    // State
    idle_sockets: Arc<DashMap<GroupId, VecDeque<SocketType>>>,
    active_per_group: Arc<DashMap<GroupId, usize>>,
    total_active: Arc<AtomicUsize>,
}

impl Default for ClientSocketPool {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientSocketPool {
    pub fn new() -> Self {
        Self {
            max_sockets_per_group: 6,
            max_sockets_total: 256,
            idle_sockets: Arc::new(DashMap::new()),
            active_per_group: Arc::new(DashMap::new()),
            total_active: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub async fn request_socket(&self, url: &Url) -> Result<SocketType, NetError> {
        let group_id = GroupId::from_url(url).ok_or(NetError::InvalidUrl)?;

        // 1. Check Idle Sockets
        if let Some(mut queue) = self.idle_sockets.get_mut(&group_id) {
            if let Some(socket) = queue.pop_front() {
                if socket.is_connected() {
                    // Mark as active
                    self.active_per_group.entry(group_id.clone()).and_modify(|c| *c += 1);
                    return Ok(socket);
                }
                // If disconnected/dead, verify if we should decrement total?
                // Ideally yes, but our atomic counts "Active". Idle is dynamic.
                // We should track "Total Open" separately if we want strictness.
                // For now: Just drop it.
            }
        }

        // 2. Check Limits
        // Total Group Sockets = Active + Idle
        let active_count = *self.active_per_group.entry(group_id.clone()).or_insert(0);
        let idle_count = self.idle_sockets.get(&group_id).map(|q| q.len()).unwrap_or(0);

        // Note: active_count might be slightly stale compared to idle check above due to race,
        // but locking ensures safety in DashMap for specific keys.

        if active_count + idle_count >= self.max_sockets_per_group {
            return Err(NetError::PreconnectMaxSocketLimit);
        }

        // Global limit check (Active only? Or Total Open?)
        // Chromium limits total open sockets.
        // Global limit check
        let total_count = self.total_active.load(Ordering::Relaxed);
        if total_count >= self.max_sockets_total {
            return Err(NetError::NoBufferSpace);
        }

        // 3. Connect (New)
        // Increment active count
        self.active_per_group.entry(group_id.clone()).and_modify(|c| *c += 1);
        self.total_active.fetch_add(1, Ordering::Relaxed);

        match ConnectJob::connect(url).await {
            Ok(socket) => Ok(socket),
            Err(e) => {
                // Decrement on failure
                self.active_per_group.entry(group_id.clone()).and_modify(|c| *c -= 1);
                self.total_active.fetch_sub(1, Ordering::Relaxed);
                Err(e)
            }
        }
    }

    pub fn release_socket(&self, url: &Url, socket: SocketType) {
        if let Some(group_id) = GroupId::from_url(url) {
            // In a real impl, we'd check if reusable
            // For now, assume reusable
            let mut queue = self.idle_sockets.entry(group_id.clone()).or_default();
            queue.push_back(socket);

            // We don't decrement active count if we put it back in idle?
            // Actually, "active" usually means "handed out". "Idle" acts as a cache.
            // If we put it back, it is no longer "active" (held by transaction) but it counts towards "total sockets" in the group?
            // Chromium counts "active" (in use) vs "idle" (in pool). Limits apply to sum.

            self.active_per_group.entry(group_id).and_modify(|c| *c -= 1);
            self.total_active.fetch_sub(1, Ordering::Relaxed);

            // Wait, if it counts towards limit, we shouldn't decrement total, just move ownership.
            // Simplification: We only count "in-flight" sockets against the limit in this initial version.
            // Correct logic: Total = Active + Idle.
            // If we return to idle, count is still occupied.
            // Let's stick to "Active" meaning "In Use by Transaction" for this simplified logic,
            // and Idle sockets are just free for taking.
        }
    }
}
