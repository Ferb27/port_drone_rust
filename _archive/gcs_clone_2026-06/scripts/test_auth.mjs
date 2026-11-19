import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Simple env parser
const envPath = path.join(__dirname, '../config/.env');
let email = '';
let password = '';

if (fs.existsSync(envPath)) {
    const envFile = fs.readFileSync(envPath, 'utf-8');
    envFile.split('\n').forEach(line => {
        if (line.startsWith('DE_EMAIL=')) email = line.replace('DE_EMAIL=', '').trim();
        if (line.startsWith('DE_PASSWORD=')) password = line.replace('DE_PASSWORD=', '').trim();
    });
}

const AUTH_URL = 'https://cloud.ardupilot.org:19408/wl/0';

async function testAuth() {
    console.log(`[+] Bắt đầu gửi yêu cầu xác thực tới: ${AUTH_URL}`);
    console.log(`[+] Email: ${email}`);

    try {
        const response = await fetch(AUTH_URL, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                email: email,
                password: password,
                GCS_ID: "test_gcs_antigravity",
                app_name: "Clone_Drone_Engage",
                app_ver: "1.0.0"
            })
        });

        const data = await response.json();
        console.log(`[+] Kết quả trả về (JSON):`);
        console.log(JSON.stringify(data, null, 2));
        
        if (data.cs) {
            console.log(`\n[SUCCESS] Lấy được thông tin Communication Server!`);
            console.log(`Host: ${data.cs.g}:${data.cs.h}`);
            console.log(`Temp Key: ${data.cs.f}`);
            console.log(`Session ID: ${data.sid}`);
        } else {
            console.log(`\n[FAILED] Không lấy được Communication Server info. Vui lòng kiểm tra lại thông tin.`);
        }
        
    } catch (error) {
        console.error(`[ERROR] Có lỗi khi gọi API:`, error.message);
    }
}

testAuth();
