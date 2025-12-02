export const PARTYKIT_HOST = import.meta.env.VITE_PARTYKIT_HOST || 'localhost:1999';
console.log(
  '🌐 PARTYKIT_HOST resolved to:',
  PARTYKIT_HOST,
  'from env:',
  import.meta.env.VITE_PARTYKIT_HOST
);
