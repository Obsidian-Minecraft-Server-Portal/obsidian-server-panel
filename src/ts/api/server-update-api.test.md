# Manual Test Plan for Server Update API

Since automated frontend tests are not yet configured in this project, use this manual test plan to verify the server update API functions.

## Test Setup

1. Start the server: `pnpm run run-api`
2. Open browser developer console
3. Navigate to a server's Options page
4. Use console to call API functions directly

## API Function Tests

### Test 1: Check for Updates

```typescript
// In browser console
import { checkForUpdates } from './ts/api/server-update-api';

// Test with a valid server ID (replace with actual encoded ID)
const serverId = "your-server-id-here";

checkForUpdates(serverId)
  .then(info => {
    console.log('✅ Check updates success:', info);
    // Verify structure:
    // - update_available (boolean)
    // - current_version? (string)
    // - latest_version? (string)
    // - download_url? (string)
    // - changelog_url? (string)
  })
  .catch(err => {
    console.error('❌ Check updates error:', err);
  });
```

**Expected Results:**
- ✅ Returns UpdateInfo object
- ✅ `update_available` is boolean
- ✅ Versions are strings when update available
- ✅ Error message is clear if server not found

### Test 2: Apply Update

```typescript
// In browser console
import { applyUpdate } from './ts/api/server-update-api';

const serverId = "your-server-id-here";

// IMPORTANT: Server must be stopped before applying update!

applyUpdate(serverId)
  .then(result => {
    console.log('✅ Apply update success:', result);
    // Verify structure:
    // - message (string)
    // - new_version (string)
  })
  .catch(err => {
    console.error('❌ Apply update error:', err);
    // Should show error if server is running
  });
```

**Expected Results:**
- ✅ Returns success message with new version
- ✅ Shows error if server is running
- ✅ Shows error if no update available
- ✅ Backup JAR is created before update

### Test 3: Rollback Update

```typescript
// In browser console
import { rollbackUpdate } from './ts/api/server-update-api';

const serverId = "your-server-id-here";

rollbackUpdate(serverId)
  .then(result => {
    console.log('✅ Rollback success:', result);
    // Verify structure:
    // - message (string)
  })
  .catch(err => {
    console.error('❌ Rollback error:', err);
    // Should show error if no backup exists
  });
```

**Expected Results:**
- ✅ Returns success message
- ✅ Shows error if no backup exists
- ✅ Restores previous JAR file

### Test 4: List Backup JARs

```typescript
// In browser console
import { listBackupJars } from './ts/api/server-update-api';

const serverId = "your-server-id-here";

listBackupJars(serverId)
  .then(result => {
    console.log('✅ List backups success:', result);
    // Verify structure:
    // - backups (string[])
  })
  .catch(err => {
    console.error('❌ List backups error:', err);
  });
```

**Expected Results:**
- ✅ Returns array of backup filenames
- ✅ Backups are sorted newest first
- ✅ Empty array if no backups

## Hook Tests (useServerUpdates)

### Test: Hook State Management

```typescript
// In React component or DevTools
const {
  checking,
  applying,
  rollingBack,
  updateInfo,
  checkUpdates,
  applyServerUpdate,
  rollbackServerUpdate
} = useServerUpdates(serverId);

// 1. Initial state
console.assert(checking === false, '❌ checking should be false initially');
console.assert(applying === false, '❌ applying should be false initially');
console.assert(updateInfo === null, '❌ updateInfo should be null initially');

// 2. After checkUpdates()
await checkUpdates();
console.assert(checking === false, '❌ checking should be false after check completes');
console.assert(updateInfo !== null, '❌ updateInfo should be populated after check');

// 3. Loading states
// Verify that loading states become true during operations
```

**Expected Results:**
- ✅ Initial states are correct
- ✅ Loading states toggle correctly
- ✅ Errors are captured and exposed
- ✅ Success results update state

## UI Component Tests

### Test: ServerOptions Update Section

Navigate to a server's Options page and verify:

1. **Check for Updates Button**
   - [ ] Button is visible
   - [ ] Button shows loading state when checking
   - [ ] Toast notification appears after check
   - [ ] Button is disabled during check

2. **Update Available Banner**
   - [ ] Banner appears when update is available
   - [ ] Current version is displayed
   - [ ] Latest version is displayed
   - [ ] Changelog link works (Vanilla only)

3. **Apply Update Button**
   - [ ] Button is visible in update banner
   - [ ] Button is disabled when server is running
   - [ ] Warning message shows when server is running
   - [ ] Button shows loading state when applying
   - [ ] Success toast appears after update
   - [ ] Banner disappears after successful update

4. **Rollback Button**
   - [ ] Button is always visible
   - [ ] Button shows loading state when rolling back
   - [ ] Success toast appears after rollback

## Integration Tests

### End-to-End Update Flow

1. **Setup**
   - Create a Vanilla 1.20.1 server
   - Stop the server

2. **Check for Update**
   - Click "Check for Updates"
   - Verify toast notification
   - Verify update banner appears (if update available)

3. **Apply Update**
   - Verify server is stopped
   - Click "Apply Update"
   - Wait for completion
   - Verify success toast
   - Check server directory for backup in `backups_jars/`
   - Verify server JAR is updated

4. **Rollback**
   - Click "Rollback to Previous Version"
   - Wait for completion
   - Verify success toast
   - Verify server JAR is restored

### Test Each Loader Type

Repeat the flow above for:
- [ ] Vanilla server
- [ ] Fabric server
- [ ] Forge server
- [ ] NeoForge server
- [ ] Quilt server

## Error Scenario Tests

### Test: Update Running Server

1. Start a server
2. Try to apply update
3. **Expected**: Error message, update blocked

### Test: Rollback Without Backup

1. Create fresh server
2. Try to rollback
3. **Expected**: Error message about no backup

### Test: Network Failure

1. Disconnect network
2. Try to check updates
3. **Expected**: Error message about network failure

## Performance Tests

### Test: Update Large Server JAR

1. Apply update to Forge server (large installer JAR)
2. **Expected**:
   - Progress indication
   - No UI freeze
   - Reasonable completion time (<2 minutes on good connection)

## Browser Compatibility

Test in multiple browsers:
- [ ] Chrome
- [ ] Firefox
- [ ] Edge

## Notes

- Replace `"your-server-id-here"` with actual encoded server IDs
- Server must be stopped to apply updates
- Backup JARs are stored in `server_directory/backups_jars/`
- Update info is cached in database (last_update_check timestamp)
