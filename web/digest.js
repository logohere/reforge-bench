import {sha256 as fallback} from '@noble/hashes/sha2.js';

export async function sha256(data){
  const bytes=typeof data==='string'?new TextEncoder().encode(data):data instanceof ArrayBuffer?new Uint8Array(data):data;
  if(!(bytes instanceof Uint8Array))throw new TypeError('Expected text or bytes');
  const result=globalThis.crypto?.subtle?new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)):fallback(bytes);
  return Array.from(result,byte=>byte.toString(16).padStart(2,'0')).join('');
}
