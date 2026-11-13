const systems={
  'ddcs-expert':'c70ff4eceadb98c811ccfe81efd0e62bd3d6c13e5ac18dfb8b590f75790f9b05',
  'ddcs-v4':'087c31ed22841b01afc0cb5ecae3882e50cc7eafada79dfb64e57e70acd847a0',
  'ddcs-v4-driver':'6e309c6368aaacf4a2b406f645f8f631f43e2a6778c934ed2ed67f7d684a4eee',
  'ddcs-m350-driver':'f762e6192097eedf28d82d446b2742089fa87c6d81dbf1b8f01c596b91b955c8',
  'haas-lathe':'148fde6e22ee1e7ef6e16fb45c5bd70776feb53c20979a4b6e9289162461b9d7',
  'haas-mill':'7b33f8e7f5f290c34e9b21eb4027eab6ab4a5fbea7b725b9c7974fcd82c15171',
  'haas-loader':'e9b9039e1241a54233033ffb5bfe135c7fc3d177d3793e8df3ff8a25f123a898',
  'acorn':'0050567caf5803fb89b6e5892dd7fbfccfbeecd50d80ababdc813375b30f0e4f',
  'magician-lite':'bae416427e968d2afcca801382bd31e09c67af1db306e5c4d459599cdce6e5d0',
  'fanuc-crx-bootrom':'030b11c8f81b2c833346529cf15e08243d5538b5f3ccd34c2952e834e3a1f924',
  'fanuc-crx-ioscan':'03bb91461e133b295d30ef2ac18a434590895ee6acbfbcc0dd797c7d9f57767f',
  'fanuc-crx-mjog':'23ea714d3e0928c73e0eadb5d97bd70097327e854ef68efedf587dcb74560570',
  'fanuc-crx-mpplan':'c9f8bd844c4400d747135baadd0a6fa5bd6ac1100fcf3a9bfb932967d47e9aba',
  'mdx-40a':'71727fd495a299800030b185207b3a114f64f6642f0a05eca9528282bdab0c95'
};
export function systemCase(search){const params=new URLSearchParams(search);if(params.has('case'))return params.get('case');if(params.has('system'))return Object.hasOwn(systems,params.get('system'))?systems[params.get('system')]:'';return null;}
export function deviceQuery(id){const slug=Object.keys(systems).find(key=>systems[key]===id);return slug?'?system='+slug:'?case='+id;}
