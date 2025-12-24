# Toggl Track API Optimization Analysis

**Date:** 2025-11-13
**Project:** Toggl TimeGuru v1.1.1+
**Status:** Implementation Recommendations

## Executive Summary

This document analyzes current Toggl Track API usage patterns in the TimeGuru application and identifies critical optimization opportunities to avoid rate limiting, especially during batch operations like project assignment to grouped time entries.

**Key Finding:** The application currently makes sequential individual API calls when updating multiple time entries, which can quickly exhaust API rate limits. The Toggl Track API provides a bulk update endpoint that can update up to 100 time entries in a single request, which would reduce API calls by 99% in typical batch scenarios.

---

## Current Rate Limits (As of September 2025)

Toggl Track enforces sliding window rate limits (60-minute windows) per user per organization:

| Subscription Tier | Requests/Hour (per user/org) | Requests/Second (safe) |
|-------------------|------------------------------|------------------------|
| **Free**          | 30                           | ~0.008 (1 per 2 min)   |
| **Starter**       | 240                          | ~0.067 (1 per 15 sec)  |
| **Premium**       | 600                          | ~0.167 (1 per 6 sec)   |
| **Enterprise**    | Custom (higher)              | Varies                 |
| **User-specific** | 30 (all plans)               | ~0.008                 |

**Response Codes:**
- `429 Too Many Requests` - Leaky bucket safe-guard (general rate limiting)
- `402 Payment Required` - Sliding window quota exceeded

**Rate Limit Headers:**
- `X-Toggl-Quota-Remaining` - Number of requests remaining in current window
- `X-Toggl-Quota-Resets-In` - Seconds until quota window resets

---

## Current Implementation Analysis

### 1. Project Assignment (Single Entry)

**Location:** `src/ui/app.rs:992-1000`

```rust
handle.spawn(async move {
    let result = client_clone
        .update_time_entry_project(workspace_id, entry_id, Some(project_id))
        .await;
    let _ = tx.send(result);
});
```

**API Call:** `PUT /api/v9/workspaces/{workspace_id}/time_entries/{entry_id}`
**Rate Impact:** 1 request per time entry

### 2. Project Assignment (Grouped Entries - CRITICAL ISSUE)

**Location:** `src/ui/app.rs:879-943`

```rust
for entry in &grouped_entry.entries {
    // ... spawns async task for EACH entry ...
    handle.spawn(async move {
        let result = client_clone
            .update_time_entry_project(workspace_id, entry_id, Some(project_id))
            .await;
        let _ = tx.send(result);
    });
    // Waits for each request synchronously via rx.recv()
}
```

**Problem:**
- When assigning a project to a grouped entry with N individual time entries, this makes **N sequential API calls**
- Example: Assigning a project to a group with 50 entries = 50 API requests
- On Free tier (30 req/hour), this exhausts the entire quota in a single operation
- On Starter tier (240 req/hour), 5 such operations exhaust the quota

### 3. Description Updates (Similar Pattern)

**Location:** `src/ui/app.rs:451-484`

```rust
for (workspace_id, entry_id) in entries_to_update.iter() {
    // ... spawns async task for EACH entry ...
    handle.spawn(async move {
        let result = client_clone
            .update_time_entry_description(...)
            .await;
        // ...
    });
}
```

**Rate Impact:** 1 request per time entry (same issue as project assignment)

### 4. Current Rate Limit Handling

**Location:** `src/toggl/client.rs:154-183`

```rust
match status {
    StatusCode::TOO_MANY_REQUESTS => {
        warn!("Rate limit hit, will retry if attempts remain");
        last_error = Some(anyhow::anyhow!("Rate limit exceeded"));
        continue;
    }
    // ...
}
```

**Current Capabilities:**
- ✅ Handles 429 (TOO_MANY_REQUESTS) with retry + exponential backoff
- ✅ Implements retry logic with max 3 attempts
- ❌ Does NOT handle 402 (quota exceeded)
- ❌ Does NOT read or utilize `X-Toggl-Quota-Remaining` header
- ❌ Does NOT read or utilize `X-Toggl-Quota-Resets-In` header
- ❌ No proactive throttling based on remaining quota

---

## Available Bulk Update Endpoint

### Endpoint Specification

**URL:** `PATCH https://api.track.toggl.com/api/v9/workspaces/{workspace_id}/time_entries/{time_entry_ids}`

**Path Parameters:**
- `workspace_id` (integer): Workspace ID
- `time_entry_ids` (string): Comma-separated entry IDs (max 100 per request)

**Request Body Format (JSON Patch - RFC 6902):**
```json
[
  {
    "op": "replace",
    "path": "/project_id",
    "value": 123456
  }
]
```

**Supported Operations:**
- `op`: "add" | "remove" | "replace"
- Common paths: `/project_id`, `/description`, `/tags`, `/billable`, etc.

**Response Format:**
```json
{
  "success": [204301830, 202700150, 202687559],
  "failure": [
    {
      "id": 202687560,
      "message": "Time entry not found"
    }
  ]
}
```

**Key Characteristics:**
- ✅ Batch update up to 100 time entries in a single request
- ✅ Partial execution (no transaction/rollback)
- ✅ Returns success/failure arrays for tracking
- ⚠️ Each update is independent (some may succeed while others fail)

### Example Use Cases

**1. Batch Project Assignment (Current Use Case)**
```json
PATCH /api/v9/workspaces/12345/time_entries/101,102,103,104,105
[
  {
    "op": "replace",
    "path": "/project_id",
    "value": 98765
  }
]
```

**2. Batch Description Update**
```json
PATCH /api/v9/workspaces/12345/time_entries/101,102,103
[
  {
    "op": "replace",
    "path": "/description",
    "value": "Updated task name"
  }
]
```

**3. Multiple Field Updates (Future)**
```json
PATCH /api/v9/workspaces/12345/time_entries/101,102
[
  {
    "op": "replace",
    "path": "/project_id",
    "value": 98765
  },
  {
    "op": "replace",
    "path": "/billable",
    "value": true
  }
]
```

---

## Performance Impact Comparison

### Scenario: Assigning Project to 50 Grouped Entries

| Approach | API Requests | Free Tier Impact | Starter Tier Impact | Premium Tier Impact |
|----------|--------------|------------------|---------------------|---------------------|
| **Current (Sequential)** | 50 | 167% quota (fails) | 21% quota | 8% quota |
| **Optimized (Bulk)** | 1 | 3% quota | 0.4% quota | 0.2% quota |
| **Savings** | **-49 requests** | **-164%** | **-20.6%** | **-7.8%** |

### Scenario: Editing Description for 30 Entries

| Approach | API Requests | Free Tier Impact | Starter Tier Impact | Premium Tier Impact |
|----------|--------------|------------------|---------------------|---------------------|
| **Current (Sequential)** | 30 | 100% quota (limit) | 13% quota | 5% quota |
| **Optimized (Bulk)** | 1 | 3% quota | 0.4% quota | 0.2% quota |
| **Savings** | **-29 requests** | **-97%** | **-12.6%** | **-4.8%** |

### Scenario: User Assigns Projects to 5 Groups (avg 40 entries each)

| Approach | API Requests | Free Tier Impact | Starter Tier Impact | Premium Tier Impact |
|----------|--------------|------------------|---------------------|---------------------|
| **Current (Sequential)** | 200 | 667% quota (fails) | 83% quota | 33% quota |
| **Optimized (Bulk)** | 5 | 17% quota | 2% quota | 0.8% quota |
| **Savings** | **-195 requests** | **-650%** | **-81%** | **-32.2%** |

**Key Takeaway:** Even Premium tier users can hit rate limits with the current implementation during moderate usage. Free tier is essentially unusable for batch operations.

---

## Optimization Recommendations

### Priority 1: Implement Bulk Update Endpoint (CRITICAL)

**Impact:** Reduces API calls by 99% for batch operations
**Effort:** Medium (2-3 days)
**Risk:** Low (backward compatible)

**Implementation Steps:**

1. **Add Bulk Update Methods to TogglClient**

   **File:** `src/toggl/client.rs`

   ```rust
   pub struct BulkUpdateOperation {
       pub op: String,        // "add" | "remove" | "replace"
       pub path: String,      // "/project_id", "/description", etc.
       pub value: serde_json::Value,
   }

   pub struct BulkUpdateResponse {
       pub success: Vec<i64>,
       pub failure: Vec<BulkUpdateFailure>,
   }

   pub struct BulkUpdateFailure {
       pub id: i64,
       pub message: String,
   }

   impl TogglClient {
       pub async fn bulk_update_time_entries(
           &self,
           workspace_id: i64,
           entry_ids: &[i64],
           operations: Vec<BulkUpdateOperation>,
       ) -> Result<BulkUpdateResponse> {
           // Max 100 IDs per request
           if entry_ids.len() > 100 {
               anyhow::bail!("Cannot update more than 100 entries per request");
           }

           let ids_str = entry_ids
               .iter()
               .map(|id| id.to_string())
               .collect::<Vec<_>>()
               .join(",");

           let url = format!(
               "{}/workspaces/{}/time_entries/{}",
               self.base_url, workspace_id, ids_str
           );

           let body: Vec<serde_json::Value> = operations
               .into_iter()
               .map(|op| {
                   serde_json::json!({
                       "op": op.op,
                       "path": op.path,
                       "value": op.value
                   })
               })
               .collect();

           let response = self
               .client
               .patch(&url)
               .header(header::AUTHORIZATION, self.auth_header())
               .json(&body)
               .send()
               .await
               .context("Failed to send bulk update request")?;

           match response.status() {
               StatusCode::OK => {
                   let result = response
                       .json::<BulkUpdateResponse>()
                       .await
                       .context("Failed to parse bulk update response")?;
                   Ok(result)
               }
               StatusCode::FORBIDDEN | StatusCode::UNAUTHORIZED => {
                   Err(anyhow::anyhow!("Authentication failed"))
               }
               StatusCode::TOO_MANY_REQUESTS => {
                   Err(anyhow::anyhow!("Rate limit exceeded (429)"))
               }
               status => {
                   let error_text = response.text().await.unwrap_or_default();
                   Err(anyhow::anyhow!(
                       "Bulk update failed. Status: {}, Error: {}",
                       status, error_text
                   ))
               }
           }
       }

       /// Convenience method for bulk project assignment
       pub async fn bulk_assign_project(
           &self,
           workspace_id: i64,
           entry_ids: &[i64],
           project_id: Option<i64>,
       ) -> Result<BulkUpdateResponse> {
           let value = match project_id {
               Some(id) => serde_json::Value::Number(id.into()),
               None => serde_json::Value::Null,
           };

           let operations = vec![BulkUpdateOperation {
               op: "replace".to_string(),
               path: "/project_id".to_string(),
               value,
           }];

           self.bulk_update_time_entries(workspace_id, entry_ids, operations)
               .await
       }

       /// Convenience method for bulk description update
       pub async fn bulk_update_descriptions(
           &self,
           workspace_id: i64,
           entry_ids: &[i64],
           description: String,
       ) -> Result<BulkUpdateResponse> {
           let operations = vec![BulkUpdateOperation {
               op: "replace".to_string(),
               path: "/description".to_string(),
               value: serde_json::Value::String(description),
           }];

           self.bulk_update_time_entries(workspace_id, entry_ids, operations)
               .await
       }
   }
   ```

2. **Update App to Use Bulk Operations**

   **File:** `src/ui/app.rs`

   Replace the loop in `assign_project_to_entry()` for grouped entries:

   ```rust
   // BEFORE (lines 879-943)
   for entry in &grouped_entry.entries {
       // ... individual API calls ...
   }

   // AFTER
   if self.show_grouped {
       let grouped_entry = match self.grouped_entries.get(selected_entry_idx) {
           Some(e) => e,
           None => {
               self.status_message = Some("Invalid entry selection".to_string());
               return;
           }
       };

       // Collect all entry IDs and workspace_id
       let entry_ids: Vec<i64> = grouped_entry.entries.iter().map(|e| e.id).collect();
       let workspace_id = grouped_entry.entries[0].workspace_id;

       // Spawn single async task for bulk update
       let (tx, rx) = std::sync::mpsc::channel();
       let client_clone = client.clone();

       handle.spawn(async move {
           let result = client_clone
               .bulk_assign_project(workspace_id, &entry_ids, Some(project_id))
               .await;
           let _ = tx.send(result);
       });

       match rx.recv() {
           Ok(Ok(bulk_result)) => {
               let success_count = bulk_result.success.len();
               let fail_count = bulk_result.failure.len();

               // Update local state for successful entries
               for entry_id in &bulk_result.success {
                   if let Some(entry) = self.time_entries.iter_mut().find(|e| e.id == *entry_id) {
                       entry.project_id = Some(project_id);
                   }
                   if let Some(entry) = self.all_entries.iter_mut().find(|e| e.id == *entry_id) {
                       entry.project_id = Some(project_id);
                   }
                   // Update database
                   let _ = self.db.update_time_entry_project(*entry_id, Some(project_id));
               }

               // Log failures
               for failure in &bulk_result.failure {
                   tracing::error!("Failed to update entry {}: {}", failure.id, failure.message);
               }

               if fail_count == 0 {
                   self.status_message = Some(format!(
                       "Assigned {} to {} entries",
                       project_name, success_count
                   ));
               } else {
                   self.status_message = Some(format!(
                       "Assigned {} to {}/{} entries ({} failed)",
                       project_name, success_count, entry_ids.len(), fail_count
                   ));
               }
           }
           Ok(Err(e)) => {
               self.error_message = Some(format!("Failed to assign project: {}", e));
           }
           Err(e) => {
               self.error_message = Some(format!("Error communicating with API: {}", e));
           }
       }

       self.recompute_grouped_entries();
       self.show_project_selector = false;
       self.project_search_query.clear();
       self.reset_filtered_projects();
   }
   ```

   Similar changes for `save_edited_description()` method.

3. **Handle Batches > 100 Entries**

   ```rust
   // Split into chunks of 100
   let entry_ids: Vec<i64> = grouped_entry.entries.iter().map(|e| e.id).collect();
   let chunks: Vec<&[i64]> = entry_ids.chunks(100).collect();

   let mut total_success = 0;
   let mut total_failures = 0;

   for chunk in chunks {
       // Process each chunk sequentially (still better than N individual calls)
       let (tx, rx) = std::sync::mpsc::channel();
       let client_clone = client.clone();
       let chunk_vec = chunk.to_vec();

       handle.spawn(async move {
           let result = client_clone
               .bulk_assign_project(workspace_id, &chunk_vec, Some(project_id))
               .await;
           let _ = tx.send(result);
       });

       match rx.recv() {
           Ok(Ok(bulk_result)) => {
               total_success += bulk_result.success.len();
               total_failures += bulk_result.failure.len();
               // ... update local state ...
           }
           Ok(Err(e)) => {
               tracing::error!("Chunk failed: {}", e);
               total_failures += chunk.len();
           }
           Err(e) => {
               tracing::error!("Channel error: {}", e);
               total_failures += chunk.len();
           }
       }
   }
   ```

**Testing Checklist:**
- [ ] Bulk update 10 entries (single batch)
- [ ] Bulk update 150 entries (2 batches of 100 + 50)
- [ ] Handle partial failures (some entries succeed, some fail)
- [ ] Verify database sync after bulk operations
- [ ] Test with invalid workspace_id (auth failure)
- [ ] Test with mixed valid/invalid entry IDs

---

### Priority 2: Rate Limit Monitoring & Proactive Throttling

**Impact:** Prevents hitting rate limits, provides better UX
**Effort:** Low (1 day)
**Risk:** Low

**Implementation:**

1. **Track Rate Limit Headers**

   **File:** `src/toggl/client.rs`

   ```rust
   pub struct RateLimitInfo {
       pub remaining: Option<u32>,
       pub resets_in: Option<u32>,
       pub last_updated: std::time::Instant,
   }

   pub struct TogglClient {
       // ... existing fields ...
       rate_limit_info: Arc<Mutex<RateLimitInfo>>,
   }

   impl TogglClient {
       fn extract_rate_limit_headers(&self, response: &reqwest::Response) {
           let remaining = response
               .headers()
               .get("X-Toggl-Quota-Remaining")
               .and_then(|v| v.to_str().ok())
               .and_then(|s| s.parse::<u32>().ok());

           let resets_in = response
               .headers()
               .get("X-Toggl-Quota-Resets-In")
               .and_then(|v| v.to_str().ok())
               .and_then(|s| s.parse::<u32>().ok());

           if let Ok(mut info) = self.rate_limit_info.lock() {
               info.remaining = remaining;
               info.resets_in = resets_in;
               info.last_updated = std::time::Instant::now();

               if let Some(r) = remaining {
                   if r < 10 {
                       warn!("Rate limit low: {} requests remaining", r);
                   }
               }
           }
       }

       pub fn get_rate_limit_info(&self) -> Option<RateLimitInfo> {
           self.rate_limit_info.lock().ok().map(|info| RateLimitInfo {
               remaining: info.remaining,
               resets_in: info.resets_in,
               last_updated: info.last_updated,
           })
       }
   }
   ```

2. **Update All API Methods to Extract Headers**

   Add to every API method after receiving response:
   ```rust
   let response = self.client.get(&url).send().await?;
   self.extract_rate_limit_headers(&response);
   // ... rest of processing ...
   ```

3. **Add Proactive Throttling**

   ```rust
   impl TogglClient {
       async fn check_rate_limit_before_request(&self) -> Result<()> {
           if let Some(info) = self.get_rate_limit_info() {
               if let Some(remaining) = info.remaining {
                   if remaining == 0 {
                       if let Some(resets_in) = info.resets_in {
                           warn!("Rate limit exhausted, waiting {} seconds", resets_in);
                           tokio::time::sleep(Duration::from_secs(resets_in as u64)).await;
                       } else {
                           // Conservative wait if no reset time available
                           tokio::time::sleep(Duration::from_secs(60)).await;
                       }
                   } else if remaining < 5 {
                       // Slow down when approaching limit
                       tokio::time::sleep(Duration::from_millis(500)).await;
                   }
               }
           }
           Ok(())
       }
   }
   ```

4. **Handle 402 Payment Required**

   Add to status code matching:
   ```rust
   StatusCode::PAYMENT_REQUIRED => {
       if let Some(resets_in) = response
           .headers()
           .get("X-Toggl-Quota-Resets-In")
           .and_then(|v| v.to_str().ok())
           .and_then(|s| s.parse::<u64>().ok())
       {
           warn!("Quota exceeded (402), waiting {} seconds", resets_in);
           tokio::time::sleep(Duration::from_secs(resets_in)).await;
           // Retry the request
           last_error = Some(anyhow::anyhow!("Quota exceeded, retrying"));
           continue;
       } else {
           return Err(anyhow::anyhow!("Quota exceeded and no reset time provided"));
       }
   }
   ```

---

### Priority 3: Request Queue with Intelligent Throttling

**Impact:** Smooth out request bursts, better UX for rate-limited tiers
**Effort:** Medium-High (3-4 days)
**Risk:** Medium (requires architectural changes)

**Concept:**

Implement a request queue that:
1. Queues API requests instead of sending immediately
2. Dispatches requests at a controlled rate based on subscription tier
3. Prioritizes requests (user-initiated > background sync)
4. Provides progress feedback to user

**Example Architecture:**

```rust
pub struct ApiRequestQueue {
    sender: mpsc::Sender<ApiRequest>,
    rate_limiter: Arc<RateLimiter>,
}

pub struct ApiRequest {
    priority: RequestPriority,
    execute: Box<dyn FnOnce(&TogglClient) -> BoxFuture<'static, Result<()>>>,
    result_sender: oneshot::Sender<Result<()>>,
}

pub enum RequestPriority {
    Critical,  // User-initiated actions
    Normal,    // Standard operations
    Background, // Sync operations
}
```

**Note:** This is a more complex solution and should only be implemented if Priority 1 & 2 don't sufficiently solve the problem.

---

### Priority 4: Additional Optimizations

1. **Cache Projects Data Locally**
   - Projects rarely change
   - Fetch once per TUI session, only refresh on demand
   - Saves API calls during repeated operations

2. **Lazy Load Time Entries**
   - Only fetch visible date range initially
   - Implement pagination for large datasets
   - Reduces initial sync time and API usage

3. **Background Sync Optimization**
   - Use `If-Modified-Since` headers where supported
   - Only sync entries that changed since last sync
   - Track sync metadata per resource

4. **Telemetry for Rate Limit Incidents**
   - Log when users hit rate limits
   - Track which subscription tier experiences issues
   - Use data to prioritize optimizations

---

## Implementation Roadmap

### Phase 1: Critical (v1.2.0 target)
**Timeline:** 1 week

- [ ] Implement bulk update endpoint in TogglClient
- [ ] Add bulk project assignment to TUI
- [ ] Add bulk description update to TUI
- [ ] Handle batches > 100 entries
- [ ] Add unit tests for bulk operations
- [ ] Update PROGRESS.md and VERSION_TIMELINE.md

### Phase 2: Important (v1.2.1 target)
**Timeline:** 3-4 days

- [ ] Add rate limit header extraction
- [ ] Implement proactive throttling
- [ ] Handle 402 status code properly
- [ ] Display rate limit info in TUI footer
- [ ] Add warning when approaching rate limit

### Phase 3: Enhancement (v1.3.0 target)
**Timeline:** 1 week

- [ ] Implement request queue (if needed)
- [ ] Add priority handling
- [ ] Progress indicators for queued requests
- [ ] Local project caching
- [ ] Optimized background sync

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bulk_update_project_assignment() {
        let client = TogglClient::new("test_token".to_string()).unwrap();
        let entry_ids = vec![1, 2, 3, 4, 5];
        let result = client
            .bulk_assign_project(12345, &entry_ids, Some(98765))
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_bulk_update_over_100_entries() {
        let client = TogglClient::new("test_token".to_string()).unwrap();
        let entry_ids: Vec<i64> = (1..=150).collect();
        let result = client
            .bulk_assign_project(12345, &entry_ids, Some(98765))
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("100"));
    }

    #[test]
    fn test_rate_limit_info_extraction() {
        // Test header parsing logic
    }
}
```

### Integration Tests

1. **Mock API Server with Rate Limiting**
   - Use `mockito` to simulate Toggl API
   - Return 429/402 responses after N requests
   - Verify retry and backoff logic

2. **Bulk Update Scenarios**
   - Test with real API (staging account)
   - Verify success/failure arrays match expectations
   - Confirm database state after partial failures

3. **Rate Limit Scenarios**
   - Trigger rate limits intentionally
   - Verify headers are extracted correctly
   - Confirm throttling logic activates

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| **Bulk API bugs cause data corruption** | Low | High | Thorough testing, rollback strategy, partial failure handling |
| **Breaking API changes** | Low | Medium | Version API client, maintain backward compatibility |
| **Performance regression** | Low | Medium | Benchmark before/after, load testing |
| **Partial failures confuse users** | Medium | Low | Clear error messages, detailed status reporting |
| **Rate limit logic errors** | Medium | Medium | Conservative defaults, extensive logging |

---

## Monitoring & Success Metrics

### Key Metrics to Track

1. **API Request Reduction**
   - Target: 90%+ reduction in API calls for batch operations
   - Measure: Requests per grouped project assignment (before/after)

2. **Rate Limit Incidents**
   - Target: Zero 429/402 errors in normal usage
   - Measure: Error logs, user reports

3. **Operation Latency**
   - Target: < 2 seconds for batch operations (100 entries)
   - Measure: Time from user action to completion message

4. **User Experience**
   - Target: No "rate limit exceeded" errors reported by users
   - Measure: GitHub issues, user feedback

### Logging & Telemetry

Add structured logging:
```rust
tracing::info!(
    api_call = "bulk_assign_project",
    workspace_id = workspace_id,
    entry_count = entry_ids.len(),
    duration_ms = elapsed.as_millis(),
    success_count = result.success.len(),
    failure_count = result.failure.len(),
    "Completed bulk project assignment"
);
```

---

## Conclusion

The current implementation is **not sustainable for Free and Starter tier users** when performing batch operations. Implementing the bulk update endpoint (Priority 1) is **critical** and should be completed before any feature releases that encourage batch editing.

**Estimated Impact:**
- **Free tier:** Usable for batch operations (currently unusable)
- **Starter tier:** 20x more operations per hour
- **Premium tier:** 12x more operations per hour
- **All tiers:** Faster operations (1 request vs N requests)

**Recommended Next Steps:**
1. Implement Priority 1 (bulk update endpoint) immediately
2. Add basic rate limit monitoring (Priority 2) in same release
3. Evaluate need for Priority 3 (request queue) after user feedback
4. Schedule Priority 4 optimizations for future versions

**Version Assignment:**
- v1.2.0: Priority 1 + Priority 2 (critical fixes)
- v1.2.1: Additional bulk operations (tags, billable status)
- v1.3.0: Priority 3 (if needed) + Priority 4 optimizations

---

## References

- [Toggl Track API Rate Limits](https://support.toggl.com/en/articles/11484112-api-webhook-limits)
- [Toggl Track API FAQs about Limits](https://support.toggl.com/en/articles/11623558-faqs-about-api-limits)
- [Toggl Track Bulk Update Endpoint](https://community.toggl.com/t/time-entries-endpoints-patch-bulk-editing-time-entries/186)
- [RFC 6902 - JSON Patch](https://datatracker.ietf.org/doc/html/rfc6902)
- [RFC 6901 - JSON Pointer](https://datatracker.ietf.org/doc/html/rfc6901)
- Current implementation: `src/toggl/client.rs`, `src/ui/app.rs`

---

**Document Status:** ✅ Ready for Review
**Last Updated:** 2025-11-13
**Next Review:** After v1.2.0 implementation
