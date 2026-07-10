// Inject inject.js into the main world context
try {
  const script = document.createElement('script');
  script.src = chrome.runtime.getURL('inject.js');
  script.onload = function() {
    this.remove();
  };
  (document.head || document.documentElement).appendChild(script);
} catch (e) {
  console.error('[SSCM Content] Failed to inject interception script:', e);
}

// Global cached response
let lastInterceptedData = null;
let tauriConnected = false;

// Create floating status widget on the page
function createFloatingStatus() {
  const container = document.createElement('div');
  container.id = 'sscm-sync-badge';
  container.style.position = 'fixed';
  container.style.bottom = '16px';
  container.style.right = '16px';
  container.style.padding = '8px 12px';
  container.style.borderRadius = '8px';
  container.style.background = 'rgba(10, 14, 23, 0.9)';
  container.style.border = '1px solid rgba(6, 182, 212, 0.3)';
  container.style.color = '#fff';
  container.style.fontFamily = 'sans-serif';
  container.style.fontSize = '12px';
  container.style.zIndex = '99999';
  container.style.display = 'flex';
  container.style.alignItems = 'center';
  container.style.gap = '8px';
  container.style.boxShadow = '0 4px 12px rgba(0,0,0,0.5)';
  container.style.pointerEvents = 'none';
  container.style.transition = 'all 0.3s ease';

  const dot = document.createElement('span');
  dot.style.width = '8px';
  dot.style.height = '8px';
  dot.style.borderRadius = '50%';
  dot.style.backgroundColor = '#6b7280'; // grey initial
  dot.style.display = 'inline-block';
  dot.id = 'sscm-badge-dot';

  const text = document.createElement('span');
  text.textContent = 'SSCM Sync: Connecting...';
  text.id = 'sscm-badge-text';

  container.appendChild(dot);
  container.appendChild(text);
  document.body.appendChild(container);

  // Check connection initially
  checkTauriConnection();
  setInterval(checkTauriConnection, 5000);
}

// Ping local Tauri server to check status
function checkTauriConnection() {
  fetch('http://127.0.0.1:14120/', { method: 'OPTIONS' })
    .then(() => {
      updateBadgeStatus(true);
    })
    .catch(() => {
      updateBadgeStatus(false);
    });
}

function updateBadgeStatus(connected) {
  tauriConnected = connected;
  const dot = document.getElementById('sscm-badge-dot');
  const text = document.getElementById('sscm-badge-text');
  const badge = document.getElementById('sscm-sync-badge');
  
  if (dot && text && badge) {
    if (connected) {
      dot.style.backgroundColor = '#10b981'; // green
      dot.style.boxShadow = '0 0 8px #10b981';
      text.textContent = 'SSCM Sync: Connected';
      badge.style.borderColor = 'rgba(16, 185, 129, 0.4)';
    } else {
      dot.style.backgroundColor = '#ef4444'; // red
      dot.style.boxShadow = '0 0 8px #ef4444';
      text.textContent = 'SSCM Sync: Disconnected (Tauri App Closed)';
      badge.style.borderColor = 'rgba(239, 68, 68, 0.4)';
    }
  }
}

// Run layout script after DOM is ready
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', createFloatingStatus);
} else {
  createFloatingStatus();
}

// Receive messages from inject.js
window.addEventListener('message', function(event) {
  if (event.source !== window || !event.data || event.data.type !== 'SSCM_AJAX_DATA') return;
  
  console.log('[SSCM Content] Intercepted getReleaseListAjax.do response payload.');
  
  lastInterceptedData = event.data;
  
  // Save to extension session storage so popup can pull it
  try {
    chrome.storage.local.set({ lastSSCMData: event.data });
  } catch (err) {
    // Chrome context might not have local storage active if background is dead
  }

  // Push directly to Tauri local API
  fetch('http://127.0.0.1:14120/api/submit', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify(event.data)
  })
  .then(res => res.json())
  .then(data => {
    console.log('[SSCM Content] Successfully synced payload to Tauri app server:', data);
    flashBadge(true);
  })
  .catch(err => {
    console.warn('[SSCM Content] Failed to sync data to Tauri (App might be closed).', err);
    flashBadge(false);
  });
});

// Flash badge to provide feedback on sync
function flashBadge(success) {
  const badge = document.getElementById('sscm-sync-badge');
  if (badge) {
    const originalBg = badge.style.background;
    badge.style.background = success ? 'rgba(16, 185, 129, 0.9)' : 'rgba(239, 68, 68, 0.9)';
    setTimeout(() => {
      badge.style.background = originalBg;
    }, 1000);
  }
}

// Listen for popup info requests
chrome.runtime.onMessage.addListener(function(request, sender, sendResponse) {
  if (request.type === 'GET_LAST_DATA') {
    sendResponse({ lastData: lastInterceptedData, connected: tauriConnected });
  }
  return true;
});
