(function() {
  // Overriding fetch
  const originalFetch = window.fetch;
  window.fetch = async function(...args) {
    const response = await originalFetch(...args);
    const url = typeof args[0] === 'string' ? args[0] : (args[0] && args[0].url) || '';
    
    if (url && url.includes('getReleaseListAjax.do')) {
      try {
        const clone = response.clone();
        const text = await clone.text();
        // ponytail: post message directly to parent frame
        window.postMessage({
          type: 'SSCM_AJAX_DATA',
          url: url,
          response: text
        }, '*');
      } catch (err) {
        console.error('[SSCM Interceptor] Error intercepting fetch response:', err);
      }
    }
    return response;
  };

  // Overriding XMLHttpRequest
  const originalOpen = XMLHttpRequest.prototype.open;
  const originalSend = XMLHttpRequest.prototype.send;

  XMLHttpRequest.prototype.open = function(method, url, ...rest) {
    this._url = url;
    this._method = method;
    return originalOpen.apply(this, [method, url, ...rest]);
  };

  XMLHttpRequest.prototype.send = function(body) {
    this.addEventListener('load', function() {
      if (this._url && this._url.includes('getReleaseListAjax.do')) {
        try {
          // ponytail: forward intercepted XHR details
          window.postMessage({
            type: 'SSCM_AJAX_DATA',
            url: this._url,
            payload: body,
            response: this.responseText
          }, '*');
        } catch (err) {
          console.error('[SSCM Interceptor] Error intercepting XHR response:', err);
        }
      }
    });
    return originalSend.apply(this, [body]);
  };

  console.log('[SSCM Interceptor] AJAX interceptors initialized.');
})();
