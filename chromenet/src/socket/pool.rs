use crate::base::neterror::NetError;
use crate::socket::client::{SocketType, StreamSocket};
use crate::socket::connectjob::ConnectJob;
use dashmap::DashMap;
use std::cmp::Ordering as CmpOrdering;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::oneshot;
use url::Url;

/// Request priority (matches Chromium's RequestPriority).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum RequestPriority {
    Throttled = 0,
    Idle = 1,
    Lowest = 2,
    Low = 3,
    #[default]
    Medium = 4,
    Highest = 5,
}

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

/// A pending socket request waiting in queue.
struct PendingRequest {
    priority: RequestPriority,
    sender: oneshot::Sender<Result<(SocketType, bool), NetError>>,
    url: Url,
    proxy: Option<crate::socket::proxy::ProxySettings>,
    created_at: std::time::Instant,
}

impl PartialEq for PendingRequest {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.created_at == other.created_at
    }
}

impl Eq for PendingRequest {}

impl PartialOrd for PendingRequest {
    fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> {
        Some(self.cmp(other))
    }
}

impl Ord for PendingRequest {
    fn cmp(&self, other: &Self) -> CmpOrdering {
        // Higher priority first, older requests first (FIFO within priority)
        match self.priority.cmp(&other.priority) {
            CmpOrdering::Equal => other.created_at.cmp(&self.created_at), // Older first
            other => other,
        }
    }
}

/// Per-group state tracking.
struct Group {
    idle_sockets: VecDeque<IdleSocket>,
    active_count: usize,
    pending_requests: Vec<PendingRequest>,
}

/// Idle socket with metadata for timeout tracking.
struct IdleSocket {
    socket: SocketType,
    /// When this socket was returned to the pool
    start_time: std::time::Instant,
    /// Whether the socket was ever used for data transfer
    was_used: bool,
}

impl Group {
    fn new() -> Self {
        Self { idle_sockets: VecDeque::new(), active_count: 0, pending_requests: Vec::new() }
    }

    fn total_slots(&self) -> usize {
        self.active_count + self.idle_sockets.len()
    }

    fn has_available_slot(&self, max_per_group: usize) -> bool {
        self.total_slots() < max_per_group
    }

    fn pop_highest_priority_request(&mut self) -> Option<PendingRequest> {
        if self.pending_requests.is_empty() {
            return None;
        }
        // Find index of highest priority request
        let max_idx = self
            .pending_requests
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.cmp(b))
            .map(|(i, _)| i)?;
        Some(self.pending_requests.swap_remove(max_idx))
    }
}

/// Manages a pool of sockets, enforcing Chromium-like limits.
/// Now with request queuing when limits are reached.
pub struct ClientSocketPool {
    // Limits
    max_sockets_per_group: usize, // Default 6
    max_sockets_total: usize,     // Default 256

    // State
    groups: Arc<DashMap<GroupId, Group>>,
    total_active: Arc<AtomicUsize>,
}

impl Clone for ClientSocketPool {
    fn clone(&self) -> Self {
        Self {
            max_sockets_per_group: self.max_sockets_per_group,
            max_sockets_total: self.max_sockets_total,
            groups: Arc::clone(&self.groups),
            total_active: Arc::clone(&self.total_active),
        }
    }
}

impl std::fmt::Debug for ClientSocketPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientSocketPool")
            .field("max_sockets_per_group", &self.max_sockets_per_group)
            .field("max_sockets_total", &self.max_sockets_total)
            .field("total_active", &self.total_active.load(Ordering::Relaxed))
            .finish()
    }
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
            groups: Arc::new(DashMap::new()),
            total_active: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Request a socket with default priority.
    pub async fn request_socket(
        &self,
        url: &Url,
        proxy: Option<&crate::socket::proxy::ProxySettings>,
    ) -> Result<(SocketType, bool), NetError> {
        self.request_socket_with_priority(url, proxy, RequestPriority::default()).await
    }

    /// Request a socket with specified priority.
    /// If limits are reached, the request is queued and will be fulfilled when a socket becomes available.
    pub async fn request_socket_with_priority(
        &self,
        url: &Url,
        proxy: Option<&crate::socket::proxy::ProxySettings>,
        priority: RequestPriority,
    ) -> Result<(SocketType, bool), NetError> {
        let group_id = GroupId::from_url(url).ok_or(NetError::InvalidUrl)?;

        // Try to get socket immediately
        if let Some(result) = self.try_get_socket_immediate(&group_id, url, proxy).await? {
            return Ok(result);
        }

        // Queue the request and wait
        let (tx, rx) = oneshot::channel();
        {
            let mut group = self.groups.entry(group_id.clone()).or_insert_with(Group::new);
            group.pending_requests.push(PendingRequest {
                priority,
                sender: tx,
                url: url.clone(),
                proxy: proxy.cloned(),
                created_at: std::time::Instant::now(),
            });
        }

        // Wait for socket to become available
        rx.await.map_err(|_| NetError::ConnectionAborted)?
    }

    /// Try to get a socket immediately without queuing.
    async fn try_get_socket_immediate(
        &self,
        group_id: &GroupId,
        url: &Url,
        proxy: Option<&crate::socket::proxy::ProxySettings>,
    ) -> Result<Option<(SocketType, bool)>, NetError> {
        let mut group = self.groups.entry(group_id.clone()).or_insert_with(Group::new);

        // 1. Check for idle socket
        while let Some(idle_socket) = group.idle_sockets.pop_front() {
            if idle_socket.socket.is_connected() {
                group.active_count += 1;
                self.total_active.fetch_add(1, Ordering::Relaxed);
                return Ok(Some((idle_socket.socket, true)));
            }
            // Dead socket, continue to next
        }

        // 2. Check limits
        if !group.has_available_slot(self.max_sockets_per_group) {
            return Ok(None); // Will be queued
        }

        let total = self.total_active.load(Ordering::Relaxed);
        if total >= self.max_sockets_total {
            return Ok(None); // Will be queued
        }

        // 3. Create new connection
        group.active_count += 1;
        self.total_active.fetch_add(1, Ordering::Relaxed);
        drop(group); // Release lock before async connect

        match ConnectJob::connect(url, proxy).await {
            Ok(socket) => Ok(Some((socket, false))),
            Err(e) => {
                // Decrement on failure
                let mut group = self.groups.entry(group_id.clone()).or_insert_with(Group::new);
                group.active_count = group.active_count.saturating_sub(1);
                self.total_active.fetch_sub(1, Ordering::Relaxed);
                Err(e)
            }
        }
    }

    /// Release a socket back to the pool.
    pub fn release_socket(&self, url: &Url, socket: SocketType) {
        let Some(group_id) = GroupId::from_url(url) else {
            return;
        };

        let pending_request = {
            let mut group = self.groups.entry(group_id.clone()).or_insert_with(Group::new);
            group.active_count = group.active_count.saturating_sub(1);
            self.total_active.fetch_sub(1, Ordering::Relaxed);

            // Check if there's a pending request to fulfill
            if socket.is_connected() {
                group.pop_highest_priority_request()
            } else {
                None
            }
        };

        if let Some(request) = pending_request {
            // Hand socket to waiting request
            let mut group = self.groups.entry(group_id.clone()).or_insert_with(Group::new);
            group.active_count += 1;
            self.total_active.fetch_add(1, Ordering::Relaxed);
            drop(group);

            let _ = request.sender.send(Ok((socket, true)));
        } else if socket.is_connected() {
            // Return to idle pool with timestamp
            let mut group = self.groups.entry(group_id).or_insert_with(Group::new);
            group.idle_sockets.push_back(IdleSocket {
                socket,
                start_time: std::time::Instant::now(),
                was_used: true, // Assume it was used
            });
        }
    }

    /// Discard a socket without returning it to the pool.
    pub fn discard_socket(&self, url: &Url) {
        let Some(group_id) = GroupId::from_url(url) else {
            return;
        };

        // Decrement count and process any waiting requests
        let pending = {
            let mut group = self.groups.entry(group_id.clone()).or_insert_with(Group::new);
            group.active_count = group.active_count.saturating_sub(1);
            self.total_active.fetch_sub(1, Ordering::Relaxed);
            group.pop_highest_priority_request()
        };

        if let Some(request) = pending {
            // Start a new connection for the waiting request
            let pool = self.clone();
            tokio::spawn(async move {
                let result = pool
                    .try_get_socket_immediate(
                        &GroupId::from_url(&request.url).unwrap(),
                        &request.url,
                        request.proxy.as_ref(),
                    )
                    .await;

                match result {
                    Ok(Some(socket_result)) => {
                        let _ = request.sender.send(Ok(socket_result));
                    }
                    Ok(None) => {
                        // Still at limit, re-queue (simplified: just fail for now)
                        let _ = request.sender.send(Err(NetError::PreconnectMaxSocketLimit));
                    }
                    Err(e) => {
                        let _ = request.sender.send(Err(e));
                    }
                }
            });
        }
    }

    /// Get number of pending requests for a group.
    pub fn pending_request_count(&self, url: &Url) -> usize {
        GroupId::from_url(url)
            .and_then(|gid| self.groups.get(&gid).map(|g| g.pending_requests.len()))
            .unwrap_or(0)
    }

    /// Get total active socket count.
    pub fn total_active_count(&self) -> usize {
        self.total_active.load(Ordering::Relaxed)
    }

    /// Get total idle socket count across all groups.
    pub fn idle_socket_count(&self) -> usize {
        self.groups.iter().map(|g| g.idle_sockets.len()).sum()
    }

    /// Clean up idle sockets based on timeout.
    /// - Used sockets: 5 minute timeout (Chromium default)
    /// - Unused sockets: 10 second timeout (Chromium unused_idle_socket_timeout)
    pub fn cleanup_idle_sockets(&self) {
        use std::time::Duration;

        const USED_IDLE_TIMEOUT: Duration = Duration::from_secs(300); // 5 minutes
        const UNUSED_IDLE_TIMEOUT: Duration = Duration::from_secs(10); // 10 seconds

        let now = std::time::Instant::now();
        let mut groups_to_remove = Vec::new();

        for mut entry in self.groups.iter_mut() {
            let group = entry.value_mut();

            // Remove expired idle sockets
            group.idle_sockets.retain(|idle_socket| {
                let elapsed = now.duration_since(idle_socket.start_time);
                let timeout =
                    if idle_socket.was_used { USED_IDLE_TIMEOUT } else { UNUSED_IDLE_TIMEOUT };

                // Keep socket if not expired and still connected
                elapsed < timeout && idle_socket.socket.is_connected()
            });

            // Track empty groups for potential cleanup
            if group.idle_sockets.is_empty()
                && group.active_count == 0
                && group.pending_requests.is_empty()
            {
                groups_to_remove.push(entry.key().clone());
            }
        }

        // Remove empty groups
        for gid in groups_to_remove {
            self.groups.remove(&gid);
        }
    }

    /// Start a background task to periodically clean up idle sockets.
    /// Should be called once during initialization.
    pub fn start_cleanup_task(self: &std::sync::Arc<Self>) {
        use std::time::Duration;

        const CLEANUP_INTERVAL: Duration = Duration::from_secs(60);

        let pool = std::sync::Arc::clone(self);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(CLEANUP_INTERVAL).await;
                pool.cleanup_idle_sockets();
            }
        });
    }
}
