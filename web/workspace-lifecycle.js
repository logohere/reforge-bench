const handlers=new Map();
export function workspaceLifecycle(path,callbacks){handlers.set(path,callbacks);}
export function activateWorkspace(path,active){handlers.get(path)?.[active?'show':'hide']?.();}
