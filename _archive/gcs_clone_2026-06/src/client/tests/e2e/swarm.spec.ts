import { test, expect } from '@playwright/test';

test.describe('Clone Drone Engage - Swarm AI Test', () => {
  
  test('should successfully start simulator, navigate overlapping waypoints, and enable flock mode without crashing', async ({ page }) => {
    test.setTimeout(300000); // Tăng timeout của Test lên 5 phút (300s) do test chạy rất dài

    const errors: string[] = [];

    // Lắng nghe lỗi console và page error
    page.on('pageerror', (err) => {
      errors.push(`Page Error: ${err.message}`);
    });
    page.on('console', (msg) => {
      if (msg.type() === 'error') {
        const text = msg.text();
        // Bỏ qua lỗi WebSocket do chúng ta đang test giao diện mà không bật Backend
        if (!text.includes('WS_ERROR') && !text.includes('WS_INIT_FAILED') && !text.includes('WebSocket connection')) {
          errors.push(`Console Error: ${text}`);
        }
        if (text.includes('COLLISION DETECTED')) {
          errors.push(`🔥 UI PIXEL COLLISION: ${text}`);
        }
      }
    });

    // MOCK API: Đánh chặn gọi lên Auth Server thật
    await page.route('**/w/wl/', async (route) => {
      const json = {
        e: 0,
        sid: 'mock_session_id_123',
        cs: {
          f: 'mock_temp_key',
          g: 'localhost',
          h: 8080
        }
      };
      await route.fulfill({ json });
    });

    // 1. Vào trang login
    await page.goto('/login');
    
    // 2. Bấm thẳng nút Bypass Login thay vì điền form
    await page.click('text=DEV: Bypass Login');

    // 4. Chờ chuyển trang sang /dashboard
    await expect(page).toHaveURL(/.*\/dashboard/);

    // 5. Khởi động Simulator
    await page.click('text=START SWARM SIM');
    await expect(page.locator('text=D1').first()).toBeVisible();

    // 6. Test Waypoints chồng chéo (Stress Test Collision Avoidance)
    // Đặt Leader thành None để tất cả Drone hoạt động độc lập
    const leaderSelect = page.locator('select').first(); 
    await leaderSelect.selectOption({ value: '' });

    // Đợi UI render xong
    await page.waitForTimeout(1000);

    // Click vào bản đồ (tọa độ pixel trong viewport) ở 1 chỗ trống cách xa trung tâm
    const map = page.locator('.leaflet-container');
    
    // Giao Drone 1 bay tới điểm này
    await page.click('text=D1');
    await page.waitForTimeout(500);
    await map.click({ position: { x: 300, y: 200 } });
    await page.click('text=Fly to here');
    await page.waitForTimeout(500);

    // Giao Drone 2 bay tới ngay sát Drone 1
    await page.click('text=D2');
    await page.waitForTimeout(500);
    await map.click({ position: { x: 305, y: 205 } });
    await page.click('text=Fly to here');
    await page.waitForTimeout(500);

    // Giao Drone 3 bay tới sát đó
    await page.click('text=D3');
    await page.waitForTimeout(500);
    await map.click({ position: { x: 295, y: 195 } });
    await page.click('text=Fly to here');

    // Cho bầy đàn đấu đá chen lấn nhau để chui vào cái lỗ hẹp này trong 30 giây
    console.log("Testing tight space collision avoidance for 30 seconds...");
    await page.waitForTimeout(30000);

    // 7. Chuyển sang Flock Mode
    await page.click('text=FLOCK MODE');

    // Để Flock Mode tự bay hữu cơ trong 30 giây
    console.log("Testing organic flock mode for 30 seconds...");
    await page.waitForTimeout(30000);

    // XÁC MINH (ASSERTION)
    expect(errors.length).toBe(0);
  });
});
