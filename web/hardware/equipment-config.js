export const equipmentExample = {
  schema: 'reforge-equipment/1',
  name: 'Pulse fixture example',
  devices: [
    {id: 'stimulus', name: 'Pico 2', role: 'Signal source', interface: 'USB console · 3.3 V GPIO'},
    {id: 'target', name: 'Cora Z7-07S', role: 'Target controller', interface: 'JTAG · Pmod JA'},
    {id: 'header', name: 'Pmod TPH2', role: 'Test access', interface: '12-pin pass-through'},
    {id: 'scope', name: 'Hantek DSO2D15', role: 'Analog acquisition', interface: 'Probe input · USB'},
    {id: 'logic', name: 'Logic analyzer', role: 'Digital acquisition', interface: 'CH0 / CH1 · USB'},
    {id: 'fixture', name: 'Breadboard', role: 'Interconnect', interface: '2.54 mm terminal strips'}
  ]
};

export function parseEquipment(text) {
  if (typeof text !== 'string' || new TextEncoder().encode(text).length > 65536) {
    throw new Error('Equipment lists must be at most 64 KiB.');
  }
  const value = JSON.parse(text);
  if (value?.schema !== 'reforge-equipment/1' || !Array.isArray(value.devices) || value.devices.length > 64) {
    throw new Error('Use a reforge-equipment/1 list with at most 64 devices.');
  }
  const field = (value, required = true) => {
    if (typeof value !== 'string' || value.length > 256 || /[\u0000-\u001f\u007f]/.test(value) || (required && !value.trim())) {
      throw new Error('Equipment fields must be plain text, at most 256 characters.');
    }
    return value.trim();
  };
  const ids = new Set();
  const devices = value.devices.map(device => {
    const id = field(device?.id);
    if (!/^[a-zA-Z0-9_-]{1,64}$/.test(id) || ids.has(id)) throw new Error('Equipment IDs must be unique letters, numbers, hyphens or underscores.');
    ids.add(id);
    return {id, name: field(device.name), role: field(device.role), interface: field(device.interface ?? '', false)};
  });
  return {schema: 'reforge-equipment/1', name: field(value.name), devices};
}
