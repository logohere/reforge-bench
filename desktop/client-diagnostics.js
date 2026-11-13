(() => {
  if (window !== window.top || window.__reaDiagnosticsInstalled) return;
  Object.defineProperty(window, '__reaDiagnosticsInstalled', {value:true});
  const text = (value, limit) => typeof value === 'string' ? value.replace(/[\x00-\x1f\x7f]/g, ' ').slice(0, limit) : '';
  const route = value => {
    try { const url = new URL(value, location.href); return url.origin === location.origin ? url.pathname.slice(0, 240) : '[external]'; }
    catch { return ''; }
  };
  const session = `${Date.now().toString(36)}-${Math.random().toString(36).slice(2,10)}`;
  let action = '', sequence = 0, sent = 0, interval = Date.now();
  Object.defineProperty(window, '__reaDebugContext', {value: () => ({session, action, errorSequence: sequence})});
  const remember = event => {
    for (const node of event.composedPath()) {
      const id = node.getAttribute?.('data-action') || node.id;
      if (typeof id === 'string' && /^[a-z][a-z0-9_.:-]{0,95}$/i.test(id)) { action = id; break; }
    }
  };
  document.addEventListener('click', remember, true);
  document.addEventListener('change', remember, true);
  document.addEventListener('keydown', event => { if (event.key === 'Enter' || event.key === ' ') remember(event); }, true);
  const send = (phase, message, source, line, column, stack) => {
    if (Date.now() - interval >= 10000) { interval = Date.now(); sent = 0; }
    if (sent++ >= 20) return;
    try {
      window.ipc.postMessage(JSON.stringify({kind:'client-error',phase,session,sequence:++sequence,
        path:route(location.href),action,message:text(message,320),source:route(source || location.href),
        line:Number.isSafeInteger(line) && line > 0 ? line : 0,
        column:Number.isSafeInteger(column) && column > 0 ? column : 0,
        stack:text(stack,1600).replace(/https?:\/\/[^\s)]+/g, value => route(value))}));
    } catch { /* Reporting must not replace the original failure. */ }
  };
  window.addEventListener('error', event => {
    if (event.message) send('uncaught', event.message, event.filename, event.lineno, event.colno, event.error?.stack);
    else if (event.target && event.target !== window) send('resource', `Failed to load ${event.target.tagName || 'resource'}`, event.target.src || event.target.href);
  }, true);
  window.addEventListener('unhandledrejection', event => {
    const reason = event.reason;
    send('rejection', typeof reason === 'string' ? reason : typeof reason?.message === 'string' ? reason.message : `Unhandled rejection (${typeof reason})`, '', 0, 0, reason?.stack);
  });
})();
