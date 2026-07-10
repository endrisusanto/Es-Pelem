document.addEventListener('DOMContentLoaded', () => {
  const tauriStatus = document.getElementById('tauri-status');
  const requestUrl = document.getElementById('request-url');
  const capturedCount = document.getElementById('captured-count');
  const copyBtn = document.getElementById('copy-btn');
  const syncBtn = document.getElementById('sync-btn');

  let currentData = null;

  // Retrieve cached intercept details from local storage
  chrome.storage.local.get(['lastSSCMData'], (result) => {
    if (result.lastSSCMData) {
      updateUI(result.lastSSCMData);
    }
  });

  // Query content script on the current tab
  chrome.tabs.query({ active: true, currentWindow: true }, (tabs) => {
    if (tabs[0]) {
      chrome.tabs.sendMessage(tabs[0].id, { type: 'GET_LAST_DATA' }, (response) => {
        if (response) {
          if (response.lastData) {
            updateUI(response.lastData);
          }
          setTauriStatus(response.connected);
        }
      });
    }
  });

  // Direct connection check to port 14120
  fetch('http://127.0.0.1:14120/', { method: 'OPTIONS' })
    .then(() => setTauriStatus(true))
    .catch(() => setTauriStatus(false));

  function setTauriStatus(connected) {
    if (connected) {
      tauriStatus.textContent = 'CONNECTED';
      tauriStatus.className = 'badge badge-connected';
      if (currentData) syncBtn.disabled = false;
    } else {
      tauriStatus.textContent = 'OFFLINE';
      tauriStatus.className = 'badge badge-disconnected';
      syncBtn.disabled = true;
    }
  }

  function updateUI(data) {
    currentData = data;
    const cleanUrl = data.url ? data.url.split('?')[0] : 'Unknown';
    requestUrl.textContent = cleanUrl;
    
    try {
      const parsed = typeof data.response === 'string' ? JSON.parse(data.response) : data.response;
      const list = parsed.objects || parsed.list || [];
      capturedCount.textContent = `${list.length} releases`;
      
      if (list.length > 0) {
        copyBtn.disabled = false;
        if (tauriStatus.textContent === 'CONNECTED') {
          syncBtn.disabled = false;
        }
      }
    } catch (e) {
      capturedCount.textContent = 'Invalid payload format';
    }
  }

  // Copy response body
  copyBtn.addEventListener('click', () => {
    if (currentData && currentData.response) {
      const textToCopy = typeof currentData.response === 'string' 
        ? currentData.response 
        : JSON.stringify(currentData.response, null, 2);
        
      navigator.clipboard.writeText(textToCopy);
      const originalText = copyBtn.textContent;
      copyBtn.textContent = 'Copied!';
      setTimeout(() => {
        copyBtn.textContent = originalText;
      }, 1500);
    }
  });

  // Manual re-trigger sync
  syncBtn.addEventListener('click', () => {
    if (currentData) {
      syncBtn.disabled = true;
      fetch('http://127.0.0.1:14120/api/submit', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(currentData)
      })
      .then(res => res.json())
      .then(() => {
        syncBtn.textContent = 'Synced!';
        setTimeout(() => {
          syncBtn.textContent = 'Resync to Tauri';
          syncBtn.disabled = false;
        }, 1500);
      })
      .catch(() => {
        alert('Failed to synchronize. The Tauri app might be closed.');
        setTauriStatus(false);
      });
    }
  });
});
