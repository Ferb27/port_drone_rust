const WebSocket = require('ws');

// Cấu hình từ Auth Server response
const host = 'andruav.com';
const port = '9669';
const tempKey = '46be8b7620304f97bfb755ba653afa0d';
const sessionId = '9401c1a198954e09934383e390a4617e246339978';
const protocol = 'wss';

// Theo bundle: wss://{ip}:{port}/?f={authKey}&s={partyID}&at=g
const url = `${protocol}://${host}:${port}/?f=${tempKey}&s=${sessionId}&at=g`;
console.log('Connecting to:', url);

const ws = new WebSocket(url, {
  rejectUnauthorized: false
});

ws.on('open', () => {
  console.log('WebSocket connected!');
  // Gửi thử một message rỗng hoặc đợi server gửi
});

ws.on('message', (data) => {
  const msg = data.toString();
  console.log('Received:', msg);
});

ws.on('close', () => {
  console.log('WebSocket closed');
});

ws.on('error', (err) => {
  console.error('WebSocket error:', err.message);
});
