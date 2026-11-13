(() => {
  if (window !== window.top || window.__reaAgent) return;
  const events = [];
  const push = event => {
    if (events.length === 200) events.shift();
    events.push({at: performance.now(), context: window.__reaDebugContext?.(), ...event});
  };
  const route = value => {
    try { const url = new URL(value, location.href); return url.origin === location.origin ? url.pathname : '[external]'; }
    catch { return ''; }
  };
  for (const level of ['log', 'info', 'warn', 'error', 'debug']) {
    const original = console[level];
    console[level] = function (...args) {
      push({type: 'console', level, message: args.map(value => typeof value === 'string' ? value : value instanceof Error ? value.message : `[${typeof value}]`).join(' ').slice(0, 1600)});
      return original.apply(this, args);
    };
  }
  const originalFetch = window.fetch;
  window.fetch = async function (input, options) {
    const started = performance.now();
    const url = route(typeof input === 'string' || input instanceof URL ? String(input) : input.url);
    const method = options?.method || input?.method || 'GET';
    try {
      const response = await originalFetch.apply(this, arguments);
      push({type: 'network', transport: 'fetch', method, path: url, status: response.status, ms: performance.now() - started});
      return response;
    } catch (error) {
      push({type: 'network', transport: 'fetch', method, path: url, error: String(error).slice(0, 320), ms: performance.now() - started});
      throw error;
    }
  };
  window.addEventListener('error', event => push({type: 'error', message: String(event.message || 'Resource failed').slice(0, 320)}), true);
  window.addEventListener('unhandledrejection', event => push({type: 'rejection', message: String(event.reason?.message || event.reason).slice(0, 320)}));
  const snapshot = () => {
    const nodes = [];
    const visit = root => {
      for (const node of root.children || []) {
        if (nodes.length >= 1500) return;
        const rect = node.getBoundingClientRect();
        if (rect.width && rect.height) {
          const style = getComputedStyle(node);
          if (style.visibility !== 'hidden' && style.display !== 'none') {
            const interactive = node.matches('button,input,select,textarea,a,[role],[tabindex],cds-button,cds-radio-button,cds-tab');
            if (interactive || node.id) nodes.push({tag: node.localName, id: node.id || null,
              action: node.getAttribute('data-action'), role: node.getAttribute('role'),
              label: node.getAttribute('aria-label') || node.getAttribute('label-text') || (node.type === 'password' ? '[secure]' : node.textContent.trim().slice(0, 160)),
              disabled: Boolean(node.disabled), checked: typeof node.checked === 'boolean' ? node.checked : null,
              rect: [rect.x, rect.y, rect.width, rect.height], focused: root.activeElement === node});
          }
        }
        if (node.shadowRoot) visit(node.shadowRoot);
        visit(node);
      }
    };
    visit(document);
    return {path: route(location.href), title: document.title, context: window.__reaDebugContext?.(), nodes, events: events.slice(),
      resources: performance.getEntriesByType('resource').slice(-100).map(entry => ({path: route(entry.name), type: entry.initiatorType, ms: entry.duration, bytes: entry.transferSize}))};
  };
  Object.defineProperty(window, '__reaAgent', {value: Object.freeze({snapshot})});
})();
