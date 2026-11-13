import {element as node, commandButton} from '../ui/components.js';
import {equipmentExample, parseEquipment} from './equipment-config.js';

export function mountEquipment() {
  const root = node('section', undefined, 'hardware-equipment');
  const heading = node('div', undefined, 'hardware-region-heading');
  heading.append(node('h2', 'Equipment'), node('span', 'Your project’s equipment list'));
  const actions = node('div', undefined, 'hardware-acquisition-actions');
  const add = commandButton('Add device', {kind: 'primary'});
  const load = commandButton('Import list', {kind: 'secondary'});
  const save = commandButton('Export list', {kind: 'secondary'});
  const example = commandButton('Download example', {kind: 'ghost'});
  const file = node('input');
  file.type = 'file'; file.accept = '.json,application/json'; file.hidden = true;
  const status = node('p', '', 'hardware-acquisition-state');
  status.setAttribute('role', 'status');
  const name = node('input'); name.type = 'text'; name.maxLength = 256;
  const label = node('label', 'Project / bench name', 'hardware-equipment-name'); label.append(name);
  const list = node('div', undefined, 'hardware-equipment-list');
  actions.append(add, load, save, example, file);
  root.append(heading, actions, label, status, list,
    node('p', 'Equipment records are editable. The 2D/3D views and wiring guide remain the example preset; adapters and physical pinouts are configured separately.', 'hardware-acquisition-note'));
  const storageName = 'reforge.equipment.v1';
  let data = structuredClone(equipmentExample);
  try {
    const saved = localStorage.getItem(storageName);
    if (saved) data = parseEquipment(saved);
  } catch { status.textContent = 'Saved equipment could not be read. The example is shown; saved data has not been replaced.'; }
  function store() {
    try {
      parseEquipment(JSON.stringify(data));
      localStorage.setItem(storageName, JSON.stringify(data));
      status.textContent = 'Saved locally';
    } catch (error) { status.textContent = error.message; }
  }
  function paint() {
    name.value = data.name; list.replaceChildren(); add.disabled = data.devices.length >= 64;
    if (!data.devices.length) list.append(node('p', 'No equipment yet. Add a device or import a list.'));
    for (const device of data.devices) {
      const card = node('fieldset', undefined, 'hardware-equipment-card');
      card.append(node('legend', device.name));
      for (const [key, title] of [['name', 'Device'], ['role', 'Role'], ['interface', 'Interface']]) {
        const input = node('input'); input.type = 'text'; input.maxLength = 256; input.value = device[key];
        const field = node('label', title); field.append(input); card.append(field);
        input.addEventListener('change', () => {
          const previous = device[key]; device[key] = input.value;
          try { parseEquipment(JSON.stringify(data)); }
          catch (error) { device[key] = previous; input.value = previous; status.textContent = error.message; return; }
          if (key === 'name') card.querySelector('legend').textContent = device.name;
          store();
        });
      }
      const remove = commandButton('Remove', {kind: 'ghost', size: 'sm'});
      remove.setAttribute('aria-label', 'Remove ' + device.name);
      remove.addEventListener('click', () => { data.devices = data.devices.filter(item => item.id !== device.id); store(); paint(); });
      card.append(remove); list.append(card);
    }
  }
  name.addEventListener('change', () => {
    const previous = data.name; data.name = name.value;
    try { parseEquipment(JSON.stringify(data)); }
    catch (error) { data.name = previous; name.value = previous; status.textContent = error.message; return; }
    store();
  });
  add.addEventListener('click', () => {
    if (data.devices.length >= 64) return;
    data.devices.push({id: crypto.randomUUID(), name: 'New device', role: 'Unassigned', interface: ''});
    store(); paint(); list.lastElementChild?.querySelector('input')?.focus();
  });
  load.addEventListener('click', () => file.click());
  file.addEventListener('change', async () => {
    const selected = file.files[0]; if (!selected) return;
    try {
      if (selected.size > 65536) throw new Error('Equipment lists must be at most 64 KiB.');
      data = parseEquipment(await selected.text()); store(); paint();
    } catch (error) { status.textContent = error.message; }
    finally { file.value = ''; }
  });
  function download(value) {
    const url = URL.createObjectURL(new Blob([JSON.stringify(value, null, 2) + '\n'], {type: 'application/json'}));
    const link = node('a'); link.href = url; link.download = 'equipment.json'; link.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  }
  save.addEventListener('click', () => download(data));
  example.addEventListener('click', () => download(equipmentExample));
  paint();
  return {root, dispose() {}};
}
